use gryphon_app::Config;
use tracing::{info, error};
use std::error::Error;
use std::sync::Arc;
use tokio::sync::mpsc;

use gryphon_app::adapters::outbound::path_planning_data::FilesystemDataSource;
use gryphon_app::adapters::outbound::postgres_graph_store::PostgresGraphStore;
use gryphon_app::application::PathPlanningService;
use gryphon_app::domains::path_planning::PathPlanningCommandActor;
use deadpool_postgres::{Config as DeadPoolConfig, Pool};
use tokio_postgres::NoTls;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("Starting Gryphon App");

    // Load configuration
    let config = Config::from_file("config.toml").await?;
    
    info!("Configuration loaded successfully");
    info!("Kafka brokers: {:?}", config.kafka.brokers);
    info!("PostgreSQL host: {}", config.postgres.host);

    // Initialize event channel and command actor
    let (event_sender, _event_receiver) = mpsc::channel(100);
    let command_actor = PathPlanningCommandActor::new(event_sender);

    // Initialize filesystem data source (uses PATH_PLANNING_DATA_DIR or default)
    let fs_ds = FilesystemDataSource::new(None);
    let ds_arc: Arc<dyn gryphon_app::domains::path_planning::PathPlanningDataSource> = Arc::new(fs_ds);

    // Construct Postgres graph store (async) and application service
    // Configure a deadpool_postgres pool from the application config
    let mut dp_cfg = DeadPoolConfig::new();
    dp_cfg.host = Some(config.postgres.host.clone());
    dp_cfg.port = Some(config.postgres.port as u16);
    dp_cfg.user = Some(config.postgres.username.clone());
    dp_cfg.password = Some(config.postgres.password.clone());
    dp_cfg.dbname = Some(config.postgres.database.clone());
    // deadpool-postgres Config does not expose max_size directly on this struct in all versions;
    // if you need to tune max connections, set it via environment or update this code to match the
    // specific deadpool version in use. For now, use defaults.

    let pool: Pool = dp_cfg.create_pool(None, NoTls).expect("failed to create pg pool");
    let pg_store = PostgresGraphStore::new(pool);
    let pg_arc: Arc<dyn gryphon_app::domains::path_planning::GraphStoreAsync> = Arc::new(pg_store);

    // Construct application service
    let path_planning_service = PathPlanningService::new(command_actor, ds_arc.clone(), pg_arc.clone());

    // Demo: try loading a sample map (non-fatal)
    match path_planning_service.load_map_source("sample_map.geojson") {
        Ok(s) => info!("Loaded sample_map.geojson, len={}", s.len()),
        Err(e) => error!("Failed to load sample_map.geojson: {:?}", e),
    }

    info!("Gryphon App started successfully");

    // Keep the application running
    tokio::signal::ctrl_c().await?;
    info!("Shutting down Gryphon App");

    Ok(())
}
