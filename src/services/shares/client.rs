use std::{sync::Arc, time::Duration};
use tokio::time;
use tracing::{debug, error, info};

use crate::{
    app_state::models::AppState, db::clickhouse::clickhouse_service::ClickhouseService, generate::tinkoff_public_invest_api_contract_v1::{InstrumentStatus, InstrumentsRequest}, services::tinkoff_client_grpc::TinkoffClient
};

// Mark the struct as pub to make it visible only within the parent module
pub struct ClientShares {
    clickhouse_service: Arc<ClickhouseService>,
    grpc_tinkoff: Arc<TinkoffClient>,
}

impl ClientShares {
    pub async fn new(
        clickhouse_service: Arc<ClickhouseService>,
        grpc_tinkoff: Arc<TinkoffClient>
    ) -> Self {
        Self { 
            clickhouse_service, 
            grpc_tinkoff 
        }
    }

    // This method is now pub(super) and can be called only by code in the parent module
    pub async fn update_shares(&self) -> Result<u64, Box<dyn std::error::Error>> {
        info!("Fetching updated instruments data");

        // Fetch shares from Tinkoff API
        let request = self
            .grpc_tinkoff
            .create_request(InstrumentsRequest {
                instrument_status: InstrumentStatus::All as i32,
            })
            .expect("Failed to create request");

        let mut instruments_client = self.grpc_tinkoff.instruments.clone();

        let shares_response = instruments_client
            .shares(request)
            .await
            .map(|response| response.into_inner())
            .expect("Failed to get shares");

        let total_shares = shares_response.instruments.len();
        info!("Shares: total {} records fetched", total_shares);

        // Insert directly from proto models
        match self
            .clickhouse_service
            .repository_share
            .insert_shares(&shares_response.instruments, true) // Set clean_first to true
            .await
        {
            Ok(count) => {
                info!("Successfully inserted {} shares into the database", count);
                Ok(count)
            }
            Err(e) => {
                error!("Failed to insert shares into database: {}", e);
                Err(Box::new(e))
            }
        }
    }
}
