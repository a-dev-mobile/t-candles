use chrono::{NaiveTime, Utc};
use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub log: LogConfig,
    pub clickhouse: ClickhouseConfig,
    pub postgres: PostgresConfig,
    pub tinkoff_api: TinkoffApiConfig,
    pub tinkoff_market_data_updater: TinkoffMarketDataUpdater,
    pub tinkoff_historical_candle_updater: TinkoffHistoricalCandleDataConfig,
}
#[derive(Debug, Deserialize)]
pub struct TinkoffMarketDataUpdater {
    pub enabled: bool,
    pub interval_seconds: u64,
    #[serde(default)]
    pub start_time: Option<String>, // Start time in UTC, format: "HH:MM:SS"
    #[serde(default)]
    pub end_time: Option<String>, // End time in UTC, format: "HH:MM:SS"
}

#[derive(Debug, Deserialize)]
pub struct LogConfig {
    pub level: String,
    pub format: String,
}

#[derive(Debug, Deserialize)]
pub struct ClickhouseConfig {
    pub timeout: u64,
    pub pool_min: u32,
    pub pool_max: u32,
}
#[derive(Debug, Deserialize)]
pub struct PostgresConfig {
    pub timeout: u64,
    pub max_connections: u32,
    pub min_connections: u32,
    pub max_lifetime: u64,
    pub idle_timeout: u64,
}

#[derive(Debug, Deserialize)]
pub struct TinkoffApiConfig {
    pub base_url: String,
    pub domain: String,
    pub timeout: u64,
    pub keepalive: u64,
}

#[derive(Debug, Deserialize)]
pub struct TinkoffHistoricalCandleDataConfig {
    pub enabled: bool,
    pub request_delay_ms: u64,
    #[serde(default)]
    pub start_time: Option<String>, // Start time in UTC, format: "HH:MM:SS"
    #[serde(default)]
    pub end_time: Option<String>, // End time in UTC, format: "HH:MM:SS"
}
impl TinkoffMarketDataUpdater {
    /// Checks if the current time is within the allowed operation window
    pub fn is_operation_allowed(&self) -> bool {
        // If no time window is configured, always allow operation
        if self.start_time.is_none() || self.end_time.is_none() {
            return true;
        }

        // Get current UTC time
        let now = chrono::Utc::now().time();

        // Parse start and end times
        if let (Some(start_str), Some(end_str)) = (&self.start_time, &self.end_time) {
            if let (Ok(start), Ok(end)) = (
                NaiveTime::parse_from_str(start_str, "%H:%M:%S"),
                NaiveTime::parse_from_str(end_str, "%H:%M:%S"),
            ) {
                // Check if current time is within the operation window
                if start <= end {
                    // Simple case: start time is before end time
                    return start <= now && now <= end;
                } else {
                    // Case where operation window crosses midnight
                    // e.g., start=21:00:00, end=04:00:00
                    return start <= now || now <= end;
                }
            }
        }

        // If parsing fails, default to allowing operation
        true
    }
}
