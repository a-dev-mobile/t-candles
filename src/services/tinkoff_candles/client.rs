use crate::app_state::models::AppState;

use crate::db::clickhouse::models::candle::DbCandle;
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

    /// Загрузка и сохранение свечей в БД
    pub async fn load_and_save_candles(&self) -> Result<usize, Box<dyn std::error::Error>> {
        let liquid_shares = self
            .app_state
            .clickhouse_service
            .share_repository
            .get_liquid_shares()
            .await?;

        let share = &liquid_shares[1];

        let status_candle = self
            .app_state
            .postgres_service
            .repository_tinkoff_candles_status
            .get_by_instrument_uid(&share.instrument_id)
            .await?;

        let is_empty_status = status_candle.is_none();

        if is_empty_status {
            let end_of_day = get_end_of_day(share.first_1min_candle_date);
            // Получаем свечи из API
            let vec_candles: Vec<HistoricCandle> = self
                .get_minute_candles(
                    &share.instrument_id,
                    share.first_1min_candle_date,
                    end_of_day,
                )
                .await?;

            self.app_state
                .clickhouse_service
                .repository_candle
                .insert_candles(vec_candles, &share.instrument_id)
                .await?;

            self.app_state
                .postgres_service
                .repository_tinkoff_candles_status
                .upsert(&share.instrument_id, end_of_day)
                .await?;
        } else {
            let to_second = status_candle.unwrap().to_second;
            let (next_day_start, next_day_end) = get_next_day_range(to_second);
            let vec_candles: Vec<HistoricCandle> = self
                .get_minute_candles(&share.instrument_id, next_day_start, next_day_end)
                .await?;

            self.app_state
                .clickhouse_service
                .repository_candle
                .insert_candles(vec_candles, &share.instrument_id)
                .await?;

            self.app_state
                .postgres_service
                .repository_tinkoff_candles_status
                .update_to_second(&share.instrument_id, next_day_end)
                .await?;
        }

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
