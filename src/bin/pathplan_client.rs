use gryphon_app::domains::path_planning::*;
use tokio::time::{sleep, Duration};
use uuid::Uuid;
use chrono::Utc;
use rand::Rng;
use std::f64::consts::{PI, TAU};

/// Path Planning Client Process
/// 
/// This process simulates clients making path planning requests.
/// It generates realistic path planning scenarios and sends them to the planner service.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting Path Planning Client");
    
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    let client = PathPlanClient::new().await;
    client.run().await?;
    
    Ok(())
}

pub struct PathPlanClient {
    scenarios: Vec<PlanningScenario>,
}

#[derive(Debug, Clone)]
pub struct PlanningScenario {
    pub name: String,
    pub description: String,
    pub agent_id: String,
    pub start_position: Position2D,
    pub destination_position: Position2D,
    pub start_orientation: Orientation2D,
    pub destination_orientation: Orientation2D,
}

impl PathPlanClient {
    pub async fn new() -> Self {
        let scenarios = vec![
            PlanningScenario {
                name: "Office Navigation".to_string(),
                description: "Robot navigating from office entrance to meeting room".to_string(),
                agent_id: "office-robot-001".to_string(),
                start_position: Position2D { x: -50.0, y: -30.0 },
                destination_position: Position2D { x: 40.0, y: 25.0 },
                start_orientation: Orientation2D { angle: 0.0 },
                destination_orientation: Orientation2D { angle: 1.57 }, // 90 degrees
            },
            PlanningScenario {
                name: "Warehouse Pickup".to_string(),
                description: "AGV moving from charging station to shelf A-12".to_string(),
                agent_id: "agv-007".to_string(),
                start_position: Position2D { x: -80.0, y: 60.0 },
                destination_position: Position2D { x: 30.0, y: -40.0 },
                start_orientation: Orientation2D { angle: PI }, // 180 degrees
                destination_orientation: Orientation2D { angle: 0.0 },
            },
            PlanningScenario {
                name: "Hospital Delivery".to_string(),
                description: "Medical robot delivering supplies from pharmacy to ward".to_string(),
                agent_id: "medbot-alpha".to_string(),
                start_position: Position2D { x: 15.0, y: -60.0 },
                destination_position: Position2D { x: -25.0, y: 70.0 },
                start_orientation: Orientation2D { angle: 1.57 },
                destination_orientation: Orientation2D { angle: 4.71 }, // 270 degrees
            },
            PlanningScenario {
                name: "Security Patrol".to_string(),
                description: "Security robot patrolling from checkpoint to perimeter".to_string(),
                agent_id: "security-bot-9".to_string(),
                start_position: Position2D { x: 0.0, y: 0.0 },
                destination_position: Position2D { x: 85.0, y: -85.0 },
                start_orientation: Orientation2D { angle: 0.0 },
                destination_orientation: Orientation2D { angle: 2.36 }, // 135 degrees
            },
            PlanningScenario {
                name: "Kitchen Service".to_string(),
                description: "Restaurant robot taking order from kitchen to table 7".to_string(),
                agent_id: "kitchen-assistant".to_string(),
                start_position: Position2D { x: -45.0, y: 10.0 },
                destination_position: Position2D { x: 55.0, y: -20.0 },
                start_orientation: Orientation2D { angle: 4.71 },
                destination_orientation: Orientation2D { angle: 1.57 },
            },
        ];
        
        Self { scenarios }
    }
    
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🎯 Path Planning Client is running");
        println!("🔄 Will send {} different planning scenarios", self.scenarios.len());
        
        // Demo mode: send all scenarios with delays
        self.run_demo_mode().await?;
        
        // Interactive mode: let user choose scenarios
        // self.run_interactive_mode().await?;
        
