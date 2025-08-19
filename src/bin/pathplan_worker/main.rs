mod worker;
mod planning;
mod mock;
mod communication;

use worker::run_worker;
use gryphon_app::adapters::outbound::file_logger::init_file_logger;
use gryphon_app::domains::logger;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 Starting A* Path Planning Worker");
    // Also write to the domain logger (hexagonal port). Keep println! as requested.
    if let Err(e) = init_file_logger("./domain.log") {
        eprintln!("Failed to initialize file logger: {}", e);
    } else {
        logger::info("Starting A* Path Planning Worker");
    }
    
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    run_worker().await?;
    
    Ok(())
}
