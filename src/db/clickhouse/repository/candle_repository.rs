use crate::db::clickhouse::connection::ClickhouseConnection;

use crate::db::clickhouse::models::candle::{DailyCandle, DbCandle};
use crate::generate::tinkoff_public_invest_api_contract_v1::HistoricCandle;

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use clickhouse::{insert, Row};
use clickhouse::error::Error as ClickhouseError;
use std::sync::Arc;
use tracing::info;

#[async_trait]
pub trait CandleRepository {
    async fn insert_candles(&self, candles: Vec<HistoricCandle>) -> Result<u64, ClickhouseError>;

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
    async fn insert_candles(&self, candles: Vec<HistoricCandle>) -> Result<u64, ClickhouseError> {
        if candles.is_empty() {
            return Ok(0);
        }

        let client = self.connection.get_client();
        // let  insert = client.insert("market_data.candles_1min")?;

        for candle in candles {
            // insert.write(candle).await?;
        }

        // insert.end().await?;

        // info!("Inserted {} candles into ClickHouse", candles.len());
        // Ok(candles.len() as u64)
        Ok(0)
    }


}
