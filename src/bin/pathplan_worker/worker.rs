use crate::planning::plan_path_astar;
use crate::communication::{WorkerCommunication, PlannerMessage};
use gryphon_app::domains::path_planning::*;
use uuid::Uuid;
use std::time::Duration;
use tokio::time::sleep;

pub struct AStarPathPlanWorker {
    pub worker_id: String,
    pub capabilities: Vec<PlanningAlgorithm>,
    pub communication: WorkerCommunication,
}

impl AStarPathPlanWorker {
    pub async fn new() -> Self {
        let worker_id = format!("astar-worker-{}", Uuid::new_v4());
        let communication = WorkerCommunication::new(worker_id.clone());
        
        Self {
            worker_id,
            capabilities: vec![PlanningAlgorithm::AStar],
            communication,
        }
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🚀 A* Worker {} starting up", self.worker_id);
        println!("   Capabilities: {:?}", self.capabilities);
        
        // Step 1: Register with the planner service
        self.register_with_planner().await?;
        
        // Step 2: Enter main worker loop
        self.work_loop().await?;
        
        Ok(())
    }
    
    async fn register_with_planner(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n📝 Phase 1: Registration");
        
        self.communication
            .register_with_planner(self.capabilities.clone())
            .await
            .map_err(|e| format!("Registration failed: {}", e))?;
        
        // Wait a bit for registration to be processed
        sleep(Duration::from_millis(500)).await;
        
        println!("✅ Registration phase completed");
        Ok(())
    }
    
    async fn work_loop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n🔄 Phase 2: Work Loop");
        println!("🎯 Worker {} entering work loop", self.worker_id);
        
        loop {
            // Signal that we're ready for work
            self.communication.signal_ready().await?;
            
            // Wait for assignment
            if let Some(assignment) = self.communication.wait_for_assignment().await {
                match assignment {
                    PlannerMessage::WorkAssignment { plan_id, request, timeout_seconds } => {
                        self.handle_work_assignment(plan_id, request, timeout_seconds).await?;
                    }
                    PlannerMessage::CancelAssignment { plan_id } => {
                        println!("❌ Assignment {} was cancelled", plan_id);
                    }
                }
            }
            
            // Small delay before checking for new work
            sleep(Duration::from_millis(100)).await;
        }
    }
    
    async fn handle_work_assignment(
        &self, 
        plan_id: String, 
        request: PathPlanRequest,
        timeout_seconds: u64
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n📋 Assignment Received:");
        println!("   Plan ID: {}", plan_id);
        println!("   Request ID: {}", request.request_id);
        println!("   Agent: {}", request.agent_id);
        println!("   Route: ({:.1}, {:.1}) -> ({:.1}, {:.1})", 
                 request.start_position.x, request.start_position.y,
                 request.destination_position.x, request.destination_position.y);
        println!("   Timeout: {} seconds", timeout_seconds);
        
        // Accept the assignment
        self.communication.accept_assignment(plan_id.clone()).await?;
        
        // Perform the actual path planning
        match self.execute_path_planning(&request).await {
            Ok(waypoints) => {
                println!("✅ Path planning successful!");
                self.communication
                    .report_completion(plan_id, waypoints)
                    .await?;
            }
            Err(e) => {
                println!("❌ Path planning failed: {}", e);
                self.communication
                    .report_failure(plan_id, e.to_string())
                    .await?;
            }
        }
        
        Ok(())
    }
    
    async fn execute_path_planning(&self, request: &PathPlanRequest) -> Result<Vec<Position2D>, Box<dyn std::error::Error>> {
        println!("\n🧠 Executing A* Path Planning:");
        println!("   Worker: {}", self.worker_id);
        println!("   Algorithm: A* (A-Star)");
        
        // Validate the request
        self.validate_request(request)?;
        
        // Execute the planning algorithm
        let waypoints = plan_path_astar(request).await?;
        
        println!("🎉 Planning completed successfully!");
        println!("   Generated {} waypoints", waypoints.len());
        
        // Log first and last few waypoints for verification
        if !waypoints.is_empty() {
            println!("   Start: ({:.2}, {:.2})", waypoints[0].x, waypoints[0].y);
            if waypoints.len() > 1 {
                let last = &waypoints[waypoints.len() - 1];
                println!("   End:   ({:.2}, {:.2})", last.x, last.y);
            }
            if waypoints.len() > 2 {
                println!("   Via {} intermediate waypoints", waypoints.len() - 2);
            }
        }
        
        Ok(waypoints)
    }
    
    fn validate_request(&self, request: &PathPlanRequest) -> Result<(), Box<dyn std::error::Error>> {
        // Check if positions are within reasonable bounds
        let bounds = 100.0; // Workspace bounds
        
        if request.start_position.x.abs() > bounds || request.start_position.y.abs() > bounds {
            return Err("Start position outside workspace bounds".into());
        }
        
        if request.destination_position.x.abs() > bounds || request.destination_position.y.abs() > bounds {
            return Err("Destination position outside workspace bounds".into());
        }
        
        // Check if start and destination are different
        let dx = request.destination_position.x - request.start_position.x;
        let dy = request.destination_position.y - request.start_position.y;
        let distance = (dx * dx + dy * dy).sqrt();
        
        if distance < 0.1 {
            return Err("Start and destination positions are too close".into());
        }
        
        println!("✅ Request validation passed (distance: {:.1})", distance);
        Ok(())
    }
}
