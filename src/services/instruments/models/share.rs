use chrono::{DateTime, TimeZone, Utc};
use clickhouse::Row;
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::generate::tinkoff_public_invest_api_contract_v1::{
    RealExchange, SecurityTradingStatus, Share, ShareType,
};

#[derive(Debug, Clone, Serialize, Deserialize, Row)]
pub struct DbTinkoffShare {
    // Basic fields
    pub figi: String,
    pub ticker: String,
    pub class_code: String,
    pub isin: String,
    pub lot: u32,
    pub currency: String,

    // Quotation fields as separate components
    pub klong_units: i64,
    pub klong_nano: i32,
    pub kshort_units: i64,
    pub kshort_nano: i32,
    pub dlong_units: i64,
    pub dlong_nano: i32,
    pub dshort_units: i64,
    pub dshort_nano: i32,
    pub dlong_min_units: i64,
    pub dlong_min_nano: i32,
    pub dshort_min_units: i64,
    pub dshort_min_nano: i32,

    pub short_enabled_flag: bool,
    pub name: String,
    pub exchange: String,

    pub ipo_date: Option<DateTime<Utc>>,
    pub issue_size: i64,
    pub country_of_risk: String,
    pub country_of_risk_name: String,
    pub sector: String,
    pub issue_size_plan: i64,

    // Money value fields as separate components
    pub nominal_currency: String,
    pub nominal_units: i64,
    pub nominal_nano: i32,

    pub trading_status: String,
    pub otc_flag: bool,
    pub buy_available_flag: bool,
    pub sell_available_flag: bool,
    pub div_yield_flag: bool,
    pub share_type: String,

    // Min price increment fields
    pub min_price_increment_units: i64,
    pub min_price_increment_nano: i32,

    pub api_trade_available_flag: bool,
    pub uid: String,
    pub real_exchange: String,
    pub position_uid: String,
    pub for_iis_flag: bool,
    pub for_qual_investor_flag: bool,
    pub weekend_flag: bool,
    pub blocked_tca_flag: bool,
    pub liquidity_flag: bool,

    pub first_1min_candle_date: Option<DateTime<Utc>>,
    pub first_1day_candle_date: Option<DateTime<Utc>>,
}

impl From<&Share> for DbTinkoffShare {
    fn from(share: &Share) -> Self {
        // Function to safely convert a timestamp to DateTime<Utc>
        fn timestamp_to_datetime(ts: &Option<prost_types::Timestamp>) -> Option<DateTime<Utc>> {
            match ts {
                Some(ts) => {
                    let seconds = ts.seconds;
                    let nanos = ts.nanos as u32;

                    // Try standard conversion first
                    DateTime::from_timestamp(seconds, nanos).or_else(|| {
                        // Fall back to chrono's legacy conversion method
                        match Utc.timestamp_opt(seconds, nanos) {
                            chrono::offset::LocalResult::Single(dt) => Some(dt),
                            _ => {
                                warn!("Invalid timestamp: seconds={}, nanos={}", seconds, nanos);
                                None
                            }
                        }
                    })
                }
                None => None,
            }
        }

        // Helper function to extract quotation fields as integers
        fn extract_quotation(
            quotation: &Option<crate::generate::tinkoff_public_invest_api_contract_v1::Quotation>,
        ) -> (i64, i32) {
            quotation.as_ref().map_or((0, 0), |q| (q.units, q.nano))
        }

        // Extract quotation values using the helper function
        let (klong_units, klong_nano) = extract_quotation(&share.klong);
        let (kshort_units, kshort_nano) = extract_quotation(&share.kshort);
        let (dlong_units, dlong_nano) = extract_quotation(&share.dlong);
        let (dshort_units, dshort_nano) = extract_quotation(&share.dshort);
        let (dlong_min_units, dlong_min_nano) = extract_quotation(&share.dlong_min);
        let (dshort_min_units, dshort_min_nano) = extract_quotation(&share.dshort_min);
        let (min_price_units, min_price_nano) = extract_quotation(&share.min_price_increment);

        // Extract nominal values
        let (nominal_currency, nominal_units, nominal_nano) = match &share.nominal {
            Some(n) => (n.currency.clone(), n.units, n.nano),
            None => ("".to_string(), 0, 0),
        };

        // Use the as_str_name method from generated enum types
        let trading_status =
            if let Some(status) = SecurityTradingStatus::from_i32(share.trading_status) {
                status.as_str_name().to_string()
            } else {
                "UNKNOWN".to_string()
            };

        let share_type = if let Some(stype) = ShareType::from_i32(share.share_type) {
            stype.as_str_name().to_string()
        } else {
            "UNKNOWN".to_string()
        };

        let real_exchange = if let Some(exchange) = RealExchange::from_i32(share.real_exchange) {
            exchange.as_str_name().to_string()
        } else {
            "UNKNOWN".to_string()
        };

        // Create the DbTinkoffShare
        DbTinkoffShare {
            figi: share.figi.clone(),
            ticker: share.ticker.clone(),
            class_code: share.class_code.clone(),
            isin: share.isin.clone(),
            lot: share.lot as u32,
            currency: share.currency.clone(),

            // Quotation fields
            klong_units,
            klong_nano,
            kshort_units,
            kshort_nano,
            dlong_units,
            dlong_nano,
            dshort_units,
            dshort_nano,
            dlong_min_units,
            dlong_min_nano,
            dshort_min_units,
            dshort_min_nano,

            short_enabled_flag: share.short_enabled_flag,
            name: share.name.clone(),
            exchange: share.exchange.clone(),

            ipo_date: timestamp_to_datetime(&share.ipo_date),
            issue_size: share.issue_size,
            country_of_risk: share.country_of_risk.clone(),
            country_of_risk_name: share.country_of_risk_name.clone(),
            sector: share.sector.clone(),
            issue_size_plan: share.issue_size_plan,

            // Money value fields
            nominal_currency,
            nominal_units,
            nominal_nano,

            trading_status,
            otc_flag: share.otc_flag,
            buy_available_flag: share.buy_available_flag,
            sell_available_flag: share.sell_available_flag,
            div_yield_flag: share.div_yield_flag,
            share_type,

            // Min price increment fields
            min_price_increment_units: min_price_units,
            min_price_increment_nano: min_price_nano,

            api_trade_available_flag: share.api_trade_available_flag,
            uid: share.uid.clone(),
            real_exchange,
            position_uid: share.position_uid.clone(),
            for_iis_flag: share.for_iis_flag,
            for_qual_investor_flag: share.for_qual_investor_flag,
            weekend_flag: share.weekend_flag,
            blocked_tca_flag: share.blocked_tca_flag,
            liquidity_flag: share.liquidity_flag,

            first_1min_candle_date: timestamp_to_datetime(&share.first_1min_candle_date),
            first_1day_candle_date: timestamp_to_datetime(&share.first_1day_candle_date),
        }
    }
}

// Extension methods for DbTinkoffShare to convert stored numeric values back to float
impl DbTinkoffShare {
    pub fn open_price(&self) -> f64 {
        self.klong_units as f64 + (self.klong_nano as f64 / 1_000_000_000.0)
    }

    pub fn min_price_increment(&self) -> f64 {
        self.min_price_increment_units as f64
            + (self.min_price_increment_nano as f64 / 1_000_000_000.0)
    }

    pub fn nominal_value(&self) -> f64 {
        self.nominal_units as f64 + (self.nominal_nano as f64 / 1_000_000_000.0)
    }
}