        Ok(())
    }
    
    async fn run_demo_mode(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n🎬 Starting demo mode - sending predefined scenarios");
        
        for (i, scenario) in self.scenarios.iter().enumerate() {
            println!("\n📋 Scenario {} of {}: {}", i + 1, self.scenarios.len(), scenario.name);
            println!("   📝 {}", scenario.description);
            
            let request = self.create_request_from_scenario(scenario).await;
            self.send_path_plan_request(request).await?;
            
            // Wait between requests to see the flow clearly
            if i < self.scenarios.len() - 1 {
                println!("   ⏱️  Waiting 8 seconds before next scenario...");
                sleep(Duration::from_secs(8)).await;
            }
        }
        
        println!("\n✅ All demo scenarios completed!");
        
        // Continue with random requests
        self.run_random_requests().await?;
        
        Ok(())
    }
    
    async fn run_random_requests(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n🎲 Starting random request mode");
        let mut rng = rand::thread_rng();
        
        for i in 1..=10 {
            println!("\n🎲 Random request {} of 10", i);
            
            // Pick a random scenario as a base
            let base_scenario = &self.scenarios[rng.gen_range(0..self.scenarios.len())];
            
            // Add some randomness to the positions
            let start_x = base_scenario.start_position.x + rng.gen_range(-20.0..20.0);
            let start_y = base_scenario.start_position.y + rng.gen_range(-20.0..20.0);
            let dest_x = base_scenario.destination_position.x + rng.gen_range(-20.0..20.0);
            let dest_y = base_scenario.destination_position.y + rng.gen_range(-20.0..20.0);
            
            // Clamp to workspace bounds (-100 to 100)
            let start_x = start_x.clamp(-95.0, 95.0);
            let start_y = start_y.clamp(-95.0, 95.0);
            let dest_x = dest_x.clamp(-95.0, 95.0);
            let dest_y = dest_y.clamp(-95.0, 95.0);
            
            let random_scenario = PlanningScenario {
                name: format!("Random-{}", i),
                description: format!("Randomized version of {}", base_scenario.name),
                agent_id: format!("{}-rand", base_scenario.agent_id),
                start_position: Position2D { x: start_x, y: start_y },
                destination_position: Position2D { x: dest_x, y: dest_y },
                start_orientation: Orientation2D { angle: rng.gen_range(0.0..TAU) },
                destination_orientation: Orientation2D { angle: rng.gen_range(0.0..TAU) },
            };
            
            let request = self.create_request_from_scenario(&random_scenario).await;
            self.send_path_plan_request(request).await?;
            
            // Random delay between requests
            let delay = rng.gen_range(3..8);
            println!("   ⏱️  Waiting {} seconds before next request...", delay);
            sleep(Duration::from_secs(delay)).await;
        }
        
        println!("\n🎉 Random request session completed!");
        
        // Keep running indefinitely with periodic requests
        loop {
            sleep(Duration::from_secs(30)).await;
            println!("\n🔄 Sending periodic request...");
            
            let mut rng = rand::thread_rng();
            let base_scenario = &self.scenarios[rng.gen_range(0..self.scenarios.len())];
            let request = self.create_request_from_scenario(base_scenario).await;
            self.send_path_plan_request(request).await?;
        }
    }
    
    async fn create_request_from_scenario(&self, scenario: &PlanningScenario) -> PathPlanRequest {
        let request_id = format!("req-{}", Uuid::new_v4());
        
        PathPlanRequest {
            request_id: request_id.clone(),
            agent_id: scenario.agent_id.clone(),
            start_position: scenario.start_position.clone(),
            destination_position: scenario.destination_position.clone(),
            start_orientation: scenario.start_orientation.clone(),
            destination_orientation: scenario.destination_orientation.clone(),
            created_at: Utc::now(),
        }
    }
    
    async fn send_path_plan_request(&self, request: PathPlanRequest) -> Result<(), Box<dyn std::error::Error>> {
        println!("📤 Sending path plan request:");
        println!("   🆔 Request ID: {}", request.request_id);
        println!("   🤖 Agent: {}", request.agent_id);
        println!("   📍 Start: ({:.1}, {:.1}) @ {:.2}rad", 
                 request.start_position.x, request.start_position.y, request.start_orientation.angle);
        println!("   🎯 Goal:  ({:.1}, {:.1}) @ {:.2}rad", 
                 request.destination_position.x, request.destination_position.y, request.destination_orientation.angle);
        
        // Calculate distance for context
        let distance = ((request.destination_position.x - request.start_position.x).powi(2) + 
                       (request.destination_position.y - request.start_position.y).powi(2)).sqrt();
        println!("   📏 Distance: {:.1} units", distance);
        
        // In a real system, this would send the request to the planner service
        // For now, we'll simulate the network delay and just log
        println!("   📡 Sending to planner service...");
        sleep(Duration::from_millis(100)).await;
        
        println!("   ✅ Request sent successfully!");
        
        // Simulate waiting for response
        println!("   ⏳ Waiting for response...");
        sleep(Duration::from_millis(500)).await;
        
        // Simulate response (in real system this would come from the planner)
        self.simulate_response(&request).await;
        
        Ok(())
    }
    
    async fn simulate_response(&self, request: &PathPlanRequest) {
        // Simulate different response types
        let mut rng = rand::thread_rng();
        let response_type = rng.gen_range(1..=10);
        
        match response_type {
            1..=7 => {
                // Success case (70% probability)
                println!("   🎉 Response: Request accepted and plan assigned to worker");
                sleep(Duration::from_millis(200)).await;
                println!("   🔄 Plan status: In progress...");
                sleep(Duration::from_millis(800)).await;
                
                let waypoint_count = rng.gen_range(3..8);
                println!("   ✅ Plan completed with {} waypoints!", waypoint_count);
                println!("   📍 Sample waypoints:");
                
                // Generate sample waypoints
                for i in 1..=waypoint_count.min(3) {
                    let progress = i as f64 / waypoint_count as f64;
                    let x = request.start_position.x + progress * (request.destination_position.x - request.start_position.x);
                    let y = request.start_position.y + progress * (request.destination_position.y - request.start_position.y);
                    println!("      {}. ({:.1}, {:.1})", i, x, y);
                }
                if waypoint_count > 3 {
                    println!("      ... and {} more waypoints", waypoint_count - 3);
                }
            },
            8..=9 => {
                // No worker available (20% probability)
                println!("   ⏳ Response: Request accepted, waiting for available worker...");
                sleep(Duration::from_millis(1000)).await;
                println!("   ⚠️  No workers currently available, request queued");
            },
            _ => {
                // Validation error (10% probability)
                println!("   ❌ Response: Request rejected - validation failed");
                println!("      Error: Position outside workspace bounds or obstacle collision");
            }
        }
    }
}
