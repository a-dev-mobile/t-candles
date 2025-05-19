
use crate::db::clickhouse::{
    connection::ClickhouseConnection, repository::candle_repository::ClickhouseCandleRepository,
};


use crate::env_config::models::app_setting::AppSettings;
use std::sync::Arc;
use tracing::{error, info};

use super::repository::share_repository::ShareRepository;
use super::repository::candle_repository::CandleRepository;

pub struct ClickhouseService {
    // Connections
    pub connection: Arc<ClickhouseConnection>,

    // Analytical repositories (ClickHouse)
    pub repository_candle: Arc<dyn CandleRepository + Send + Sync>,

    pub share_repository: Arc<ShareRepository>,
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

        let analytics_share_repository =
            Arc::new(ShareRepository::new(clickhouse_connection.clone()));

        // Initialize operational repositories (PostgreSQL)
        info!("Initialize repositories (PostgreSQL)");

        info!("Database service initialized successfully");
        Ok(Self {
            connection: clickhouse_connection,

            repository_candle: candle_repository,

            share_repository: analytics_share_repository,
        })
    }

    /// Форматирует полное имя таблицы с учетом схемы из конфигурации
    pub fn format_table_name(&self, table: &str) -> String {
        format!("{}.{}", self.connection.get_database(), table)
    }
}
