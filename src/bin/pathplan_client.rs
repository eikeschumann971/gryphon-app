use gryphon_app::domains::path_planning::*;
use gryphon_app::adapters::inbound::file_event_store::FileEventStore;
use gryphon_app::common::{EventStore, EventEnvelope, EventMetadata, DomainEvent};
use gryphon_app::config::Config;
use tokio::time::{sleep, Duration};
use uuid::Uuid;
use chrono::Utc;
use rand::Rng;
use std::f64::consts::PI;
use std::sync::Arc;

/// Path Planning Client Process
/// 
/// This process simulates clients making path planning requests using the event-driven architecture.
/// It publishes PathPlanRequested events to the event store and demonstrates realistic scenarios.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting Path Planning Client (Event-Driven)");
    
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    let client = PathPlanClient::new().await?;
    client.run().await?;
    
    Ok(())
}

pub struct PathPlanClient {
    scenarios: Vec<PlanningScenario>,
    event_store: Arc<dyn EventStore>,
    planner_id: String,
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
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // For demo purposes, use default config and in-memory event store
        let config = Config::default();
        println!("📋 Using default configuration for demo");

        // Initialize event store - use file-based store for demo so all processes can share events
        let event_store: Arc<dyn EventStore> = Arc::new(FileEventStore::new("/tmp/gryphon-events"));
        println!("✅ Using file-based event store for demo (shared between processes)");

        let planner_id = "main-path-planner".to_string();
        
        let scenarios = vec![
            PlanningScenario {
                name: "Office Navigation".to_string(),
                description: "Robot navigating from office entrance to meeting room".to_string(),
                agent_id: "office-robot-001".to_string(),
                start_position: Position2D { x: -50.0, y: -30.0 },
                destination_position: Position2D { x: 40.0, y: 25.0 },
                start_orientation: Orientation2D { angle: 0.0 }, // 0 degrees
                destination_orientation: Orientation2D { angle: 90.0 / 180.0 * PI }, // 90 degrees
            },
            PlanningScenario {
                name: "Warehouse Pickup".to_string(),
                description: "AGV moving from charging station to shelf A-12".to_string(),
                agent_id: "agv-007".to_string(),
                start_position: Position2D { x: -80.0, y: 60.0 },
                destination_position: Position2D { x: 30.0, y: -40.0 },
                start_orientation: Orientation2D { angle: PI }, // 180 degrees
                destination_orientation: Orientation2D { angle: 0.0 }, // 0 degrees
            },
            PlanningScenario {
                name: "Hospital Delivery".to_string(),
                description: "Medical robot delivering supplies from pharmacy to ward".to_string(),
                agent_id: "medbot-alpha".to_string(),
                start_position: Position2D { x: 15.0, y: -60.0 },
                destination_position: Position2D { x: -25.0, y: 70.0 },
                start_orientation: Orientation2D { angle: 90.0 / 180.0 * PI },
                destination_orientation: Orientation2D { angle: 270.0 / 180.0 * PI }, // 270 degrees
            },
            PlanningScenario {
                name: "Security Patrol".to_string(),
                description: "Security robot patrolling from checkpoint to perimeter".to_string(),
                agent_id: "security-bot-9".to_string(),
                start_position: Position2D { x: 0.0, y: 0.0 },
                destination_position: Position2D { x: 85.0, y: -85.0 },
                start_orientation: Orientation2D { angle: 0.0 }, // 0 degrees
                destination_orientation: Orientation2D { angle: 135.0 / 180.0 * PI }, // 135 degrees
            },
            PlanningScenario {
                name: "Kitchen Service".to_string(),
                description: "Restaurant robot taking order from kitchen to table 7".to_string(),
                agent_id: "kitchen-assistant".to_string(),
                start_position: Position2D { x: -45.0, y: 10.0 },
                destination_position: Position2D { x: 55.0, y: -20.0 },
                start_orientation: Orientation2D { angle: 270.0 / 180.0 * PI },
                destination_orientation: Orientation2D { angle: 90.0 / 180.0 * PI },
            },
        ];
        
