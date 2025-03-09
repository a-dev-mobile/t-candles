use chrono::{DateTime, NaiveDate, Utc};
use clickhouse::Row;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Row)]
pub struct DbLoadStatus {
    pub instrument_id: String,
    pub to: i64,
}
