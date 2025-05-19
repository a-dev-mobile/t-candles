use crate::db::clickhouse::clickhouse_service::ClickhouseService;

// src/app_state/mod.rs
use crate::env_config::models::app_setting::AppSettings;

use crate::services::tinkoff_client_grpc::TinkoffClient;
use std::sync::Arc;

pub struct AppState {
    pub settings: Arc<AppSettings>,
    pub clickhouse_service: Arc<ClickhouseService>,

    pub grpc_tinkoff: Arc<TinkoffClient>,
}

impl AppState {
    pub fn new(
        settings: Arc<AppSettings>,
        clickhouse_service: Arc<ClickhouseService>,

        grpc_tinkoff: Arc<TinkoffClient>,
    ) -> Self {
        Self {
            settings,
            clickhouse_service,
    
            grpc_tinkoff,
        }
    }
}
