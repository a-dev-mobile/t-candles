use crate::db::clickhouse::clickhouse_service::ClickhouseService;
use crate::db::postgres::postgres_service::PostgresService;
// src/app_state/mod.rs
use crate::env_config::models::app_setting::AppSettings;

use crate::services::tinkoff_client_grpc::TinkoffClient;
use std::sync::Arc;

pub struct AppState {
    pub settings: Arc<AppSettings>,
    pub clickhouse_service: Arc<ClickhouseService>,
    pub postgres_service: Arc<PostgresService>,
    pub grpc_tinkoff: Arc<TinkoffClient>,
}

impl AppState {
    pub fn new(
        settings: Arc<AppSettings>,
        clickhouse_service: Arc<ClickhouseService>,
        postgres_service: Arc<PostgresService>,
        grpc_tinkoff: Arc<TinkoffClient>,
    ) -> Self {
        Self {
            settings,
            clickhouse_service,
            postgres_service,
            grpc_tinkoff,
        }
    }
}
