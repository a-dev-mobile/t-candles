use std::sync::Arc;
use tracing::{debug, error, info};

use crate::{
    AppState,
    db::clickhouse::error::ClickhouseError,
    generate::tinkoff_public_invest_api_contract_v1::{
        InstrumentStatus, InstrumentsRequest, Share,
    },
};

pub struct TinkoffInstrumentsUpdater {
    app_state: Arc<AppState>,
}

impl TinkoffInstrumentsUpdater {
    pub async fn new(app_state: Arc<AppState>) -> Self {
        TinkoffInstrumentsUpdater { app_state }
    }

    pub async fn update_shares(&self) -> Result<u64, Box<dyn std::error::Error>> {
        // Fetch shares from Tinkoff API
        let request = self
            .app_state
            .grpc_tinkoff
            .create_request(InstrumentsRequest {
                instrument_status: InstrumentStatus::All as i32,
            })
            .expect("Failed to create request");

        let mut instruments_client = self.app_state.grpc_tinkoff.instruments.clone();

        let shares_response = instruments_client
            .shares(request)
            .await
            .map(|response| response.into_inner())
            .expect("Failed to get shares");

        let total_shares = shares_response.instruments.len();
        info!("Shares: total {} records fetched", total_shares);

        // Insert directly from proto models
        match self
            .app_state
            .db_service
            .share_repository
            .insert_shares(&shares_response.instruments)
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
