use serde::{Deserialize, Serialize};

#[derive(Debug, clickhouse::Row, Deserialize, Serialize)]
pub struct DbLiquidShares {
    pub instrument_id: String,
    pub first_1min_candle_date: i64,
}
