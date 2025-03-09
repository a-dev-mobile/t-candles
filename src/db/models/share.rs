use serde::{Deserialize, Serialize};

#[derive(Debug, clickhouse::Row, Deserialize, Serialize)]
pub struct DbSharesLiquid {
    pub instrument_id: String,
    pub first_1min_candle_date: i64,
}
