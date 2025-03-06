use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub log: LogConfig,
    pub clickhouse: ClickhouseConfig,
    pub tinkoff_api: TinkoffApiConfig,
    pub historical_candle_data: HistoricalCandleDataConfig,
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
pub struct TinkoffApiConfig {
    pub base_url: String,
    pub domain: String,
    pub timeout: u64,
    pub keepalive: u64,
}

#[derive(Debug, Deserialize)]
pub struct HistoricalCandleDataConfig {
    pub enabled: bool,
    pub request_delay_ms: u64,
}