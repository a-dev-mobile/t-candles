use crate::db::db_service::DbService;
// src/app_state/mod.rs
use crate::env_config::models::app_setting::AppSettings;

use crate::services::tinkoff_client_grpc::TinkoffClient;
use std::sync::Arc;

pub struct AppState {
    pub settings: Arc<AppSettings>,
    pub db_service: Arc<DbService>,
    pub grpc_tinkoff: Arc<TinkoffClient>,
}


impl AppState {
    pub fn new(
        settings: Arc<AppSettings>,
        db_service: Arc<DbService>,
        grpc_tinkoff: Arc<TinkoffClient>,
    ) -> Self {
        Self {
            settings,
            db_service,
            grpc_tinkoff,
        }
    }
    
    // Вспомогательные методы для упрощения доступа к часто используемым сервисам
    pub fn get_clickhouse_client(&self) -> clickhouse::Client {
        self.db_service.connection.get_client()
    }
}