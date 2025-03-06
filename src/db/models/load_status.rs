use chrono::{DateTime, NaiveDate, Utc};
use clickhouse::Row;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Row)]
pub struct LoadStatus {
    pub figi: String,
    pub date_from: NaiveDate,
    pub date_to: NaiveDate,
    pub timestamp_from: u64,
    pub timestamp_to: u64,
    pub load_time: DateTime<Utc>,
    pub candles_count: u32,
}