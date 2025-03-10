use crate::db::clickhouse::connection::ClickhouseConnection;

use crate::db::clickhouse::models::candle::{DailyCandle, DbCandle};
use crate::generate::tinkoff_public_invest_api_contract_v1::HistoricCandle;

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use clickhouse::error::Error as ClickhouseError;
use clickhouse::{Row, error, insert};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info};

#[async_trait]
pub trait CandleRepository {
    /// Вставка исторических свечей в ClickHouse
    ///
    /// # Параметры
    /// * `candles` - Вектор свечей для вставки
    /// * `instrument_uid` - Идентификатор инструмента
    ///
    /// # Возвращает
    /// * `Result<u64, ClickhouseError>` - Количество успешно вставленных свечей или ошибку
    async fn insert_candles(
        &self,
        candles: Vec<HistoricCandle>,
        instrument_uid: &str,
    ) -> Result<u64, ClickhouseError>;
}

pub struct ClickhouseCandleRepository {
    connection: Arc<ClickhouseConnection>,
}

impl ClickhouseCandleRepository {
    pub fn new(connection: Arc<ClickhouseConnection>) -> Self {
        Self { connection }
    }
}

#[async_trait]
impl CandleRepository for ClickhouseCandleRepository {
    async fn insert_candles(
        &self,
        candles: Vec<HistoricCandle>,
        instrument_uid: &str,
    ) -> Result<u64, ClickhouseError> {
        if candles.is_empty() {
            info!("No candles to insert");
            return Ok(0);
        }

        let client = self.connection.get_client();
        const BATCH_SIZE: usize = 1000; // Оптимальный размер пакета для вставки
        let total_count = candles.len();
        let mut successful_inserts = 0;
        info!(
            "Starting batch insertion of {} candles for instrument_uid={}",
            total_count, instrument_uid
        );

        // Конвертируем candles в Vec, чтобы можно было удалять элементы
        // Фильтруем свечи с недопустимыми значениями времени
        let mut remaining_candles: Vec<&HistoricCandle> = candles
            .iter()
            .filter(|candle| candle.time.is_some())
            .collect();

        // Реализация двоичного поиска для обработки пакетов с ошибками
        // Начинаем с максимального размера пакета
        let mut current_batch_size = BATCH_SIZE;

        while !remaining_candles.is_empty() {
            // Ограничиваем размер пакета оставшимися элементами
            let actual_batch_size = std::cmp::min(current_batch_size, remaining_candles.len());
            let batch = &remaining_candles[0..actual_batch_size];
            debug!(
                "Processing batch of {} candles, {} remaining",
                actual_batch_size,
                remaining_candles.len()
            );

            // Формируем части VALUES для SQL запроса пакетной вставки
            let mut values_parts = Vec::with_capacity(batch.len());
            for candle in batch {
                // Извлечение timestamp из prost_types::Timestamp
                let timestamp = candle.time.as_ref().unwrap().seconds; // Safe unwrap (filtered above)

                // Извлечение значений из Quotation (open, high, low, close)
                let open_units = candle.open.as_ref().map_or(0, |q| q.units);
                let open_nano = candle.open.as_ref().map_or(0, |q| q.nano);

                let high_units = candle.high.as_ref().map_or(0, |q| q.units);
                let high_nano = candle.high.as_ref().map_or(0, |q| q.nano);

                let low_units = candle.low.as_ref().map_or(0, |q| q.units);
                let low_nano = candle.low.as_ref().map_or(0, |q| q.nano);

                let close_units = candle.close.as_ref().map_or(0, |q| q.units);
                let close_nano = candle.close.as_ref().map_or(0, |q| q.nano);

                  // Используем volume как есть, поскольку теперь тип таблицы совпадает с типом данных
            let volume = candle.volume;

                values_parts.push(format!(
                    "('{}', {}, {}, {}, {}, {}, {}, {}, {}, {}, {})",
                    instrument_uid,
                    timestamp,
                    open_units,
                    open_nano,
                    high_units,
                    high_nano,
                    low_units,
                    low_nano,
                    close_units,
                    close_nano,
                    volume
                ));
            }

            // Формируем полный SQL-запрос для пакетной вставки с обновлением дубликатов
            let sql = format!(
                "INSERT INTO market_data.tinkoff_candles_1min 
                (instrument_uid, time, open_units, open_nano, high_units, high_nano, 
                low_units, low_nano, close_units, close_nano, volume) 
                VALUES {}
                ON DUPLICATE KEY UPDATE 
                open_units = VALUES(open_units),
                open_nano = VALUES(open_nano),
                high_units = VALUES(high_units),
                high_nano = VALUES(high_nano),
                low_units = VALUES(low_units),
                low_nano = VALUES(low_nano),
                close_units = VALUES(close_units),
                close_nano = VALUES(close_nano),
                volume = VALUES(volume)",
                values_parts.join(",")
            );

            // Выполняем пакетную вставку или обновление
            match client.query(&sql).execute().await {
                Ok(_) => {
                    // Успешная вставка/обновление пакета
                    successful_inserts += actual_batch_size as u64;
                    debug!(
                        "Successfully inserted/updated batch of {} candles ({}/{})",
                        actual_batch_size, successful_inserts, total_count
                    );
                    // Удаляем обработанные элементы из оставшихся
                    remaining_candles.drain(0..actual_batch_size);
                    // Возвращаемся к максимальному размеру пакета
                    current_batch_size = BATCH_SIZE;
                }
                Err(e) => {
                    // Ошибка при вставке пакета
                    error!("Batch insertion failed: {}", e);
                    // Если пакет состоит из одного элемента, удаляем его и продолжаем
                    if actual_batch_size == 1 {
                        error!(
                            "Failed to insert candle at time={}: {}",
                            remaining_candles[0].time.as_ref().unwrap().seconds,
                            e
                        );
                        // Удаляем проблемный элемент и продолжаем с максимальным размером
                        remaining_candles.remove(0);
                        current_batch_size = BATCH_SIZE;
                    } else {
                        // Если пакет больше, делим размер пакета пополам для следующей попытки
                        current_batch_size = std::cmp::max(1, actual_batch_size / 2);
                        debug!("Reducing batch size to {} for retry", current_batch_size);
                    }
                }
            }
        }

        info!(
            "Insertion complete. Successfully inserted/updated {} out of {} candles for instrument_uid={}",
            successful_inserts, total_count, instrument_uid
        );

        Ok(successful_inserts)
    }
}
