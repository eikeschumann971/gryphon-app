use crate::planning::plan_path_astar;
use crate::mock::{simulate_receive_work, simulate_send_result};
use gryphon_app::domains::path_planning::*;
use uuid::Uuid;
use std::time::Duration;

pub struct AStarPathPlanWorker {
    pub worker_id: String,
    pub capabilities: Vec<PlanningAlgorithm>,
}

impl AStarPathPlanWorker {
    pub fn new() -> Self {
        Self {
            worker_id: format!("astar-worker-{}", Uuid::new_v4()),
            capabilities: vec![PlanningAlgorithm::AStar],
        }
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🚀 Worker {} starting up with capabilities: {:?}", 
                 self.worker_id, self.capabilities);
        loop {
            println!("📡 Worker {} waiting for path planning requests...", self.worker_id);
            tokio::time::sleep(Duration::from_secs(5)).await;
            if let Some(path_plan_request) = simulate_receive_work().await {
                println!(
                    "🎯 Worker {} received planning request: {} -> {}",
                    self.worker_id,
                    format_args!("({:.1}, {:.1})", path_plan_request.start_position.x, path_plan_request.start_position.y),
                    format_args!("({:.1}, {:.1})", path_plan_request.destination_position.x, path_plan_request.destination_position.y),
                );
                let waypoints = plan_path_astar(&path_plan_request).await?;
                println!("✅ Worker {} completed path with {} waypoints", 
                         self.worker_id, waypoints.len());
                simulate_send_result(&self.worker_id, &path_plan_request.request_id, waypoints).await;
            }
        }
    }
}
