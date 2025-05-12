use crate::app_state::models::AppState;

use crate::db::clickhouse::models::candle::DbCandle;
use crate::generate::tinkoff_public_invest_api_contract_v1::{
    CandleInterval, GetCandlesRequest, HistoricCandle,
};
use chrono::{DateTime, Datelike, Duration, NaiveDateTime, TimeZone, Utc};

use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// Клиент для работы с API свечей Tinkoff
/// Предоставляет функциональность для загрузки и сохранения свечей в БД
pub struct TinkoffCandleClient {
    app_state: Arc<AppState>,
}

impl TinkoffCandleClient {
    /// Создает новый экземпляр клиента для работы с API свечей
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self { app_state }
    }

    /// Получение минутных свечей за указанный период
    ///
    /// # Arguments
    /// * `instrument_id` - Идентификатор инструмента
    /// * `from` - Начало периода в Unix timestamp (секунды)
    /// * `to` - Конец периода в Unix timestamp (секунды)
    ///
    /// # Returns
    /// Вектор исторических свечей или ошибку
    async fn get_minute_candles(
        &self,
        instrument_id: &str,
        from: i64,
        to: i64,
    ) -> Result<Vec<HistoricCandle>, Box<dyn std::error::Error>> {
        info!(
            "Requesting minute candles for {} from {} to {}",
            instrument_id, from, to
        );

        // Создаем запрос к Tinkoff API
        let request = GetCandlesRequest {
            from: Some(prost_types::Timestamp {
                seconds: from,
                nanos: 0,
            }),
            to: Some(prost_types::Timestamp {
                seconds: to,
                nanos: 0,
            }),
            instrument_id: instrument_id.to_string(),
            figi: "".to_string(),
            interval: CandleInterval::CandleInterval1Min as i32,
        };

        // Выполняем запрос
        let grpc_request = self.app_state.grpc_tinkoff.create_request(request)?;
        let mut market_data_client = self.app_state.grpc_tinkoff.market_data.clone();

        let response = market_data_client.get_candles(grpc_request).await?;
        let candles_response = response.into_inner();

        info!(
            "Received {} candles for {}",
            candles_response.candles.len(),
            instrument_id
        );

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
        first_candle_date: i64,
        index: usize,
        total: usize,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        info!(
            "Processing instrument {}/{}: {}",
            index + 1,
            total,
            instrument_id
        );

        // Получаем статус загрузки свечей для этого инструмента
        let status_candle = self
            .app_state
            .postgres_service
            .repository_tinkoff_candles_status
            .get_by_instrument_uid(instrument_id)
            .await?;

        // Получаем текущую дату и вычисляем дату предыдущего дня
        let now = Utc::now();
        let yesterday_end = Utc
            .with_ymd_and_hms(now.year(), now.month(), now.day(), 0, 0, 0)
            .single()
            .unwrap_or_default()
            .timestamp();

        let mut candles_processed = 0;

        if let Some(status) = status_candle {
            // Инструмент уже обрабатывался ранее
            let mut current_to_second = status.to_second;

            // Проверяем, не достигли ли мы уже вчерашнего дня
            if current_to_second >= yesterday_end {
                debug!(
                    "Already up to date for {}, last update to {}",
                    instrument_id, current_to_second
                );
                return Ok(0);
            }

            // Последовательно обрабатываем все дни с последнего сохраненного до вчерашнего
            let mut total_day_candles = 0;
            let mut days_processed = 0;

            while current_to_second < yesterday_end {
                let (next_day_start, next_day_end) = get_next_day_range(current_to_second);

                // Проверяем, не выходит ли следующий день за пределы вчерашнего
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

                // Сохраняем только если есть данные
                if !vec_candles.is_empty() {
                    self.app_state
                        .clickhouse_service
                        .repository_candle
                        .insert_candles(vec_candles, instrument_id)
                        .await?;

                    self.app_state
                        .postgres_service
                        .repository_tinkoff_candles_status
                        .update_to_second(instrument_id, end_time)
                        .await?;

                    info!(
                        "Processed day {}: {} candles for {} ({}/{})",
                        days_processed + 1,
                        day_candles,
                        instrument_id,
                        index + 1,
                        total
                    );
                }

                // Обновляем текущую позицию для следующей итерации
                current_to_second = end_time;
                days_processed += 1;

                // Добавляем небольшую задержку между запросами к API
                tokio::time::sleep(std::time::Duration::from_millis(250)).await;
            }

            candles_processed = total_day_candles;
            info!(
                "Completed processing {} days with total {} candles for {} ({}/{})",
                days_processed,
                total_day_candles,
                instrument_id,
                index + 1,
                total
            );
        } else {
            // Первая загрузка для этого инструмента
            let end_of_day = get_end_of_day(first_candle_date);

            let vec_candles: Vec<HistoricCandle> = self
                .get_minute_candles(instrument_id, first_candle_date, end_of_day)
                .await?;

            candles_processed = vec_candles.len();

            // Сохраняем только если есть данные
            if !vec_candles.is_empty() {
                self.app_state
                    .clickhouse_service
                    .repository_candle
                    .insert_candles(vec_candles, instrument_id)
                    .await?;

                self.app_state
                    .postgres_service
                    .repository_tinkoff_candles_status
                    .upsert(instrument_id, end_of_day)
                    .await?;
            }
        }

        info!(
            "Processed {} candles for instrument {}/{}: {}",
            candles_processed,
            index + 1,
            total,
            instrument_id
        );
        Ok(candles_processed)
    }

    /// Загрузка и сохранение свечей для всех ликвидных инструментов в БД
    ///
    /// # Returns
    /// Общее количество обработанных инструментов или ошибку
    pub async fn load_and_save_candles(&self) -> Result<usize, Box<dyn std::error::Error>> {
        // Получаем список ликвидных акций
        let liquid_shares = self
            .app_state
            .clickhouse_service
            .share_repository
            .get_liquid_shares()
            .await?;

        if liquid_shares.is_empty() {
            warn!("No liquid shares found to process");
            return Ok(0);
        }

        info!("Starting to process {} liquid shares", liquid_shares.len());

        let mut processed_count = 0;

        // Перебираем все ликвидные акции
        for (index, share) in liquid_shares.iter().enumerate() {
            match self
                .process_instrument(
                    &share.instrument_id,
                    share.first_1min_candle_date,
                    index,
                    liquid_shares.len(),
                )
                .await
            {
                Ok(candles) => {
                    processed_count += 1;
                    debug!(
                        "Successfully processed instrument {}/{}: {} with {} candles",
                        index + 1,
                        liquid_shares.len(),
                        share.instrument_id,
                        candles
                    );
                }
                Err(e) => {
                    error!(
                        "Error processing instrument {}/{}: {}: {}",
                        index + 1,
                        liquid_shares.len(),
                        share.instrument_id,
                        e
                    );
                    // Продолжаем с следующим инструментом
                }
            }

            // Добавляем небольшую задержку между инструментами, чтобы не перегружать API
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }

        info!(
            "Completed processing {} out of {} liquid shares",
            processed_count,
            liquid_shares.len()
        );

        Ok(processed_count)
    }
}

