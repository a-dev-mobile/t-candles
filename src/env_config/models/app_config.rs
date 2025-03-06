use chrono::{NaiveTime, Utc};
use chrono_tz::Tz;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub log: LogConfig,
    pub postgres_db: PostgresDbConfig,
    pub mongo_db: MongoDbConfig,
    pub tinkoff_api: TinkoffApiConfig,
    pub tinkoff_market_data_updater: UpdaterConfig,
    pub tinkoff_market_data_stream: UpdaterConfig,
    pub currency_rates_updater: UpdaterConfig,
    pub historical_candle_data: HistoricalCandleDataConfig,
    pub historical_candle_updater: HistoricalCandleUpdaterConfig,
}

#[derive(Debug, Deserialize)]
pub struct UpdaterConfig {
    pub enabled: bool,
    pub interval_seconds: u64,
    pub max_retries: u32,
    pub retry_delay_seconds: u64,
    pub update_start_time: String,
    pub update_end_time: String,
    pub timezone: String,
}

#[derive(Debug, Deserialize)]
pub struct LogConfig {
    pub level: String,
    pub format: String,
}

#[derive(Debug, Deserialize)]
pub struct PostgresDbConfig {
    pub max_connections: u32,
    pub timeout_seconds: u64,
    pub pool_size: u32,
}

#[derive(Debug, Deserialize)]
pub struct MongoDbConfig {
    pub timeout_seconds: u64,
    pub pool_size: u32,
    pub retry_writes: bool,
    pub write_concern: String,
    pub read_concern: String,
}

#[derive(Debug, Deserialize)]
pub struct TinkoffApiConfig {
    pub base_url: String,
    pub domain: String,
    pub timeout: u64,
    pub keepalive: u64,
}

#[derive(Debug, Deserialize)]
pub struct HistoricalCandleDataConfig {
    pub enabled: bool,
    pub max_days_history: u32,
    pub request_delay_ms: u64,
    pub run_on_startup: bool,
    #[serde(default = "default_false")]
    pub force_update: bool,
}

#[derive(Debug, Deserialize)]
pub struct HistoricalCandleUpdaterConfig {
    pub enabled: bool,
    pub max_retries: u32,
    pub retry_delay_seconds: u64,
    pub update_start_time: String,
    pub update_end_time: String,
    pub timezone: String,
    pub run_on_startup: bool,
}

fn default_false() -> bool {
    false
}

impl UpdaterConfig {
    pub fn is_update_time(&self) -> bool {
        // Парсим временную зону
        let timezone: Tz = self.timezone.parse().expect("Invalid timezone");

        // Получаем текущее время в UTC и конвертируем его в указанную временную зону
        let current_time = Utc::now().with_timezone(&timezone).time();

        let start_time = NaiveTime::parse_from_str(&self.update_start_time, "%H:%M")
            .expect("Invalid update_start_time format");
        let end_time = NaiveTime::parse_from_str(&self.update_end_time, "%H:%M")
            .expect("Invalid update_end_time format");
        
        if start_time <= end_time {
            current_time >= start_time && current_time <= end_time
        } else {
            // Обработка случая, когда период обновления пересекает полночь
            current_time >= start_time || current_time <= end_time
        }
    }
}