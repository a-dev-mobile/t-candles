mod api;
mod db;
mod env_config;
mod generate;
mod layers;
mod logger;
mod services;
mod utils;

mod app_state;


use app_state::models::AppState;
use axum::{Router, routing::get};
use chrono::DateTime;
use db::db_service::DbService;
use env_config::models::{app_config::AppConfig, app_env::AppEnv, app_setting::AppSettings};
use layers::{create_cors, create_trace};
use serde::de;
use services::{

    tinkoff_candles::client::TinkoffCandleClient, tinkoff_client_grpc::TinkoffClient, tinkoff_instruments::scheduler::TinkoffInstrumentsScheduler
};
use std::{net::SocketAddr, sync::Arc};
use tokio::{net::TcpListener, signal};
use tracing::{debug, error, info};


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

/// Setup database connections
async fn setup_databases(settings: Arc<AppSettings>) -> DbService {
    info!("Initializing database service...");
    let db_service: DbService = match DbService::new(settings).await {
        Ok(service) => {
            info!("Database service initialized successfully");
            service
        }
        Err(err) => {
            error!("Failed to initialize database service: {}", err);
            panic!("Cannot continue without database service");
        }
    };
    db_service
}

fn create_app(app_state: Arc<AppState>) -> Router {
    Router::new()
        .layer(create_cors())
        .route("/api-health", get(api::health_api))
        .route("/db-health", get(api::health_db))
        .layer(axum::Extension(app_state.clone()))
        .layer(create_trace())
}

/// Start the HTTP server
async fn run_server(app: Router, addr: SocketAddr) {
    tracing::info!("Starting server on {}", addr);
    let listener = TcpListener::bind(addr).await.unwrap();
    tracing::info!("Server started successfully");
    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}

#[tokio::main]
async fn main() {
    // Initialize application
    let settings: Arc<AppSettings> = Arc::new(initialize().await);
    // Initialize database service
    let db_service = setup_databases(settings.clone()).await;
    
    // No need to extract ClickHouse connection separately as we'll use AppState
    
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
    
    // Initialize and start services
    start_services(app_state.clone()).await;
    
    // Create application router
    let app = create_app(app_state.clone());
    
    let a  = app_state.db_service.share_repository.get_liquid_shares().await.unwrap();
    
    dbg!(a);
    
    let client = TinkoffCandleClient::new(app_state.clone());
    
    let from = DateTime::from_timestamp(1709769600, 0).unwrap();
    let to = DateTime::from_timestamp(1709856000, 0).unwrap();
    
    let b = client.get_minute_candles("BBG000BBV4M5",from,to ).await.unwrap();
    
    dbg!(b);

    
    run_server(app, http_addr).await;
    
    info!("Application initialization complete!");
    


    // Keep the application running until a shutdown signal is received
    wait_for_shutdown_signal().await;
    
    info!("Shutdown signal received, terminating application...");
}

// Wait for a shutdown signal (Ctrl+C on most systems)
async fn wait_for_shutdown_signal() {
    // Wait for CTRL+C
    match signal::ctrl_c().await {
        Ok(()) => info!("CTRL+C received, shutting down..."),
        Err(e) => error!("Failed to listen for shutdown signal: {}", e),
    }
    // Here you can add a graceful shutdown sequence if needed:
    // - Close database connections
    // - Finish pending tasks
    // - Notify services to terminate
}

// Start all required services
async fn start_services(app_state: Arc<AppState>) {
    // Initialize the instruments scheduler
    let instruments_scheduler = TinkoffInstrumentsScheduler::new(app_state.clone()).await;
    
    // If you need to trigger an immediate update before starting the scheduler:
    match instruments_scheduler.trigger_update().await {
        Ok(count) => info!(
            "Initial instrument update completed. Updated {} records",
            count
        ),
        Err(e) => error!("Failed to perform initial instrument update: {}", e),
    }
    
    // Start the automatic scheduler
    instruments_scheduler.start().await;
}
