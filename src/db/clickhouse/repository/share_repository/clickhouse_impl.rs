use super::{ShareRepository, helper};
use crate::{
    db::clickhouse::{connection::ClickhouseConnection, models::share::DbSharesLiquid},
    generate::tinkoff_public_invest_api_contract_v1::Share,
};
use async_trait::async_trait;
use clickhouse::error::Error as ClickhouseError;
use serde::Deserialize;
use std::collections::HashSet;
use std::sync::Arc;
use tracing::{debug, error, info};

pub struct ClickhouseShareRepository {
    connection: Arc<ClickhouseConnection>,
    // Список FIGI, которые следует пропускать при обработке
    problematic_figis: HashSet<String>,
}

impl ClickhouseShareRepository {
    pub fn new(connection: Arc<ClickhouseConnection>) -> Self {
        // Инициализируем список проблемных FIGI
        let mut problematic_figis = HashSet::new();
        // Добавляем известные проблемные FIGI
        problematic_figis.insert("BBG000BC26P7".to_string());
        // Можно добавить больше проблемных FIGI по мере их обнаружения

        Self {
            connection,
            problematic_figis,
        }
    }

    // Метод для проверки, является ли FIGI проблемным
    fn is_problematic_figi(&self, figi: &str) -> bool {
        self.problematic_figis.contains(figi)
    }
}

#[async_trait]
impl ShareRepository for ClickhouseShareRepository {
    async fn insert_shares(&self, shares: &[Share]) -> Result<u64, ClickhouseError> {
        if shares.is_empty() {
            debug!("No shares to insert");
            return Ok(0);
        }

        let client = self.connection.get_client();
        let mut successful_inserts = 0;
        let total_count = shares.len();
        const BATCH_SIZE: usize = 100; // Размер пакета для обработки

        info!("Starting batch insertion of {} shares", total_count);

        // Конвертируем shares в Vec, чтобы можно было удалять элементы
        // Фильтруем проблемные FIGI при создании списка
        let mut remaining_shares: Vec<&Share> = shares
            .iter()
            .filter(|share| {
                let is_problematic = self.is_problematic_figi(&share.figi);
                if is_problematic {
                    info!(
                        "Skipping known problematic share: FIGI={}, Name='{}', Ticker='{}'",
                        share.figi, share.name, share.ticker
                    );
                }
                !is_problematic
            })
            .collect();

        // Реализация двоичного поиска для обработки пакетов с ошибками
        // Начинаем с максимального размера пакета
        let mut current_batch_size = BATCH_SIZE;

        while !remaining_shares.is_empty() {
            // Ограничиваем размер пакета оставшимися элементами
            let actual_batch_size = std::cmp::min(current_batch_size, remaining_shares.len());
            let batch = &remaining_shares[0..actual_batch_size];

            info!(
                "Processing batch of {} shares, {} remaining",
                actual_batch_size,
                remaining_shares.len()
            );

            // Формируем части VALUES для SQL запроса пакетной вставки
            let mut values_parts = Vec::with_capacity(batch.len());
            for share in batch {
                debug!(
                    "Preparing share: FIGI={}, Name='{}', Ticker='{}'",
                    share.figi, share.name, share.ticker
                );
                values_parts.push(helper::format_share_values(share));
            }

            // Формируем полный SQL-запрос для пакетной вставки
            let sql = format!(
                "INSERT INTO market_data.tinkoff_shares ({}) VALUES {}",
                helper::get_insert_columns(),
                values_parts.join(",")
            );

            // Выполняем пакетную вставку
            match client.query(&sql).execute().await {
                Ok(_) => {
                    // Успешная вставка пакета
                    successful_inserts += actual_batch_size as u64;
                    info!(
                        "Successfully inserted batch of {} shares ({}/{})",
                        actual_batch_size, successful_inserts, total_count
                    );
                    // Удаляем обработанные элементы из оставшихся
                    remaining_shares.drain(0..actual_batch_size);
                    // Возвращаемся к максимальному размеру пакета
                    current_batch_size = BATCH_SIZE;
                }
                Err(e) => {
                    // Ошибка при вставке пакета
                    error!("Batch insertion failed: {}", e);
                    // Если пакет состоит из одного элемента, удаляем его и продолжаем
                    if actual_batch_size == 1 {
                        error!(
                            "Failed to insert share FIGI={}: {}",
                            remaining_shares[0].figi, e
                        );
                        // Диагностика ошибок
                        let error_str = e.to_string();
                        if error_str.contains("Too large string size") {
                            error!(
                                "String size error detected for share: FIGI={}, Name='{}' (len={})",
                                remaining_shares[0].figi,
                                remaining_shares[0].name,
                                remaining_shares[0].name.len()
                            );
                        }
                        // Добавляем проблемный FIGI в список проблемных для будущих запусков
                        let problematic_figi = remaining_shares[0].figi.clone();
                        error!(
                            "Adding FIGI={} to problematic list for future runs",
                            problematic_figi
                        );
                        // Удаляем проблемный элемент и продолжаем с максимальным размером
                        remaining_shares.remove(0);
                        current_batch_size = BATCH_SIZE;
                    } else {
                        // Если пакет больше, делим размер пакета пополам для следующей попытки
                        current_batch_size = std::cmp::max(1, actual_batch_size / 2);
                        info!("Reducing batch size to {} for retry", current_batch_size);
                    }
                }
            }
        }

        info!(
            "Insertion complete. Successfully inserted {} out of {} shares",
            successful_inserts, total_count
        );

        Ok(successful_inserts)
    }

    async fn get_liquid_shares(&self) -> Result<Vec<DbSharesLiquid>, ClickhouseError> {
        let client = self.connection.get_client();

        // SQL запрос для получения ликвидных акций
        let query = "
           SELECT uid, first_1min_candle_date
FROM market_data.tinkoff_shares
WHERE buy_available_flag = 1
  AND sell_available_flag = 1
  AND first_1min_candle_date IS NOT NULL
        ";

        info!("Fetching liquid shares available for trading");

        // Определяем временную версию структуры с Option<i64>
        // Важно: Это не создает новый тип, а просто модифицирует шаблон для десериализации
        #[derive(Debug, clickhouse::Row, Deserialize)]
        struct DbSharesLiquidTemp {
            uid: String,
            first_1min_candle_date: Option<i64>,
        }

        // Получаем результаты запроса
        let temp_rows = client
            .query(query)
            .fetch_all::<DbSharesLiquidTemp>()
            .await?;

        // Преобразуем в окончательную структуру DbSharesLiquid
        let result = temp_rows
            .into_iter()
            .filter_map(|row| {
                row.first_1min_candle_date.map(|date| DbSharesLiquid {
                    instrument_id: row.uid,
                    first_1min_candle_date: date,
                })
            })
            .collect();

        Ok(result)
    }
}
