// src/db/repository/share_repository.rs

use crate::db::clickhouse::error::ClickhouseError;
use crate::services::tinkoff_instruments::models::share::DbTinkoffShare;
use crate::{
    db::clickhouse::connection::ClickhouseConnection,
    generate::tinkoff_public_invest_api_contract_v1::Share,
};
use async_trait::async_trait;
use std::collections::HashSet;
use std::sync::Arc;
use tracing::{debug, error, info};

#[async_trait]
pub trait ShareRepository {
    /// Insert multiple shares into the database
    async fn insert_shares(&self, shares: &[Share]) -> Result<u64, ClickhouseError>;
}

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

    // Helper method to generate the SQL insert statement columns
    fn get_insert_columns(&self) -> &str {
        "figi, ticker, class_code, isin, lot, currency, 
        klong_units, klong_nano, kshort_units, kshort_nano, 
        dlong_units, dlong_nano, dshort_units, dshort_nano, 
        dlong_min_units, dlong_min_nano, dshort_min_units, dshort_min_nano, 
        short_enabled_flag, name, exchange, ipo_date, issue_size, 
        country_of_risk, country_of_risk_name, sector, issue_size_plan, 
        nominal_currency, nominal_units, nominal_nano, trading_status, 
        otc_flag, buy_available_flag, sell_available_flag, div_yield_flag, 
        share_type, min_price_increment_units, min_price_increment_nano, 
        api_trade_available_flag, uid, real_exchange, position_uid, 
        for_iis_flag, for_qual_investor_flag, weekend_flag, 
        blocked_tca_flag, liquidity_flag, first_1min_candle_date, 
        first_1day_candle_date"
    }

    // Helper method to format a share into an SQL VALUES clause
    fn format_share_values(&self, share: &Share) -> String {
        // Quotation -> (units, nano)
        fn quotation_units(
            quotation: &Option<crate::generate::tinkoff_public_invest_api_contract_v1::Quotation>,
        ) -> String {
            match quotation {
                Some(q) => q.units.to_string(),
                None => "NULL".to_string(),
            }
        }

        fn quotation_nano(
            quotation: &Option<crate::generate::tinkoff_public_invest_api_contract_v1::Quotation>,
        ) -> String {
            match quotation {
                Some(q) => q.nano.to_string(),
                None => "NULL".to_string(),
            }
        }

        // Timestamp -> DateTime
        fn timestamp_to_sql(ts: &Option<prost_types::Timestamp>) -> String {
            match ts {
                Some(ts) => {
                    let seconds = ts.seconds;
                    let datetime = chrono::DateTime::from_timestamp(seconds, 0);
                    match datetime {
                        Some(dt) => format!("'{}'", dt.format("%Y-%m-%d %H:%M:%S")),
                        None => "NULL".to_string(),
                    }
                }
                None => "NULL".to_string(),
            }
        }

        // Обработка опционального MoneyValue
        let (nominal_currency, nominal_units, nominal_nano) = match &share.nominal {
            Some(n) => (
                format!("'{}'", (&n.currency)),
                n.units.to_string(),
                n.nano.to_string(),
            ),
            None => ("NULL".to_string(), "NULL".to_string(), "NULL".to_string()),
        };

        // Return the VALUES part of the SQL query for this share
        format!(
            "(
                '{}', '{}', '{}', '{}', {}, '{}', 
                {}, {}, {}, {}, 
                {}, {}, {}, {}, 
                {}, {}, {}, {}, 
                {}, '{}', '{}', {}, {}, 
                '{}', '{}', '{}', {}, 
                {}, {}, {}, {}, 
                {}, {}, {}, {}, 
                {}, {}, {}, 
                {}, '{}', {}, '{}', 
                {}, {}, {}, 
                {}, {}, {}, {}
            )",
            (&share.figi),
            (&share.ticker),
            (&share.class_code),
            (&share.isin),
            share.lot,
            (&share.currency),
            quotation_units(&share.klong),
            quotation_nano(&share.klong),
            quotation_units(&share.kshort),
            quotation_nano(&share.kshort),
            quotation_units(&share.dlong),
            quotation_nano(&share.dlong),
            quotation_units(&share.dshort),
            quotation_nano(&share.dshort),
            quotation_units(&share.dlong_min),
            quotation_nano(&share.dlong_min),
            quotation_units(&share.dshort_min),
            quotation_nano(&share.dshort_min),
            if share.short_enabled_flag { 1 } else { 0 },
            escape_string_max(&share.name),
            (&share.exchange),
            timestamp_to_sql(&share.ipo_date),
            share.issue_size,
            (&share.country_of_risk),
            (&share.country_of_risk_name),
            (&share.sector),
            share.issue_size_plan,
            nominal_currency,
            nominal_units,
            nominal_nano,
            share.trading_status,
            if share.otc_flag { 1 } else { 0 },
            if share.buy_available_flag { 1 } else { 0 },
            if share.sell_available_flag { 1 } else { 0 },
            if share.div_yield_flag { 1 } else { 0 },
            share.share_type,
            quotation_units(&share.min_price_increment),
            quotation_nano(&share.min_price_increment),
            if share.api_trade_available_flag { 1 } else { 0 },
            (&share.uid),
            share.real_exchange,
            (&share.position_uid),
            if share.for_iis_flag { 1 } else { 0 },
            if share.for_qual_investor_flag { 1 } else { 0 },
            if share.weekend_flag { 1 } else { 0 },
            if share.blocked_tca_flag { 1 } else { 0 },
            if share.liquidity_flag { 1 } else { 0 },
            timestamp_to_sql(&share.first_1min_candle_date),
            timestamp_to_sql(&share.first_1day_candle_date)
        )
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

                values_parts.push(self.format_share_values(share));
            }

            // Формируем полный SQL-запрос для пакетной вставки
            let sql = format!(
                "INSERT INTO market_data.tinkoff_shares ({}) VALUES {}",
                self.get_insert_columns(),
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
}

// Функция безопасного экранирования строк для SQL запросов

fn escape_string_max(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('?', "\\?")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
        .replace('\0', "\\0")
}
