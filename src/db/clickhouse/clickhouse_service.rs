use crate::db::clickhouse::{
    connection::ClickhouseConnection, repository::candle_repository::ClickhouseCandleRepository,
};

use crate::env_config::models::app_setting::AppSettings;
use std::sync::Arc;
use tracing::{error, info};

use super::repository::candle_repository::CandleRepository;
use super::repository::repository_my_instrument::RepositoryMyInstrument;
use super::repository::repository_share::ShareRepository;

pub struct ClickhouseService {
    // Connections
    pub connection: Arc<ClickhouseConnection>,

    pub repository_candle: Arc<dyn CandleRepository + Send + Sync>,

    pub repository_share: Arc<ShareRepository>,
    pub repository_my_instrument: Arc<RepositoryMyInstrument>,
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
        let repository_candle = Arc::new(ClickhouseCandleRepository::new(
            clickhouse_connection.clone(),
        )) as Arc<dyn CandleRepository + Send + Sync>;

        let repository_share = Arc::new(ShareRepository::new(clickhouse_connection.clone()));

        let repository_my_instrument =
            Arc::new(RepositoryMyInstrument::new(clickhouse_connection.clone()));
        // Initialize operational repositories (PostgreSQL)
        info!("Initialize repositories (PostgreSQL)");

        info!("Database service initialized successfully");
        Ok(Self {
            connection: clickhouse_connection,

            repository_candle,

            repository_share,
            repository_my_instrument,
        })
    }


}
