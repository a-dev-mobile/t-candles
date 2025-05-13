use super::helper;
use crate::{
    db::clickhouse::{connection::ClickhouseConnection, models::share::DbSharesLiquid},
    generate::tinkoff_public_invest_api_contract_v1::Share,
};

use clickhouse::error::Error as ClickhouseError;
use serde::Deserialize;
use std::collections::HashSet;
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use chrono::{TimeZone, Utc, FixedOffset};

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
                
                // After inserting shares, now update the liquid_shares table
                self.update_liquid_shares().await?;
                
                Ok(filtered_count as u64)
            }
            Err(e) => {
                error!("Insertion failed: {}", e);
                Err(e)
            }
        }
    }

    // Updated method to match the new table structure
    async fn update_liquid_shares(&self) -> Result<u64, ClickhouseError> {
        let client = self.connection.get_client();
        let database = self.connection.get_database();
        
        // Get current timestamp for update_time field - using Moscow time zone (UTC+3)
        let moscow_timezone = FixedOffset::east_opt(3 * 3600).unwrap(); // UTC+3 for Moscow
        let current_time = Utc::now()
            .with_timezone(&moscow_timezone)
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();
        
        // 1. Insert currently liquid shares with is_liquid_now=true
        let insert_query = format!(
            r#"
            INSERT INTO {0}.liquid_shares (uid, update_time, is_liquid_now)
            SELECT 
                uid,
                '{1}',
                true
            FROM {0}.tinkoff_shares
            WHERE buy_available_flag = 1
              AND sell_available_flag = 1
              AND first_1min_candle_date IS NOT NULL
            "#,
            database, current_time
        );
        
        info!("Inserting liquid shares (ReplacingMergeTree engine will handle duplicates)");
        
        match client.query(&insert_query).execute().await {
            Ok(_) => {
                info!("Successfully inserted currently liquid shares");
                
                // 2. Get a list of UIDs that are currently in liquid_shares table but are not in current liquid shares
                let get_old_uids_query = format!(
                    r#"
                    SELECT uid 
                    FROM {0}.liquid_shares
                    WHERE uid NOT IN (
                        SELECT uid
                        FROM {0}.tinkoff_shares
                        WHERE buy_available_flag = 1
                          AND sell_available_flag = 1
                          AND first_1min_candle_date IS NOT NULL
                    )
                    "#,
                    database
                );
                
                info!("Finding shares that are no longer liquid");
                
                // Execute the query to get a list of UIDs that need to be updated to is_liquid_now=false
                #[derive(Debug, serde::Deserialize, clickhouse::Row)]
                struct UidRecord {
                    uid: String,
                }
                
                let old_uids = match client.query(&get_old_uids_query).fetch_all::<UidRecord>().await {
                    Ok(uids) => {
                        info!("Found {} shares that are no longer liquid", uids.len());
                        uids
                    }
                    Err(e) => {
                        error!("Failed to find shares that are no longer liquid: {}", e);
                        return Err(e);
                    }
                };
                
                // 3. For each UID that is no longer liquid, insert a record with is_liquid_now=false
                for uid_record in old_uids {
                    let update_query = format!(
                        r#"
                        INSERT INTO {0}.liquid_shares (uid, update_time, is_liquid_now)
                        VALUES ('{1}', '{2}', false)
                        "#,
                        database, uid_record.uid, current_time
                    );
                    
                    match client.query(&update_query).execute().await {
                        Ok(_) => {
                            debug!("Updated share {} to is_liquid_now=false", uid_record.uid);
                        }
                        Err(e) => {
                            error!("Failed to update share {} to is_liquid_now=false: {}", uid_record.uid, e);
                            // Continue with other UIDs instead of failing the entire process
                        }
                    }
                }
                
                info!("Completed updating shares that are no longer liquid");
                
                // 4. Force a merge to ensure the latest versions are visible
                let optimize_query = format!("OPTIMIZE TABLE {0}.liquid_shares FINAL", database);
                info!("Optimizing table to ensure latest versions are visible");
                
                match client.query(&optimize_query).execute().await {
                    Ok(_) => info!("Successfully optimized liquid_shares table"),
                    Err(e) => warn!("Failed to optimize table: {}", e)
                }
                
                Ok(1) // Return 1 to indicate success
            }
            Err(e) => {
                error!("Failed to insert liquid shares: {}", e);
                Err(e)
            }
        }
    }

    pub async fn get_liquid_shares(&self) -> Result<Vec<DbSharesLiquid>, ClickhouseError> {
        let client = self.connection.get_client();
        // Получаем полное имя таблицы с использованием схемы из конфигурации
        let database = self.connection.get_database();

        // SQL запрос для получения ликвидных акций с таблицы liquid_shares
        let query = format!(
            "
            SELECT 
                uid as instrument_id,
                toUnixTimestamp(update_time) as update_timestamp
            FROM {}.liquid_shares
            WHERE is_liquid_now = true
            ",
            database
        );

        info!("Fetching liquid shares from liquid_shares table");

        // Since we're no longer using first_1min_candle_date, we need to update the DbSharesLiquid
        // structure or modify how we process the query results
        #[derive(Debug, clickhouse::Row, Deserialize)]
        struct DbSharesLiquidTemp {
            instrument_id: String,
            update_timestamp: i64,
        }

        // Получаем результаты запроса
        let temp_rows = client
            .query(&query)
            .fetch_all::<DbSharesLiquidTemp>()
            .await?;

        // Convert to DbSharesLiquid format - assuming we're repurposing first_1min_candle_date 
        // to store the update timestamp instead
        let result = temp_rows
            .into_iter()
            .map(|row| DbSharesLiquid {
                instrument_id: row.instrument_id,
                first_1min_candle_date: row.update_timestamp,
            })
            .collect();

        Ok(result)
    }
}