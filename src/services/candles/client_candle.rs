use crate::app_state::models::AppState;

use crate::db::clickhouse::clickhouse_service::ClickhouseService;

use crate::env_config::models::app_config::AppConfig;
use crate::env_config::models::app_setting::AppSettings;
use crate::generate::tinkoff_public_invest_api_contract_v1::{
    CandleInterval, GetCandlesRequest, HistoricCandle,
};
use crate::services::tinkoff_client_grpc::TinkoffClient;
use crate::utils::utils_date_time;

use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// Клиент для работы с API свечей Tinkoff
/// Предоставляет функциональность для загрузки и сохранения свечей в БД
pub struct ClientCandle {
    clickhouse_service: Arc<ClickhouseService>,
    grpc_tinkoff: Arc<TinkoffClient>,
    settings: Arc<AppSettings>, // Store a reference to the existing Arc<AppSettings>
}

impl ClientCandle {
    pub fn new(
        clickhouse_service: Arc<ClickhouseService>,
        grpc_tinkoff: Arc<TinkoffClient>,
        settings: Arc<AppSettings>, 
    ) -> Self {
        Self {
            clickhouse_service,
            grpc_tinkoff,
            settings,
        }
    }

    // Then modify the get_minute_candles method to use the config
    pub async fn get_minute_candles(
        &self,
        uid: &str,
        from: i64,
        to: i64,
    ) -> Result<Vec<HistoricCandle>, Box<dyn std::error::Error>> {
        info!(
            "Requesting minute candles for {} from {} to {}",
            uid, from, to
        );

        // Create request to Tinkoff API
        let request = GetCandlesRequest {
            from: Some(prost_types::Timestamp {
                seconds: from,
                nanos: 0,
            }),
            to: Some(prost_types::Timestamp {
                seconds: to,
                nanos: 0,
            }),
            instrument_id: uid.to_string(),
            figi: "".to_string(),
            interval: CandleInterval::CandleInterval1Min as i32,
        };

        // Execute request
        let grpc_request = self.grpc_tinkoff.create_request(request)?;
        let mut market_data_client = self.grpc_tinkoff.market_data.clone();

        let response = market_data_client.get_candles(grpc_request).await?;
        let candles_response = response.into_inner();

        info!(
            "Received {} candles for {}",
            candles_response.candles.len(),
            uid
        );

        // Apply the configured delay after API call
        let delay_ms = self.settings.app_config.candles_scheduler.request_delay_ms;
        if delay_ms > 0 {
            debug!("Applying API request delay of {}ms", delay_ms);
            tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
        }

        Ok(candles_response.candles)
    }

    /// Загружает и сохраняет свечи для одного инструмента
    ///
    /// # Arguments
    /// * `instrument_id` - Идентификатор инструмента
    /// * `first_candle_date` - Дата первой возможной свечи для инструмента
    /// * `index` - Индекс инструмента в общем списке
    /// * `total` - Общее количество инструментов в списке
    ///
    /// # Returns
    /// Количество обработанных свечей или ошибку
    async fn process_instrument(
        &self,
        instrument_id: &str,
        first_1min_candle_date: i64,
        last_1min_candle_date: i64,
        index: usize,
        total: usize,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        info!(
            "Processing instrument {}/{}: {}",
            index + 1,
            total,
            instrument_id
        );

        // Get current date and calculate yesterday's end
        let (_, yesterday_end) = utils_date_time::get_yesterday_range(None);

        // Check if we've already reached yesterday
        if last_1min_candle_date >= yesterday_end {
            debug!(
                "Already up to date for {}, last update to {}",
                instrument_id, last_1min_candle_date
            );
            return Ok(0);
        }

        // Start from the last recorded date or the first possible date
        let mut current_date = last_1min_candle_date;
        if current_date == 0 {
            current_date = first_1min_candle_date;
        }

        // Process all days from last saved to yesterday
        let mut total_day_candles = 0;
        let mut days_processed = 0;
        let mut latest_timestamp = current_date;

        while current_date < yesterday_end {
            let (next_day_start, next_day_end) = utils_date_time::get_next_day_range(current_date);

            // Make sure we don't exceed yesterday
            let end_time = std::cmp::min(next_day_end, yesterday_end);

            debug!(
                "Fetching day {}: {} to {} for {}",
                days_processed + 1,
                next_day_start,
                end_time,
                instrument_id
            );

            let vec_candles: Vec<HistoricCandle> = self
                .get_minute_candles(instrument_id, next_day_start, end_time)
                .await?;

            let day_candles = vec_candles.len();
            total_day_candles += day_candles;

            // Save candles only if there are data
            if !vec_candles.is_empty() {
                // Find the latest timestamp in this batch of candles
                if let Some(last_candle) = vec_candles.last() {
                    if let Some(time) = &last_candle.time {
                        latest_timestamp = time.seconds;
                    }
                }

                // Insert candles
                self.clickhouse_service
                    .repository_candle
                    .insert_candles(vec_candles, instrument_id)
                    .await?;

                // Update the last candle date in the database after each successful batch
                self.clickhouse_service
                    .repository_my_instrument
                    .update_last_candle_date(instrument_id, latest_timestamp)
                    .await?;
            }

            // Move to the next day
            current_date = next_day_end;
            days_processed += 1;
        }

        info!(
            "Completed processing {} days with total {} candles for {} ({}/{})",
            days_processed,
            total_day_candles,
            instrument_id,
            index + 1,
            total
        );

        Ok(index + 1)
    }

    pub async fn load_and_save_candles(&self) -> Result<usize, Box<dyn std::error::Error>> {
        // Get list of instruments with their candle info
        let my_instruments = self
            .clickhouse_service
            .repository_my_instrument
            .get_my_instrument()
            .await?;

        if my_instruments.is_empty() {
            warn!("No instruments found to process");
            return Ok(0);
        }

        info!("Starting to process {} instruments", my_instruments.len());

        let mut processed_count = 0;

        // Process each instrument
        for (index, instrument) in my_instruments.iter().enumerate() {
            match self
                .process_instrument(
                    &instrument.uid,
                    instrument.first_1min_candle_date,
                    instrument.last_1min_candle_date,
                    index,
                    my_instruments.len(),
                )
                .await
            {
                Ok(_) => {
                    processed_count += 1;
                    debug!(
                        "Successfully processed instrument {}/{}: {}",
                        index + 1,
                        my_instruments.len(),
                        instrument.uid
                    );
                }
                Err(e) => {
                    error!(
                        "Error processing instrument {}/{}: {}: {}",
                        index + 1,
                        my_instruments.len(),
                        instrument.uid,
                        e
                    );
                    // Continue with the next instrument
                }
            }

            // Add a small delay between instruments to avoid API throttling
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }

        info!(
            "Completed processing {} out of {} instruments",
            processed_count,
            my_instruments.len()
        );

        Ok(processed_count)
    }
}
