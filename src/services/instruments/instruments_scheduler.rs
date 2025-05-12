use std::{sync::Arc, time::Duration};
use tokio::time;
use tracing::{debug, error, info};

use super::client::InstrumentsUpdater;
use crate::{env_config::models::app_config::OperationWindow, AppState};

/// Scheduler for periodic tasks related to Tinkoff instruments
pub struct InstrumentsScheduler {
    app_state: Arc<AppState>,
}

impl InstrumentsScheduler {
    pub async fn new(app_state: Arc<AppState>) -> Self {
        InstrumentsScheduler { app_state }
    }

    /// Trigger a manual update (respects enabled flag)
    pub async fn trigger_update(&self) -> Result<u64, Box<dyn std::error::Error>> {
        // Check if enabled before proceeding
        if !self.app_state.settings.app_config.instruments_scheduler.enabled {
            info!("Instruments updates are disabled in configuration");
            return Ok(0);
        }
        
        // Create the client and delegate to its implementation
        let updater = InstrumentsUpdater::new(self.app_state.clone()).await;
        updater.update_shares().await
    }

    /// Start the scheduler with proper configuration checks
    pub async fn start(&self) {
        let config = &self.app_state.settings.app_config.instruments_scheduler;
        
        // Check if enabled
        if !config.enabled {
            info!("Instruments scheduler is disabled in configuration");
            return;
        }

        info!("Starting instruments scheduler");

        // Run initial update if configured
        if config.initial_run {
            info!("Performing initial market instruments update");
            match self.trigger_update().await {
                Ok(count) => info!("Initial update completed: {} instruments updated", count),
                Err(e) => error!("Failed to perform initial instruments update: {}", e),
            }
        }

        // Log operation window
        info!(
            "Instruments scheduler operation window: {} to {} UTC",
            config.start_time, config.end_time
        );

        // Create the updater client
        let updater = InstrumentsUpdater::new(self.app_state.clone()).await;
        
        // Clone app_state for the task
        let app_state = self.app_state.clone();
        
        // Start interval-based loop
        let interval_seconds = config.interval_seconds;
        let mut interval = tokio::time::interval(Duration::from_secs(interval_seconds));

        tokio::spawn(async move {
            loop {
                interval.tick().await;

                // Check operation window
                let config = &app_state.settings.app_config.instruments_scheduler;
                if !config.is_operation_allowed() {
                    debug!(
                        "Instruments scheduler: skipping update - outside operation window (current time: {})",
                        chrono::Utc::now().format("%H:%M:%S")
                    );
                    continue;
                }

                info!("Instruments scheduler: triggering update");

                // Trigger update
                match updater.update_shares().await {
                    Ok(count) => info!("Instruments scheduler: successfully updated {} shares", count),
                    Err(e) => error!("Instruments scheduler: failed to update shares: {}", e),
                }
            }
        });
    }
}
