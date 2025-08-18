mod worker;
mod planning;
mod mock;
use worker::AStarPathPlanWorker;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 Starting PathPlanWorker with A* algorithm");
    let worker = AStarPathPlanWorker::new();
    worker.run().await?;
    Ok(())
}
