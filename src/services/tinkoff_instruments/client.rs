use std::sync::Arc;

use tracing::info;

use crate::{
    AppState,
    generate::tinkoff_public_invest_api_contract_v1::{
        InstrumentStatus, InstrumentsRequest, Share, SharesResponse,
    },
};

use super::models::share::DbTinkoffShare;

pub struct TinkoffInstrumentsUpdater {
    app_state: Arc<AppState>,
}

impl TinkoffInstrumentsUpdater {
    pub async fn new(app_state: Arc<AppState>) -> Self {
        TinkoffInstrumentsUpdater { app_state }
    }

    pub async fn update_shares(&self) {
        let shares_response = self.fetch_shares().await;
        let total_shares = shares_response.instruments.len();
        info!("Sshares: total {} records", total_shares);

        // Convert each share to MongoDB document
        for share in &shares_response.instruments {
            let db_share: DbTinkoffShare = DbTinkoffShare::from(share);
        }
    }

    async fn fetch_shares(&self) -> SharesResponse {
        let request = self
            .app_state
            .grpc_tinkoff
            .create_request(InstrumentsRequest {
                instrument_status: InstrumentStatus::All as i32,
            })
            .expect("Failed to create request");

        let mut instruments_client = self.app_state.grpc_tinkoff.instruments.clone();

        instruments_client
            .shares(request)
            .await
            .map(|response| response.into_inner())
            .expect("Failed to get shares")
    }
}
