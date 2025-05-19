use serde::{Deserialize, Serialize};

#[derive(Debug, clickhouse::Row, Deserialize, Serialize)]
pub struct DbModelMyInstrument{

pub uid: String,
pub first_1min_candle_date: i64,
pub last_1min_candle_date: i64,


}