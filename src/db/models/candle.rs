use chrono::{DateTime, Utc};
use clickhouse::Row;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Row)]
pub struct Candle {
    pub figi: String,
    pub time: DateTime<Utc>,
    pub open_units: i64,
    pub open_nano: i32,
    pub high_units: i64,
    pub high_nano: i32,
    pub low_units: i64,
    pub low_nano: i32,
    pub close_units: i64,
    pub close_nano: i32,
    pub volume: u64,
    pub is_complete: bool,
}

impl Candle {
    pub fn open_price(&self) -> f64 {
        self.open_units as f64 + (self.open_nano as f64 / 1_000_000_000.0)
    }
    
    pub fn high_price(&self) -> f64 {
        self.high_units as f64 + (self.high_nano as f64 / 1_000_000_000.0)
    }
    
    pub fn low_price(&self) -> f64 {
        self.low_units as f64 + (self.low_nano as f64 / 1_000_000_000.0)
    }
    
    pub fn close_price(&self) -> f64 {
        self.close_units as f64 + (self.close_nano as f64 / 1_000_000_000.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Row)]
pub struct DailyCandle {
    pub figi: String,
    pub day: chrono::NaiveDate,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub first_open_units: i64,
    pub first_open_nano: i32,
    pub max_high: f64,
    pub min_low: f64,
    pub last_close_units: i64,
    pub last_close_nano: i32,
    pub total_volume: u64,
    pub candle_count: u64,
}

impl DailyCandle {
    pub fn open_price(&self) -> f64 {
        self.first_open_units as f64 + (self.first_open_nano as f64 / 1_000_000_000.0)
    }
    
    pub fn close_price(&self) -> f64 {
        self.last_close_units as f64 + (self.last_close_nano as f64 / 1_000_000_000.0)
    }
}