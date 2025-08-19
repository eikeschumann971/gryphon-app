mod communication;
mod mock;
mod planning;
mod worker;

use gryphon_app::adapters::outbound::file_logger::init_file_logger;
use worker::run_worker;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 Starting A* Path Planning Worker");
    // Initialize domain file logger and inject into worker
    let logger = match init_file_logger("./domain.log") {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to initialize file logger: {}", e);
            gryphon_app::adapters::outbound::init_console_logger()
        }
    };

    // Initialize tracing
    tracing_subscriber::fmt::init();

    run_worker(logger).await?;

    Ok(())
}
