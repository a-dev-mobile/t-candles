use crate::db::clickhouse::connection::ClickhouseConnection;
use crate::db::models::load_status::DbLoadStatus;
use async_trait::async_trait;
use chrono::{NaiveDate, Utc};
use clickhouse::Row;
use clickhouse::error::Error as ClickhouseError;
use std::sync::Arc;
use tracing::info;

#[async_trait]
pub trait LoadStatusRepository {
    async fn insert_load_status(&self, status: &DbLoadStatus) -> Result<(), ClickhouseError>;
    async fn get_load_status_by_figi(
        &self,
        figi: &str,
    ) -> Result<Vec<DbLoadStatus>, ClickhouseError>;
    async fn get_latest_load_date_by_figi(
        &self,
        figi: &str,
    ) -> Result<Option<NaiveDate>, ClickhouseError>;
}

pub struct ClickhouseLoadStatusRepository {
    connection: Arc<ClickhouseConnection>,
}

impl ClickhouseLoadStatusRepository {
    pub fn new(connection: Arc<ClickhouseConnection>) -> Self {
        Self { connection }
    }
}

#[async_trait]
impl LoadStatusRepository for ClickhouseLoadStatusRepository {
    async fn insert_load_status(&self, status: &DbLoadStatus) -> Result<(), ClickhouseError> {
        let client = self.connection.get_client();

        let mut insert = client.insert("market_data.load_status_1min")?;
        insert.write(status).await?;
        insert.end().await?;

        // info!(
        //     "Inserted load status for figi {} from {} to {}",
        //     status.figi, status.date_from, status.date_to
        // );
        Ok(())
    }

    async fn get_load_status_by_figi(
        &self,
        figi: &str,
    ) -> Result<Vec<DbLoadStatus>, ClickhouseError> {
        let client = self.connection.get_client();

        let query = format!(
            "SELECT * FROM market_data.load_status_1min WHERE figi = '{}' ORDER BY date_from",
            figi
        );

        let result = client.query(&query).fetch_all::<DbLoadStatus>().await?;
        Ok(result)
    }

    async fn get_latest_load_date_by_figi(
        &self,
        figi: &str,
    ) -> Result<Option<NaiveDate>, ClickhouseError> {
        let client = self.connection.get_client();

        let query = format!(
            "SELECT max(date_to) as latest_date FROM market_data.load_status_1min WHERE figi = '{}'",
            figi
        );

        #[derive(serde::Deserialize, clickhouse::Row)]
        struct MaxDate {
            latest_date: Option<NaiveDate>,
        }

        let result = client.query(&query).fetch_one::<MaxDate>().await?;
        Ok(result.latest_date)
    }
}
