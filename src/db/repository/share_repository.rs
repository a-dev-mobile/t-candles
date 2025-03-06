// src/db/repository/share_repository.rs

use crate::db::clickhouse::error::ClickhouseError;
use crate::services::tinkoff_instruments::models::share::DbTinkoffShare;
use crate::{
    db::clickhouse::connection::ClickhouseConnection,
    generate::tinkoff_public_invest_api_contract_v1::Share,
};
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{debug, error, info};

#[async_trait]
pub trait ShareRepository {
    /// Insert multiple shares into the database
    async fn insert_shares(&self, shares: &[Share]) -> Result<u64, ClickhouseError>;
}

pub struct ClickhouseShareRepository {
    connection: Arc<ClickhouseConnection>,
}

impl ClickhouseShareRepository {
    pub fn new(connection: Arc<ClickhouseConnection>) -> Self {
        Self { connection }
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

        info!("Starting insertion of {} shares", total_count);

        for (index, share) in shares.iter().enumerate() {
            // Прямое преобразование из proto в SQL-значения

            // Quotation -> (units, nano)
            fn quotation_units(
                quotation: &Option<
                    crate::generate::tinkoff_public_invest_api_contract_v1::Quotation,
                >,
            ) -> String {
                match quotation {
                    Some(q) => q.units.to_string(),
                    None => "NULL".to_string(),
                }
            }

            fn quotation_nano(
                quotation: &Option<
                    crate::generate::tinkoff_public_invest_api_contract_v1::Quotation,
                >,
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

            // Формируем SQL-запрос, напрямую преобразуя поля proto в значения SQL
            let sql = format!(
                "INSERT INTO market_data.tinkoff_shares (
                    figi, ticker, class_code, isin, lot, currency, 
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
                    first_1day_candle_date
                ) VALUES (
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
                // Прямое преобразование Quotation полей
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
                // Булевы поля
                if share.short_enabled_flag { 1 } else { 0 },
                escape_string(&share.name),
                (&share.exchange),
                // Опциональная дата
                timestamp_to_sql(&share.ipo_date),
                share.issue_size,
                (&share.country_of_risk),
                (&share.country_of_risk_name),
                (&share.sector),
                share.issue_size_plan,
                // MoneyValue
                nominal_currency,
                nominal_units,
                nominal_nano,
                // Enum сохраняем как Int32 (без преобразования в строку)
                share.trading_status,
                if share.otc_flag { 1 } else { 0 },
                if share.buy_available_flag { 1 } else { 0 },
                if share.sell_available_flag { 1 } else { 0 },
                if share.div_yield_flag { 1 } else { 0 },
                // Enum сохраняем как Int32 (без преобразования в строку)
                share.share_type,
                // Опциональное Quotation
                quotation_units(&share.min_price_increment),
                quotation_nano(&share.min_price_increment),
                if share.api_trade_available_flag { 1 } else { 0 },
                (&share.uid),
                // Enum сохраняем как Int32 (без преобразования в строку)
                share.real_exchange,
                (&share.position_uid),
                if share.for_iis_flag { 1 } else { 0 },
                if share.for_qual_investor_flag { 1 } else { 0 },
                if share.weekend_flag { 1 } else { 0 },
                if share.blocked_tca_flag { 1 } else { 0 },
                if share.liquidity_flag { 1 } else { 0 },
                // Опциональные даты
                timestamp_to_sql(&share.first_1min_candle_date),
                timestamp_to_sql(&share.first_1day_candle_date)
            );

            // Логгируем информацию перед вставкой
            debug!(
                "Inserting share #{}/{}: FIGI={}, Name='{}', Ticker='{}'",
                index + 1,
                total_count,
                share.figi,
                share.name,
                share.ticker
            );

            // Выполняем запрос
            match client.query(&sql).execute().await {
                Ok(_) => {
                    successful_inserts += 1;

                    if successful_inserts % 100 == 0 || successful_inserts == 1 {
                        info!(
                            "Successfully inserted {}/{} shares",
                            successful_inserts, total_count
                        );
                    }
                }
                Err(e) => {
                    error!(
                        "Failed to insert share #{} (FIGI: {}): {}",
                        index + 1,
                        share.figi,
                        e
                    );

                    // Диагностика ошибок
                    let error_str = e.to_string();
                    if error_str.contains("Too large string size") {
                        error!(
                            "String size error detected for share: FIGI={}, Name='{}' (len={})",
                            share.figi,
                            share.name,
                            share.name.len()
                        );
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
// Функция безопасного экранирования строк
fn escape_string(s: &str) -> String {
    s.replace('\'', "''")
}
