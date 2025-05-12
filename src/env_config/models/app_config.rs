use chrono::{NaiveTime, Utc};
use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub log: LogConfig,
    pub clickhouse: ClickhouseConfig,
    pub postgres: PostgresConfig,
    pub tinkoff_api: TinkoffApiConfig,
    pub instruments_scheduler: InstrumentsScheduler,
    pub candles_scheduler: CandlesScheduler,
}
#[derive(Debug, Deserialize)]
pub struct InstrumentsScheduler {
    pub enabled: bool,
    pub initial_run: bool,
    pub interval_seconds: u64,
    pub start_time: String, // Start time in UTC, format: "HH:MM:SS"
    pub end_time: String,   // End time in UTC, format: "HH:MM:SS"
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
pub struct CandlesScheduler {
    pub enabled: bool,
    pub initial_run: bool,
    pub request_delay_ms: u64,

    pub start_time: String, // Start time in UTC, format: "HH:MM:SS"

    pub end_time: String, // End time in UTC, format: "HH:MM:SS"
}

// For CandlesScheduler
impl OperationWindow for CandlesScheduler {
    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn start_time(&self) -> &str {
        &self.start_time
    }

    fn end_time(&self) -> &str {
        &self.end_time
    }
}
impl OperationWindow for InstrumentsScheduler {
    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn start_time(&self) -> &str {
        &self.start_time
    }

    fn end_time(&self) -> &str {
        &self.end_time
    }
}

/// Common trait for schedulers that have time-based operation windows
pub trait OperationWindow {
    /// Check if scheduler is enabled
    fn is_enabled(&self) -> bool;

    /// Get the start time string
    fn start_time(&self) -> &str;

    /// Get the end time string
    fn end_time(&self) -> &str;

    /// Check if operation is allowed based on enabled flag and time window
    fn is_operation_allowed(&self) -> bool {
        // First check the enabled flag - highest priority
        if !self.is_enabled() {
            return false;
        }

        // Get current UTC time
        let now = chrono::Utc::now().time();

        // Parse start and end times
        let (start_str, end_str) = (self.start_time(), self.end_time());
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

        // If parsing fails, log the error and default to not allowing operation
        tracing::error!(
            "Failed to parse time window: start='{}', end='{}'",
            start_str,
            end_str
        );
        false
    }
}
