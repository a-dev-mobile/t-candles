use crate::generate::tinkoff_public_invest_api_contract_v1::Share;

// Функция безопасного экранирования строк для SQL запросов
pub(crate) fn escape_string_max(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('?', "\\?")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
        .replace('\0', "\\0")
}

// Helper function to get SQL insert statement columns
pub(crate) fn get_insert_columns() -> &'static str {
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

// Helper function to format a share into an SQL VALUES clause
pub(crate) fn format_share_values(share: &Share) -> String {
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
    
    // Timestamp -> Int64 (хранение timestamp в секундах)
    fn timestamp_to_sql(ts: &Option<prost_types::Timestamp>) -> String {
        match ts {
            Some(ts) => {
                // Возвращаем timestamp в секундах
                ts.seconds.to_string()
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