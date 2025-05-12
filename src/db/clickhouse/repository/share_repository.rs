use super::helper;
use crate::{
    db::clickhouse::{connection::ClickhouseConnection, models::share::DbSharesLiquid},
    generate::tinkoff_public_invest_api_contract_v1::Share,
};

use clickhouse::error::Error as ClickhouseError;
use serde::Deserialize;
use std::collections::HashSet;
use std::sync::Arc;
use tracing::{debug, error, info};

pub struct ShareRepository {
    connection: Arc<ClickhouseConnection>,
    // Список FIGI, которые следует пропускать при обработке
    problematic_figis: HashSet<String>,
}

impl ShareRepository {
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
    pub async fn delete_all_shares(&self) -> Result<u64, ClickhouseError> {
        let client = self.connection.get_client();
        
        // Get the fully qualified table name
        let table_name = format!("{}.{}", self.connection.get_database(), "tinkoff_shares");
        
        // Create the delete query
        let sql = format!("TRUNCATE TABLE {}", table_name);
        
        info!("Truncating shares table before update");
        
        // Execute the query
        match client.query(&sql).execute().await {
            Ok(_) => {
                info!("Successfully cleared shares table for fresh update");
                Ok(1) // Return 1 to indicate success
            }
            Err(e) => {
                error!("Failed to clear shares table: {}", e);
                Err(e)
            }
        }
    }
    // Метод для проверки, является ли FIGI проблемным
    fn is_problematic_figi(&self, figi: &str) -> bool {
        self.problematic_figis.contains(figi)
    }

     pub async fn insert_shares(&self, shares: &[Share], clean_first: bool) -> Result<u64, ClickhouseError> {
        if shares.is_empty() {
            debug!("No shares to insert");
            return Ok(0);
        }

        // Optionally clean the table before inserting
        if clean_first {
            match self.delete_all_shares().await {
                Ok(_) => info!("Preparing for fresh data insert"),
                Err(e) => {
                    error!("Failed to clean shares table before insert: {}", e);
                    // Continue with insert even if cleanup fails
                }
            }
        }

        let client = self.connection.get_client();
        let total_count = shares.len();

        info!("Starting insertion of {} shares", total_count);

        // Фильтруем проблемные FIGI при создании списка
        let filtered_shares: Vec<&Share> = shares
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

        let filtered_count = filtered_shares.len();
        if filtered_count == 0 {
            info!("No valid shares to insert after filtering");
            return Ok(0);
        }

        // Формируем части VALUES для SQL запроса вставки
        let mut values_parts = Vec::with_capacity(filtered_count);
        for share in &filtered_shares {
            debug!(
                "Preparing share: FIGI={}, Name='{}', Ticker='{}'",
                share.figi, share.name, share.ticker
            );
            values_parts.push(helper::format_share_values(share));
        }
        // Получаем полное имя таблицы с использованием схемы из конфигурации
        let table_name = format!("{}.{}", self.connection.get_database(), "tinkoff_shares");

        // Формируем полный SQL-запрос для вставки
        let sql = format!(
            "INSERT INTO {} ({}) VALUES {}",
            table_name,
            helper::get_insert_columns(),
            values_parts.join(",")
        );

        // Выполняем вставку
        match client.query(&sql).execute().await {
            Ok(_) => {
                info!(
                    "Successfully inserted {} shares (out of {} total)",
                    filtered_count, total_count
                );
                Ok(filtered_count as u64)
            }
            Err(e) => {
                error!("Insertion failed: {}", e);
                Err(e)
            }
        }
    }

    pub async fn get_liquid_shares(&self) -> Result<Vec<DbSharesLiquid>, ClickhouseError> {
        let client = self.connection.get_client();
        // Получаем полное имя таблицы с использованием схемы из конфигурации
        let table_name = format!("{}.{}", self.connection.get_database(), "tinkoff_shares");

        // SQL запрос для получения ликвидных акций с использованием таблицы с учетом схемы
        let query = format!(
            "
        SELECT uid, first_1min_candle_date
        FROM {}
        WHERE buy_available_flag = 1
          AND sell_available_flag = 1
          AND first_1min_candle_date IS NOT NULL
    ",
            table_name
        );

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
            .query(&query)
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
