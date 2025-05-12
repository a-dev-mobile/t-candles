use crate::env_config::models::app_setting::AppSettings;
use clickhouse::Client;
use std::sync::Arc;
use tracing::{debug, error, info};

#[derive(Clone)]
pub struct ClickhouseConnection {
    client: Client,
       database: String, 
}

impl ClickhouseConnection {
    pub async fn new(settings: Arc<AppSettings>) -> Result<Self, clickhouse::error::Error> {
        info!("Initializing ClickHouse connection...");

        // Сохраняем имя базы данных
        let database = settings.app_env.clickhouse_database.clone();
             // Создаем клиент с настройками аутентификации
        let client = Client::default()
            .with_url(&settings.app_env.clickhouse_url)
            .with_user(&settings.app_env.clickhouse_user)
            .with_password(&settings.app_env.clickhouse_password)
             .with_database(&database) 
            .with_option(
                "connect_timeout",
                settings.app_config.clickhouse.timeout.to_string(),
            )
            .with_option(
                "receive_timeout",
                settings.app_config.clickhouse.timeout.to_string(),
            )
            .with_option(
                "send_timeout",
                settings.app_config.clickhouse.timeout.to_string(),
            );

        // Test connection
        let test_query = "SELECT 1";
        debug!("Executing test query: {}", test_query);

        match client.query(test_query).execute().await {
            Ok(_) => info!("ClickHouse connection successful"),
            Err(e) => {
                error!("Failed to connect to ClickHouse: {}", e);
                return Err(e);
            }
        }

        Ok(Self { client, database })
    }

    pub fn get_client(&self) -> Client {
        self.client.clone()
    }

    /// Возвращает имя базы данных из конфигурации
    pub fn get_database(&self) -> &str {
        &self.database
    }
}