        Ok(Self { 
            scenarios,
            event_store,
            planner_id,
        })
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
                start_orientation: Orientation2D { angle: rng.gen_range(0.0..360.0) / 180.0 * PI },
                destination_orientation: Orientation2D { angle: rng.gen_range(0.0..360.0) / 180.0 * PI },
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
        println!("📤 Publishing path plan request event:");
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
        
        // Create the domain event
        let plan_id = format!("plan-{}", Uuid::new_v4());
        let event = PathPlanningEvent::PathPlanRequested {
            planner_id: self.planner_id.clone(),
            request_id: request.request_id.clone(),
            plan_id: plan_id.clone(),
            agent_id: request.agent_id.clone(),
            start_position: request.start_position.clone(),
            destination_position: request.destination_position.clone(),
            start_orientation: request.start_orientation.clone(),
            destination_orientation: request.destination_orientation.clone(),
            timestamp: Utc::now(),
        };

        // Create event envelope
        let event_envelope = EventEnvelope {
            event_id: Uuid::new_v4(),
            aggregate_id: self.planner_id.clone(),
            aggregate_type: "PathPlanner".to_string(),
            event_type: event.event_type().to_string(),
            event_version: 1,
            event_data: serde_json::to_value(&event)?,
            metadata: EventMetadata {
                correlation_id: Some(Uuid::new_v4()),
                causation_id: None,
                user_id: Some(request.agent_id.clone()),
                source: "pathplan_client".to_string(),
            },
            occurred_at: Utc::now(),
        };

        // Publish to event store
        println!("   📡 Publishing event to event store...");
        
        // Load current version (for this demo, we'll use 0 as we're not implementing full event sourcing)
        let current_version = 0;
        
        match self.event_store.append_events(
            &self.planner_id,
            current_version,
            vec![event_envelope]
        ).await {
            Ok(_) => {
                println!("   ✅ Event published successfully!");
                println!("   🎯 Plan ID: {}", plan_id);
                println!("   📝 Event: PathPlanRequested");
                
                // Simulate processing time
                sleep(Duration::from_millis(100)).await;
                
                // In a real system, we would listen for response events
                // For this demo, we'll simulate the response
                self.simulate_event_response(&request, &plan_id).await;
            }
            Err(e) => {
                println!("   ❌ Failed to publish event: {}", e);
                return Err(e.into());
            }
        }
        
        Ok(())
    }
    
    async fn simulate_event_response(&self, request: &PathPlanRequest, plan_id: &str) {
        // Simulate waiting for event-driven response
        println!("   ⏳ Waiting for response events...");
        sleep(Duration::from_millis(500)).await;
        
        // Simulate different response events
        let mut rng = rand::thread_rng();
        let response_type = rng.gen_range(1..=10);
        
        match response_type {
            1..=7 => {
                // Success case (70% probability) - simulate PlanCompleted event
                println!("   🎉 Event received: PlanAssigned to worker");
                sleep(Duration::from_millis(200)).await;
                println!("   🔄 Event received: Worker processing plan...");
                sleep(Duration::from_millis(800)).await;
                
                let waypoint_count = rng.gen_range(3..8);
                println!("   ✅ Event received: PlanCompleted with {} waypoints!", waypoint_count);
                println!("   📍 Sample waypoints from completed plan:");
                
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
                
                // In a real system, we would publish a PlanCompleted event here
                println!("   📝 Would publish: PlanCompleted event for plan {}", plan_id);
            },
            8..=9 => {
                // No worker available (20% probability)
                println!("   ⏳ Event received: Plan queued, waiting for available worker...");
                sleep(Duration::from_millis(1000)).await;
                println!("   ⚠️  Event received: No workers currently available");
                println!("   📝 Plan {} remains in queue", plan_id);
            },
            _ => {
                // Validation error (10% probability)
                println!("   ❌ Event received: PlanFailed - validation error");
                println!("      Reason: Position outside workspace bounds or obstacle collision");
                println!("   📝 Would publish: PlanFailed event for plan {}", plan_id);
            }
        }
    }
}
