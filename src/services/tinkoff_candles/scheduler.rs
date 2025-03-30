// src/services/tinkoff_candles/scheduler.rs
use std::{sync::Arc, time::Duration};
use tokio::time;
use tracing::{debug, error, info};

use super::client::TinkoffCandleClient;
use crate::AppState;

pub struct TinkoffCandlesScheduler {
    app_state: Arc<AppState>,
}

impl TinkoffCandlesScheduler {
    pub fn new(app_state: Arc<AppState>) -> Self {
        TinkoffCandlesScheduler { app_state }
    }

    pub async fn trigger_update(&self) -> Result<usize, Box<dyn std::error::Error>> {
        // Create the client and delegate to its implementation
        let client = TinkoffCandleClient::new(self.app_state.clone());
        client.load_and_save_candles().await
    }

    pub async fn start(&self) {
        if !self
            .app_state
            .settings
            .app_config
            .tinkoff_historical_candle_updater
            .enabled
        {
            info!("Historical candle data scheduler is disabled in configuration");
            return;
        }

        let candle_config = &self
            .app_state
            .settings
            .app_config
            .tinkoff_historical_candle_updater;

        // Log operation window if configured
        if let (Some(start), Some(end)) = (&candle_config.start_time, &candle_config.end_time) {
            info!(
                "Candle scheduler operation window configured: {} to {} UTC",
                start, end
            );
        }

        info!(
            "Starting historical candle data scheduler with {} ms request delay",
            candle_config.request_delay_ms,
        );

        // Create the candle client that will handle the actual API calls
        let candle_client = TinkoffCandleClient::new(self.app_state.clone());

        // Clone app_state for use in the task
        let app_state = self.app_state.clone();

        // Create a task that runs periodically
        tokio::spawn(async move {
            // Create initial delay (1 minute) to allow other initialization to complete
            tokio::time::sleep(Duration::from_secs(60)).await;

            // Main scheduler loop
            loop {
                // Check if we're in the allowed operation window
                let candle_config = &app_state
                    .settings
                    .app_config
                    .tinkoff_historical_candle_updater;
                let operation_allowed = is_operation_allowed(candle_config);

                if !operation_allowed {
                    debug!(
                        "Candle scheduler: skipping update - outside operation window (current time: {})",
                        chrono::Utc::now().format("%H:%M:%S")
                    );

                    // Wait for 5 minutes before checking again
                    tokio::time::sleep(Duration::from_secs(300)).await;
                    continue;
                }

                info!(
                    "Candle scheduler"
                );

                // Trigger candle update
                match candle_client.load_and_save_candles().await {
                    Ok(count) => info!(
                        "Candle scheduler: successfully processed {} instruments",
                        count
                    ),
                    Err(e) => error!("Candle scheduler: failed to update candle data: {}", e),
                }

                // Wait for 12 hours before the next full update
                // This is a reasonable interval since candles are usually updated once per day
                tokio::time::sleep(Duration::from_secs(12 * 60 * 60)).await;
            }
        });

        // Return immediately after spawning the background task
    }
}

// Helper function outside the impl block to check if operation is allowed based on time window
fn is_operation_allowed(
    candle_config: &crate::env_config::models::app_config::TinkoffHistoricalCandleDataConfig,
) -> bool {
    // If no time window is configured, always allow operation
    if candle_config.start_time.is_none() || candle_config.end_time.is_none() {
        return true;
    }

    // Get current UTC time
    let now = chrono::Utc::now().time();

    // Parse start and end times
    if let (Some(start_str), Some(end_str)) = (&candle_config.start_time, &candle_config.end_time) {
        if let (Ok(start), Ok(end)) = (
            chrono::NaiveTime::parse_from_str(start_str, "%H:%M:%S"),
            chrono::NaiveTime::parse_from_str(end_str, "%H:%M:%S"),
        ) {
            // Check if current time is within the operation window
            if start <= end {
                // Simple case: start time is before end time
                return start <= now && now <= end;
            } else {
                // Case where operation window crosses midnight
                // e.g., start=21:00:00, end=04:00:00
                return start <= now || now <= end;
            }
        }
    }

    // If parsing fails, default to allowing operation
    true
}
