mod db;
mod env_config;
mod generate;
mod logger;
mod services;
use env_config::models::{
    app_config::AppConfig,
    app_env::{AppEnv, Env},
    app_setting::AppSettings,
};
use services::{db_service::DbService, tinkoff_client_grpc::TinkoffClient, tinkoff_instruments::client::TinkoffInstrumentsUpdater};
use std::{net::SocketAddr, sync::Arc};
use tracing::{debug, error, info}; // Import the DbService

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

    // Initialize database service
    info!("Initializing database service...");
    let db_service = match DbService::new(settings.clone()).await {
        Ok(service) => {
            info!("Database service initialized successfully");
            service
        }
        Err(err) => {
            error!("Failed to initialize database service: {}", err);
            panic!("Cannot continue without database service");
        }
    };

    // Parse server address
    let http_addr: SocketAddr = format!(
        "{}:{}",
        settings.app_env.server_address, settings.app_env.server_port,
    )
    .parse()
    .expect("Invalid server address configuration");

    info!("Server address configured at: {}", http_addr);

    // Initialize Tinkoff client
    let grpc_tinkoff = Arc::new(
        TinkoffClient::new(settings.clone())
            .await
            .expect("Failed to initialize Tinkoff client"),
    );

    // Create application state with services
    let app_state: Arc<AppState> = Arc::new(AppState {
        settings: settings.clone(),
        db_service: Arc::new(db_service),
        grpc_tinkoff,
    });
    let updater = TinkoffInstrumentsUpdater::new(app_state).await;


    let a = updater.update_shares().await;

    println!("{:?}", a);
    // You would add other initialization code here:
    // - Setup databases
    // - Create application router
    // - Initialize clients
    // - Start background services
    // - Start HTTP server

    info!("Application initialization complete!");
}

// Application state struct to hold all services
pub struct AppState {
    pub settings: Arc<AppSettings>,
    pub db_service: Arc<DbService>,
    pub grpc_tinkoff: Arc<TinkoffClient>,
}
