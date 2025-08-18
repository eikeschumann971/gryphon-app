use gryphon_app::Config;
use tracing::{info, error};
use std::error::Error;

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

    // Initialize adapters
    // TODO: Initialize Kafka producer/consumer
    // TODO: Initialize PostgreSQL connection pool
    // TODO: Initialize event store
    // TODO: Start application services

    info!("Gryphon App started successfully");

    // Keep the application running
    tokio::signal::ctrl_c().await?;
    info!("Shutting down Gryphon App");

    Ok(())
}
