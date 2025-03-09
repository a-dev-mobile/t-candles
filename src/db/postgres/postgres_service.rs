use crate::db::postgres::repository::operational_repository::OperationalRepository;
use crate::db::postgres::{connection::PostgresConnection, repository::operational_repository::PgOperationalRepository};

use crate::env_config::models::app_setting::AppSettings;
use std::sync::Arc;
use tracing::{error, info};

pub struct PostgresService {
    // Connection
    pub connection: Arc<PostgresConnection>,

    // Operational repositories (PostgreSQL)
    pub operational_repository: Arc<dyn OperationalRepository + Send + Sync>,
    
    // Add other PostgreSQL repositories here as needed
    // Example: pub user_repository: Arc<dyn UserRepository + Send + Sync>,
    // Example: pub order_repository: Arc<dyn OrderRepository + Send + Sync>,
}

impl PostgresService {
    pub async fn new(settings: &Arc<AppSettings>) -> Result<Self, Box<dyn std::error::Error>> {
        info!("Initializing PostgreSQL service components");

        // Initialize PostgreSQL connection
        info!("Creating PostgreSQL connection");
        let postgres_connection = match PostgresConnection::new(settings.clone()).await {
            Ok(conn) => {
                info!("PostgreSQL connection established successfully");
                Arc::new(conn)
            }
            Err(e) => {
                error!("Failed to establish PostgreSQL connection: {}", e);
                return Err(Box::new(e));
            }
        };

        // Initialize  repositories
        info!("Initializing  repositories");
        let operational_repository = Arc::new(PgOperationalRepository::new(
            postgres_connection.clone(),
        )) as Arc<dyn OperationalRepository + Send + Sync>;

        // Initialize any other repositories here
        // Example:
        // let user_repository = Arc::new(PgUserRepository::new(
        //    postgres_connection.clone(),
        // )) as Arc<dyn UserRepository + Send + Sync>;

        info!("PostgreSQL service initialized successfully");
        Ok(Self {
            connection: postgres_connection,
            operational_repository,
            // Add other repositories here as they are implemented
            // Example: user_repository,
        })
    }
    
    // Add any service-level methods here that might coordinate between repositories
    
    pub async fn health_check(&self) -> Result<bool, Box<dyn std::error::Error>> {
        match self.operational_repository.health_check().await {
            Ok(result) => Ok(result),
            Err(e) => {
                error!("PostgreSQL health check failed: {}", e);
                Err(Box::new(e))
            }
        }
    }
}