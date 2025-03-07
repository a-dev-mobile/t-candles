use crate::app_state::models::AppState;
use crate::db::models::candle::DbCandle;
use crate::generate::tinkoff_public_invest_api_contract_v1::{
    CandleInterval, GetCandlesRequest, HistoricCandle,
};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tracing::{debug, error, info};

/// Клиент для работы с API свечей Tinkoff
pub struct TinkoffCandleClient {
    app_state: Arc<AppState>,
}

impl TinkoffCandleClient {
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self { app_state }
    }

    /// Получение минутных свечей за указанный период
    pub async fn get_minute_candles(
        &self,
        figi: &str,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<HistoricCandle>, Box<dyn std::error::Error>> {
        info!(
            "Requesting minute candles for {} from {} to {}",
            figi, from, to
        );

        // Создаем запрос к Tinkoff API
        let request = GetCandlesRequest {
            from: Some(prost_types::Timestamp {
                seconds: from.timestamp(),
                nanos: 0,
            }),
            to: Some(prost_types::Timestamp {
                seconds: to.timestamp(),
                nanos: 0,
            }),
            instrument_id: figi.to_string(),
            figi: figi.to_string(),
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
            figi
        );

        Ok(candles_response.candles)
    }

    /// Преобразование HistoricCandle из API в модель Candle для БД
    pub fn convert_candles(&self, figi: &str, api_candles: Vec<HistoricCandle>) -> Vec<DbCandle> {
        let mut result = Vec::with_capacity(api_candles.len());

        for candle in api_candles {
            // Парсим временную метку
            if let Some(time_stamp) = candle.time {
                let seconds = time_stamp.seconds;
                if let Some(time) = DateTime::from_timestamp(seconds, 0) {
                    let db_candle = DbCandle {
                        figi: figi.to_string(),
                        time,
                        open_units: candle.open.as_ref().map_or(0, |q| q.units),
                        open_nano: candle.open.as_ref().map_or(0, |q| q.nano),
                        high_units: candle.high.as_ref().map_or(0, |q| q.units),
                        high_nano: candle.high.as_ref().map_or(0, |q| q.nano),
                        low_units: candle.low.as_ref().map_or(0, |q| q.units),
                        low_nano: candle.low.as_ref().map_or(0, |q| q.nano),
                        close_units: candle.close.as_ref().map_or(0, |q| q.units),
                        close_nano: candle.close.as_ref().map_or(0, |q| q.nano),
                        volume: candle.volume as u64,
                        is_complete: candle.is_complete,
                    };

                    result.push(db_candle);
                } else {
                    debug!("Failed to parse timestamp for candle: {}", seconds);
                }
            }
        }

        result
    }

    /// Загрузка и сохранение свечей в БД
    pub async fn load_and_save_candles(
        &self,
        figi: &str,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        // Получаем свечи из API
        let api_candles = self.get_minute_candles(figi, from, to).await?;

        if api_candles.is_empty() {
            debug!("No candles received for {} from {} to {}", figi, from, to);
            return Ok(0);
        }

        // Преобразуем в модель для БД
        let db_candles = self.convert_candles(figi, api_candles);

        if db_candles.is_empty() {
            debug!(
                "No valid candles to save for {} from {} to {}",
                figi, from, to
            );
            return Ok(0);
        }

        // Сохраняем в БД
        let inserted = self
            .app_state
            .db_service
            .candle_repository
            .insert_candles(&db_candles)
            .await?;

        info!(
            "Saved {} candles for {} from {} to {}",
            db_candles.len(),
            figi,
            from,
            to
        );

        Ok(db_candles.len())
    }


}
