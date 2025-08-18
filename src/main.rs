use gryphon_app::Config;
use tracing::{info, error};
use std::error::Error;
use std::sync::Arc;
use tokio::sync::mpsc;

use gryphon_app::adapters::outbound::path_planning_data::FilesystemDataSource;
use gryphon_app::application::PathPlanningService;
use gryphon_app::domains::path_planning::PathPlanningCommandActor;

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

    // Construct application service
    let path_planning_service = PathPlanningService::new(command_actor, ds_arc.clone());

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
