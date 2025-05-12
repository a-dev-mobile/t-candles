mod api;
mod app_state;
mod db;
mod env_config;
mod generate;
mod layers;
mod logger;
mod services;
mod utils;

use app_state::models::AppState;
use axum::{Router, routing::get};
use db::{
    clickhouse::clickhouse_service::ClickhouseService,
    postgres::postgres_service::PostgresService,
};
use env_config::models::{app_config::AppConfig, app_env::AppEnv, app_setting::AppSettings};
use layers::{create_cors, create_trace};
use services::{candles::candles_scheduler::CandlesScheduler, instruments::instruments_scheduler::InstrumentsScheduler, tinkoff_client_grpc::TinkoffClient};
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tracing::{debug, error, info};

/// Initializes the application's configuration and logging system
///
/// Returns the application settings loaded from environment and config files
async fn initialize_application() -> AppSettings {
    // Load environment variables and configuration
    let environment = AppEnv::new();
    let config = AppConfig::new(&environment.env);

    let app_settings = AppSettings {
        app_config: config,
        app_env: environment,
    };

    // Setup logging with configured level and format
    logger::init_logger(
        &app_settings.app_config.log.level,
        &app_settings.app_config.log.format,
    )
    .expect("Failed to initialize logger");

    // Log application startup information
    info!("Starting application...");
    info!("Current environment: {}", app_settings.app_env.env);

    // Add more detailed logging in development environments
    if app_settings.app_env.is_local() {
        info!("Running in local development mode");
        debug!("Configuration details: {:#?}", app_settings);
    } else {
        info!("Running in production mode");
    }

    app_settings
}

/// Establishes connections to databases
async fn initialize_database_connections(
    settings: Arc<AppSettings>,
) -> (ClickhouseService, PostgresService) {
    info!("Initializing database connections...");

    // Initialize ClickHouse connection
    let clickhouse_service = match ClickhouseService::new(&settings).await {
        Ok(service) => {
            info!("ClickHouse connection established successfully");
            service
        }
        Err(err) => {
            error!("Failed to connect to ClickHouse: {}", err);
            panic!("Cannot continue without ClickHouse connection");
        }
    };

    // Initialize PostgreSQL connection
    let postgres_service = match PostgresService::new(&settings).await {
        Ok(service) => {
            info!("PostgreSQL connection established successfully");
            service
        }
        Err(err) => {
            error!("Failed to connect to PostgreSQL: {}", err);
            panic!("Cannot continue without PostgreSQL connection");
        }
    };

    (clickhouse_service, postgres_service)
}

/// Creates the application router with all API endpoints and middleware
fn create_application_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .layer(create_cors())
        .route("/api-health", get(api::health_api))
        .route("/db-health", get(api::health_db))
        .layer(axum::Extension(app_state.clone()))
        .layer(create_trace())
}

/// Starts the HTTP server on the specified address
async fn start_http_server(app: Router, addr: SocketAddr) {
    info!("Starting HTTP server on {}", addr);

    let listener = match TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(err) => {
            error!("Failed to bind to address {}: {}", addr, err);
            panic!("Cannot start server: {}", err);
        }
    };

    info!("Server started successfully, now accepting connections");

    if let Err(err) = axum::serve(listener, app).await {
        error!("Server error: {}", err);
        panic!("Server failed: {}", err);
    }
}

/// Initializes and starts all background services
/// Initializes and starts all background services
async fn initialize_background_services(app_state: Arc<AppState>) {
    // Initialize the instruments scheduler
    let instruments_scheduler = InstrumentsScheduler::new(app_state.clone()).await;
    
    // Initialize the candles scheduler
    let candles_scheduler = CandlesScheduler::new(app_state.clone());
    
    // Start both schedulers (they'll check their enabled status internally)
    instruments_scheduler.start().await;
    candles_scheduler.start().await;
    
    info!("Background services initialization completed");
}

#[tokio::main]
async fn main() {
    // Initialize application settings and logging
    let settings: Arc<AppSettings> = Arc::new(initialize_application().await);

    // Connect to databases
    let (clickhouse_service, postgres_service) =
        initialize_database_connections(settings.clone()).await;

    // Parse server address from configuration
    let server_address: SocketAddr = format!(
        "{}:{}",
        settings.app_env.server_address, settings.app_env.server_port,
    )
    .parse()
    .expect("Invalid server address configuration");

    info!("Server will listen on: {}", server_address);

    // Initialize Tinkoff API client
    let tinkoff_client = Arc::new(
        TinkoffClient::new(settings.clone())
            .await
            .expect("Failed to initialize Tinkoff API client"),
    );

    // Create application state with all services
    let app_state: Arc<AppState> = Arc::new(AppState {
        settings: settings.clone(),
        clickhouse_service: Arc::new(clickhouse_service),
        postgres_service: Arc::new(postgres_service),
        grpc_tinkoff: tinkoff_client,
    });

    // Initialize and start background services
    initialize_background_services(app_state.clone()).await;

    // Create API router
    let app_router = create_application_router(app_state.clone());

    // // Initialize candle client and load candle data
    // let candle_client = TinkoffCandleClient::new(app_state.clone());
    // if let Err(err) = candle_client.load_and_save_candles().await {
    //     error!("Failed to load and save candles: {}", err);
    // }

    // Start HTTP server
    start_http_server(app_router, server_address).await;

    info!("Application started successfully!");
}
