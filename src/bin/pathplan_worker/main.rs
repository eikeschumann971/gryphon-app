mod worker;
mod planning;
mod mock;
mod communication;

use worker::run_worker;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 Starting A* Path Planning Worker");
    
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    run_worker().await?;
    
    Ok(())
}
