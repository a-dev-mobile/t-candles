use std::{sync::Arc, time::Duration};
use tokio::time;
use tracing::{debug, error, info};

use super::client::InstrumentsUpdater;
use crate::AppState;

/// Scheduler for periodic tasks related to Tinkoff instruments
pub struct InstrumentsScheduler {
    app_state: Arc<AppState>,
}

impl InstrumentsScheduler {
    pub async fn new(app_state: Arc<AppState>) -> Self {
        InstrumentsScheduler { app_state }
    }

    /// Public method to manually trigger an instruments update
    pub async fn trigger_update(&self) -> Result<u64, Box<dyn std::error::Error>> {
        // Create the client and delegate to its implementation
        let updater = InstrumentsUpdater::new(self.app_state.clone()).await;
        updater.update_shares().await
    }

    /// Starts the scheduler with the configured interval
    pub async fn start(&self) {
        if !self
            .app_state
            .settings
            .app_config
            .market_instruments_updater
            .enabled
        {
            info!("Instruments scheduler is disabled in configuration");
            return;
        }

        let updater_config = &self
            .app_state
            .settings
            .app_config
            .market_instruments_updater;
        // Проверяем, нужен ли первоначальный запуск
        if updater_config.initial_run {
            info!("Performing initial market instruments update");
            match self.trigger_update().await {
                Ok(count) => info!("Initial update completed: {} instruments updated", count),
                Err(e) => error!("Failed to perform initial instruments update: {}", e),
            }
        }
        // Log operation window if configured
        let (start, end) = (&updater_config.start_time, &updater_config.end_time);
        info!(
            "Scheduler operation window configured: {} to {} UTC",
            start, end
        );

        info!(
            "Starting instruments scheduler with {} second interval",
            updater_config.interval_seconds,
        );

        // Create the scheduler interval from configuration
        let interval_seconds = updater_config.interval_seconds;

        // Create the updater client that will handle the actual API calls
        let updater = super::client::InstrumentsUpdater::new(self.app_state.clone()).await;

        // Clone app_state for use in the task
        let app_state = self.app_state.clone();

        // Start interval-based loop
        let mut interval = time::interval(Duration::from_secs(interval_seconds));

        // Main scheduler loop
        tokio::spawn(async move {
            loop {
                interval.tick().await;

                // Check if we're in the allowed operation window
                let updater_config = &app_state.settings.app_config.market_instruments_updater;
                if !updater_config.is_operation_allowed() {
                    debug!(
                        "Scheduler: skipping update - outside operation window (current time: {})",
                        chrono::Utc::now().format("%H:%M:%S")
                    );
                    continue;
                }

                info!("Scheduler: triggering instruments update");

                // Trigger shares update
                match updater.update_shares().await {
                    Ok(count) => info!("Scheduler: successfully updated {} shares", count),
                    Err(e) => error!("Scheduler: failed to update shares: {}", e),
                }

                // Here you can add more scheduled tasks as needed
                // For example:
                // - Update bonds
                // - Update ETFs
                // - Update futures
                // - etc.
            }
        });

        // Return immediately after spawning the background task
        // This ensures we don't block the main thread
    }
}
