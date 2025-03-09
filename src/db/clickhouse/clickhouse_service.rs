use crate::db::clickhouse::repository::share_repository::ClickhouseShareRepository;
use crate::db::clickhouse::{
    connection::ClickhouseConnection, repository::candle_repository::ClickhouseCandleRepository,
};
use crate::db::postgres::connection::PostgresConnection;

use crate::env_config::models::app_setting::AppSettings;
use std::sync::Arc;
use tracing::{error, info};

use super::repository::{candle_repository::CandleRepository, share_repository::ShareRepository};

pub struct ClickhouseService {
    // Connections
    pub connection: Arc<ClickhouseConnection>,

    // Analytical repositories (ClickHouse)
    pub candle_repository: Arc<dyn CandleRepository + Send + Sync>,

    pub share_repository: Arc<dyn ShareRepository + Send + Sync>,
}

impl ClickhouseService {
    pub async fn new(settings: &Arc<AppSettings>) -> Result<Self, Box<dyn std::error::Error>> {
        info!("Initializing database service components");

        // Initialize ClickHouse connection
        info!("Creating ClickHouse connection");
        let clickhouse_connection = match ClickhouseConnection::new(settings.clone()).await {
            Ok(conn) => {
                info!("ClickHouse connection established successfully");
                Arc::new(conn)
            }
            Err(e) => {
                error!("Failed to establish ClickHouse connection: {}", e);
                return Err(Box::new(e));
            }
        };

        // Initialize analytical repositories (ClickHouse)
        info!("Initialize repositories (ClickHouse)");
        let candle_repository = Arc::new(ClickhouseCandleRepository::new(
            clickhouse_connection.clone(),
        )) as Arc<dyn CandleRepository + Send + Sync>;

        let analytics_share_repository = Arc::new(ClickhouseShareRepository::new(
            clickhouse_connection.clone(),
        )) as Arc<dyn ShareRepository + Send + Sync>;

        // Initialize operational repositories (PostgreSQL)
        info!("Initialize repositories (PostgreSQL)");

        info!("Database service initialized successfully");
        Ok(Self {
            connection: clickhouse_connection,

            candle_repository,

            share_repository: analytics_share_repository,
        })
    }
}
