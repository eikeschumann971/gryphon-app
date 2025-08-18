use gryphon_app::domains::path_planning::*;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;
use chrono::Utc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 Starting PathPlanWorker with A* algorithm");
    
    let worker = AStarPathPlanWorker::new();
    worker.run().await?;
    
    Ok(())
}

struct AStarPathPlanWorker {
    worker_id: String,
    capabilities: Vec<PlanningAlgorithm>,
}

impl AStarPathPlanWorker {
    fn new() -> Self {
        Self {
            worker_id: format!("astar-worker-{}", Uuid::new_v4()),
            capabilities: vec![PlanningAlgorithm::AStar],
        }
    }

    async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🚀 Worker {} starting up with capabilities: {:?}", 
                 self.worker_id, self.capabilities);

        // Simulate worker registration and work processing loop
        loop {
            println!("📡 Worker {} waiting for path planning requests...", self.worker_id);
            
            // Simulate receiving a work assignment
            tokio::time::sleep(Duration::from_secs(5)).await;
            
            // Simulate processing a path planning request
            if let Some(path_plan_request) = self.simulate_receive_work().await {
                println!("🎯 Worker {} received planning request: {} -> {}", 
                         self.worker_id, 
                         format!("({:.1}, {:.1})", path_plan_request.start_position.x, path_plan_request.start_position.y),
                         format!("({:.1}, {:.1})", path_plan_request.destination_position.x, path_plan_request.destination_position.y));
                
                let waypoints = self.plan_path_astar(&path_plan_request).await?;
                
                println!("✅ Worker {} completed path with {} waypoints", 
                         self.worker_id, waypoints.len());
                
                // In a real implementation, this would send results back via message bus
                self.simulate_send_result(&path_plan_request.request_id, waypoints).await;
            }
        }
    }

    async fn simulate_receive_work(&self) -> Option<PathPlanRequest> {
        // Simulate receiving work every few iterations
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        if rng.gen_bool(0.7) { // 70% chance of receiving work
            Some(PathPlanRequest {
                request_id: format!("req-{}", Uuid::new_v4()),
                agent_id: format!("agent-{}", rng.gen::<u32>() % 5),
                start_position: Position2D { 
                    x: rng.gen_range(-50.0..50.0), 
                    y: rng.gen_range(-50.0..50.0) 
                },
                destination_position: Position2D { 
                    x: rng.gen_range(-50.0..50.0), 
                    y: rng.gen_range(-50.0..50.0) 
                },
                start_orientation: Orientation2D { angle: 0.0 },
                destination_orientation: Orientation2D { angle: 1.57 },
                created_at: Utc::now(),
            })
        } else {
            None
        }
    }

    async fn plan_path_astar(&self, path_plan_request: &PathPlanRequest) -> Result<Vec<Position2D>, Box<dyn std::error::Error>> {
        println!("🧠 Starting A* pathfinding from ({:.1}, {:.1}) to ({:.1}, {:.1})", 
                 path_plan_request.start_position.x, path_plan_request.start_position.y,
                 path_plan_request.destination_position.x, path_plan_request.destination_position.y);

        // Dummy A* implementation with sleep delays to simulate computation
        let waypoints = self.dummy_astar_algorithm(&path_plan_request.start_position, &path_plan_request.destination_position).await;
        
        Ok(waypoints)
    }

    async fn dummy_astar_algorithm(&self, start: &Position2D, goal: &Position2D) -> Vec<Position2D> {
        let mut waypoints = Vec::new();
        
        println!("  🔍 Initializing A* search...");
        sleep(Duration::from_millis(200)).await;
        
        println!("  📊 Building grid and calculating heuristics...");
        sleep(Duration::from_millis(300)).await;
        
        println!("  🎯 Finding optimal path...");
        
        // Simulate A* iterations
        let distance = ((goal.x - start.x).powi(2) + (goal.y - start.y).powi(2)).sqrt();
        let num_waypoints = (distance / 10.0).ceil() as usize + 1;
        
        for i in 1..=num_waypoints {
            let progress = i as f64 / num_waypoints as f64;
            
            let waypoint = Position2D {
                x: start.x + progress * (goal.x - start.x),
                y: start.y + progress * (goal.y - start.y),
            };
            
            println!("Generated waypoint {}: ({:.2}, {:.2})", 
                     i, waypoint.x, waypoint.y);
            
            waypoints.push(waypoint);
            
            // Simulate computation time for each A* iteration
            sleep(Duration::from_millis(150)).await;
        }
        
        println!("  🎉 A* search completed! Found path with {} waypoints", waypoints.len());
        
        waypoints
    }

    async fn simulate_send_result(&self, request_id: &str, waypoints: Vec<Position2D>) {
        println!("📤 Worker {} sending results for request {}", self.worker_id, request_id);
        println!("   📍 Waypoints:");
        for (i, waypoint) in waypoints.iter().enumerate() {
            println!("      {}. ({:.1}, {:.1})", i + 1, waypoint.x, waypoint.y);
        }
        
        // In a real implementation, this would publish to a message bus
        sleep(Duration::from_millis(100)).await;
        println!("✅ Results sent successfully!");
    }
}
