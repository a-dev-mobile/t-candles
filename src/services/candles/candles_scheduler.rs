use std::{sync::Arc, time::Duration};
use tracing::{debug, error, info};

use super::client::TinkoffCandleClient;
use crate::{env_config::models::app_config::OperationWindow, AppState};

pub struct CandlesScheduler {
    app_state: Arc<AppState>,
}

impl CandlesScheduler {
    pub fn new(app_state: Arc<AppState>) -> Self {
        CandlesScheduler { app_state }
    }

    /// Trigger a manual update (respects enabled flag)
    pub async fn trigger_update(&self) -> Result<usize, Box<dyn std::error::Error>> {
        // Check if enabled before proceeding
        if !self.app_state.settings.app_config.candles_scheduler.enabled {
            info!("Candle updates are disabled in configuration");
            return Ok(0);
        }
        
        // Create the client and delegate to its implementation
        let client = TinkoffCandleClient::new(self.app_state.clone());
        client.load_and_save_candles().await
    }

    /// Start the scheduler with proper configuration checks
    pub async fn start(&self) {
        let config = &self.app_state.settings.app_config.candles_scheduler;
        
        // Check if enabled
        if !config.enabled {
            info!("Historical candle data scheduler is disabled in configuration");
            return;
        }

        info!("Starting candles scheduler");

        // Run initial update if configured
        if config.initial_run {
            info!("Performing initial historical candle data update");
            match self.trigger_update().await {
                Ok(count) => info!(
                    "Initial candle update completed: processed {} instruments",
                    count
                ),
                Err(e) => error!("Failed to perform initial candle update: {}", e),
            }
        }

        // Log operation window
        info!(
            "Candle scheduler operation window: {} to {} UTC",
            config.start_time, config.end_time
        );

        info!(
            "Candle scheduler configured with {} ms request delay",
            config.request_delay_ms,
        );

        // Create the candle client
        let candle_client = TinkoffCandleClient::new(self.app_state.clone());
        
        // Clone app_state for the task
        let app_state = self.app_state.clone();

        tokio::spawn(async move {
            // Initial delay to allow other initialization to complete
            tokio::time::sleep(Duration::from_secs(60)).await;

            // Main scheduler loop
            loop {
                // Check operation window
                let config = &app_state.settings.app_config.candles_scheduler;
                if !config.is_operation_allowed() {
                    debug!(
                        "Candle scheduler: skipping update - outside operation window (current time: {})",
                        chrono::Utc::now().format("%H:%M:%S")
                    );

                    // Wait before checking again
                    tokio::time::sleep(Duration::from_secs(300)).await;
                    continue;
                }

                info!("Candle scheduler: triggering update");

                // Trigger candle update
                match candle_client.load_and_save_candles().await {
                    Ok(count) => info!(
                        "Candle scheduler: successfully processed {} instruments",
                        count
                    ),
                    Err(e) => error!("Candle scheduler: failed to update candle data: {}", e),
                }

                // Wait before the next full update
                tokio::time::sleep(Duration::from_secs(12 * 60 * 60)).await;
            }
        });
    }
}