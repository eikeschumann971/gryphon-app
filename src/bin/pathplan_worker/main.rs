mod worker;
mod planning;
mod mock;
mod communication;

use worker::AStarPathPlanWorker;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 Starting A* Path Planning Worker");
    
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    let mut worker = AStarPathPlanWorker::new().await;
    worker.run().await?;
    
    Ok(())
}
