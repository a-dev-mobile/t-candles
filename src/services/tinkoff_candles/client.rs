use crate::app_state::models::AppState;
use crate::db::models::candle::DbCandle;
use crate::db::models::share;
use crate::generate::tinkoff_public_invest_api_contract_v1::{
    CandleInterval, GetCandlesRequest, HistoricCandle,
};
use chrono::{DateTime, Datelike, Duration, NaiveDateTime, TimeZone, Utc};

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

    /// Преобразование HistoricCandle из API в модель Candle для БД
    fn convert_candles(&self, figi: &str, api_candles: Vec<HistoricCandle>) -> Vec<DbCandle> {
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
    pub async fn load_and_save_candles(&self) -> Result<usize, Box<dyn std::error::Error>> {
        let liquid_shares = self
            .app_state
            .db_service
            .share_repository
            .get_liquid_shares()
            .await?;

        let share = liquid_shares.first().unwrap();


// let endTime = self.app_state.db_service.candle_repository.get_last_candle_time(&share.figi).await?;



        // // Получаем свечи из API
        // let api_candles = self
        //     .get_minute_candles(
        //         &share.instrument_id,
        //         share.first_1min_candle_date,
        //         get_end_of_day(share.first_1min_candle_date),
        //     )
        //     .await?;

        // if api_candles.is_empty() {
        //     debug!("No candles received for {} from {} to {}", figi, from, to);
        //     return Ok(0);
        // }

        // // Преобразуем в модель для БД
        // let db_candles = self.convert_candles(figi, api_candles);

        // if db_candles.is_empty() {
        //     debug!(
        //         "No valid candles to save for {} from {} to {}",
        //         figi, from, to
        //     );
        //     return Ok(0);
        // }

        // // Сохраняем в БД
        // let inserted = self
        //     .app_state
        //     .db_service
        //     .candle_repository
        //     .insert_candles(&db_candles)
        //     .await?;

        // info!(
        //     "Saved {} candles for {} from {} to {}",
        //     db_candles.len(),
        //     figi,
        //     from,
        //     to
        // );

        // Ok(db_candles.len())
        Ok(0)
    }
}

/// Принимает время в секундах (Unix timestamp) и возвращает
/// время 23:59:59 того же дня в секундах (Unix timestamp)
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
