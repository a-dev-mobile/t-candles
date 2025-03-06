use crate::db::clickhouse::connection::ClickhouseConnection;
use crate::db::repository::candle_repository::{CandleRepository, ClickhouseCandleRepository};
use crate::db::repository::load_status_repository::{ClickhouseLoadStatusRepository, LoadStatusRepository};
use crate::db::repository::share_repository::{ClickhouseShareRepository, ShareRepository};
use crate::env_config::models::app_setting::AppSettings;
use std::sync::Arc;
use tracing::{info, error};

pub struct DbService {
    pub connection: Arc<ClickhouseConnection>,
    pub candle_repository: Arc<dyn CandleRepository + Send + Sync>,
    pub load_status_repository: Arc<dyn LoadStatusRepository + Send + Sync>,
    pub share_repository: Arc<dyn ShareRepository + Send + Sync>,
}

impl DbService {
    pub async fn new(settings: Arc<AppSettings>) -> Result<Self, Box<dyn std::error::Error>> {
        info!("Initializing database service components");
        
        // Initialize ClickHouse connection
        info!("Creating ClickHouse connection");
        let connection = match ClickhouseConnection::new(settings).await {
            Ok(conn) => {
                info!("ClickHouse connection established successfully");
                Arc::new(conn)
            },
            Err(e) => {
                error!("Failed to establish ClickHouse connection: {}", e);
                return Err(Box::new(e));
            }
        };
        
        // Initialize repositories
        info!("Initializing repositories");
        let candle_repository = Arc::new(ClickhouseCandleRepository::new(connection.clone())) as Arc<dyn CandleRepository + Send + Sync>;
        let load_status_repository = Arc::new(ClickhouseLoadStatusRepository::new(connection.clone())) as Arc<dyn LoadStatusRepository + Send + Sync>;
        let share_repository = Arc::new(ClickhouseShareRepository::new(connection.clone())) as Arc<dyn ShareRepository + Send + Sync>;
        
        info!("Database service initialized successfully");
        Ok(Self {
            connection,
            candle_repository,
            load_status_repository,
            share_repository,
        })
    }
}