/// Принимает время в секундах (Unix timestamp) и возвращает
/// время 23:59:59 того же дня в секундах (Unix timestamp)
///
/// # Arguments
/// * `timestamp_seconds` - Unix timestamp в секундах
///
/// # Returns
/// Unix timestamp в секундах для конца дня (23:59:59)
fn get_end_of_day(timestamp_seconds: i64) -> i64 {
    // Преобразуем Unix timestamp в DateTime
    let datetime = DateTime::from_timestamp(timestamp_seconds, 0).unwrap_or_else(|| Utc::now());

    // Получаем год, месяц и день из входного времени
    let year = datetime.year();
    let month = datetime.month();
    let day = datetime.day();

    // Создаем новую дату с временем 23:59:59
    let end_of_day = Utc
        .with_ymd_and_hms(year, month, day, 23, 59, 59)
        .single()
        .unwrap_or_default();

    // Возвращаем новое время в формате Unix timestamp (секунды)
    end_of_day.timestamp()
}

/// Принимает время в секундах (Unix timestamp) и возвращает
/// два значения в секундах (Unix timestamp):
/// 1. Начало следующего дня (00:00:00)
/// 2. Конец следующего дня (23:59:59)
///
/// # Arguments
/// * `timestamp_seconds` - Unix timestamp в секундах
///
/// # Returns
/// Кортеж из двух Unix timestamp в секундах (начало и конец следующего дня)
fn get_next_day_range(timestamp_seconds: i64) -> (i64, i64) {
    // Преобразуем Unix timestamp в DateTime
    let datetime = DateTime::from_timestamp(timestamp_seconds, 0).unwrap_or_else(|| Utc::now());

    // Добавляем 1 день
    let next_day = datetime + Duration::days(1);

    // Получаем год, месяц и день следующего дня
    let year = next_day.year();
    let month = next_day.month();
    let day = next_day.day();

    // Создаем временную метку на начало следующего дня (00:00:00)
    let start_of_next_day = Utc
        .with_ymd_and_hms(year, month, day, 0, 0, 0)
        .single()
        .unwrap_or_default();

    // Создаем временную метку на конец следующего дня (23:59:59)
    let end_of_next_day = Utc
        .with_ymd_and_hms(year, month, day, 23, 59, 59)
        .single()
        .unwrap_or_default();

    // Возвращаем обе временные метки в формате Unix timestamp (секунды)
    (start_of_next_day.timestamp(), end_of_next_day.timestamp())
}
