use crate::db::clickhouse::connection::ClickhouseConnection;
use crate::db::clickhouse::error::ClickhouseError;
use crate::db::models::candle::{Candle, DailyCandle};
use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use clickhouse::Row;
use std::sync::Arc;
use tracing::info;

#[async_trait]
pub trait CandleRepository {
    async fn insert_candles(&self, candles: &[Candle]) -> Result<u64, ClickhouseError>;
    async fn get_candles_by_figi(&self, figi: &str, from: DateTime<Utc>, to: DateTime<Utc>) -> Result<Vec<Candle>, ClickhouseError>;
    async fn get_daily_candles_by_figi(&self, figi: &str, from: NaiveDate, to: NaiveDate) -> Result<Vec<DailyCandle>, ClickhouseError>;
}

pub struct ClickhouseCandleRepository {
    connection: Arc<ClickhouseConnection>,
}

impl ClickhouseCandleRepository {
    pub fn new(connection: Arc<ClickhouseConnection>) -> Self {
        Self { connection }
    }
}

#[async_trait]
impl CandleRepository for ClickhouseCandleRepository {
    async fn insert_candles(&self, candles: &[Candle]) -> Result<u64, ClickhouseError> {
        if candles.is_empty() {
            return Ok(0);
        }
    
        let client = self.connection.get_client();
        let mut insert = client.insert("market_data.candles_1min")?;
        
        for candle in candles {
            insert.write(candle).await?;
        }
        
        insert.end().await?;
        
        info!("Inserted {} candles into ClickHouse", candles.len());
        Ok(candles.len() as u64)
    }

    async fn get_candles_by_figi(&self, figi: &str, from: DateTime<Utc>, to: DateTime<Utc>) -> Result<Vec<Candle>, ClickhouseError> {
        let client = self.connection.get_client();
        
        let query = format!(
            "SELECT * FROM market_data.candles_1min WHERE figi = '{}' AND time >= '{}' AND time <= '{}' ORDER BY time",
            figi, from.to_rfc3339(), to.to_rfc3339()
        );
        
        let candles = client.query(&query).fetch_all::<Candle>().await?;
        Ok(candles)
    }

    async fn get_daily_candles_by_figi(&self, figi: &str, from: NaiveDate, to: NaiveDate) -> Result<Vec<DailyCandle>, ClickhouseError> {
        let client = self.connection.get_client();
        
        let query = format!(
            "SELECT * FROM market_data.candles_1min_daily WHERE figi = '{}' AND day >= '{}' AND day <= '{}' ORDER BY day",
            figi, from, to
        );
        
        let candles = client.query(&query).fetch_all::<DailyCandle>().await?;
        Ok(candles)
    }
}