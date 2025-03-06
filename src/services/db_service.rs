use crate::db::clickhouse::connection::ClickhouseConnection;
use crate::db::repository::candle_repository::{CandleRepository, ClickhouseCandleRepository};
use crate::db::repository::load_status_repository::{ClickhouseLoadStatusRepository, LoadStatusRepository};
use crate::env_config::models::app_setting::AppSettings;
use std::sync::Arc;

pub struct DbService {
    pub connection: Arc<ClickhouseConnection>,
    pub candle_repository: Arc<dyn CandleRepository + Send + Sync>,
    pub load_status_repository: Arc<dyn LoadStatusRepository + Send + Sync>,
}

impl DbService {
    pub async fn new(settings: Arc<AppSettings>) -> Result<Self, Box<dyn std::error::Error>> {
        let connection = Arc::new(ClickhouseConnection::new(settings).await?);
        
        let candle_repository = Arc::new(ClickhouseCandleRepository::new(connection.clone())) as Arc<dyn CandleRepository + Send + Sync>;
        let load_status_repository = Arc::new(ClickhouseLoadStatusRepository::new(connection.clone())) as Arc<dyn LoadStatusRepository + Send + Sync>;
        
        Ok(Self {
            connection,
            candle_repository,
            load_status_repository,
        })
    }
}