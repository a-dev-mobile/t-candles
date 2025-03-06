mod env_config;
mod logger;

use env_config::models::{
    app_config::AppConfig,
    app_env::{AppEnv, Env},
    app_setting::AppSettings,
};
use std::{net::SocketAddr, sync::Arc};
use tracing::{debug, info};

/// Initialize application settings and logger
async fn initialize() -> AppSettings {
    let app_env = AppEnv::new();
    let app_config = AppConfig::new(&app_env.env);

    let app_settings = AppSettings {
        app_config,
        app_env,
    };

    // Initialize logger with settings
    logger::init_logger(
        &app_settings.app_config.log.level,
        &app_settings.app_config.log.format,
    )
    .expect("Failed to initialize logger");

    // Log startup information using tracing
    info!("Starting application...");
    info!("Current environment: {}", app_settings.app_env.env);

    // Log configuration details

    // info!("Server: {}:{}", app_env.server_address, app_env.server_port);


   

    // Add environment-specific logging
    if app_settings.app_env.is_local() {
        info!("Running in local development mode");
        // Debug dump of complete configuration only in local mode
        debug!("Full configuration dump: {:#?}", app_settings);
    } else {
        info!("Running in production mode");
    }

    app_settings
}

#[tokio::main]
async fn main() {
    // Environment variables are already loaded via IDE/editor settings

    // Initialize application
    let settings = Arc::new(initialize().await);

    // Parse server address
    let http_addr: SocketAddr = format!(
        "{}:{}",
        settings.app_env.server_address, settings.app_env.server_port,
    )
    .parse()
    .expect("Invalid server address configuration");

    info!("Server address configured at: {}", http_addr);

    // You would add other initialization code here:
    // - Setup databases
    // - Create application router
    // - Initialize clients
    // - Start background services
    // - Start HTTP server

    info!("Application initialization complete!");
}
