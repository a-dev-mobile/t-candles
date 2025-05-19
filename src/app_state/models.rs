use crate::db::clickhouse::clickhouse_service::ClickhouseService;

// src/app_state/mod.rs
use crate::env_config::models::app_setting::AppSettings;

use crate::services::candles::client_candle::ClientCandle;

use crate::services::shares::client::ClientShares;
use crate::services::tinkoff_client_grpc::TinkoffClient;

use std::sync::Arc;

pub struct AppState {
    pub settings: Arc<AppSettings>,
    pub clickhouse_service: Arc<ClickhouseService>,
    pub grpc_tinkoff: Arc<TinkoffClient>,

    // Клиенты
    pub client_tinkoff_candle: Arc<ClientCandle>,
    pub client_shares: Arc<ClientShares>,
}

impl AppState {
    pub async fn new(
        settings: Arc<AppSettings>,
        clickhouse_service: Arc<ClickhouseService>,
        grpc_tinkoff: Arc<TinkoffClient>,
    ) -> Self {
        //  создаем бюзнес-клиентов
        let client_tinkoff_candle = Arc::new(ClientCandle::new(
            clickhouse_service.clone(),
            grpc_tinkoff.clone(),
            settings.clone(),
        ));

        let client_shares =
            Arc::new(ClientShares::new(clickhouse_service.clone(), grpc_tinkoff.clone()).await);

        Self {
            settings,
            clickhouse_service,
            grpc_tinkoff,

            client_tinkoff_candle,
            client_shares,
        }
    }
}
