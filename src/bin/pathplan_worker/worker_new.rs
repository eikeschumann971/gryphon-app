use crate::planning::plan_path_astar;
use gryphon_app::domains::path_planning::*;
use gryphon_app::adapters::inbound::file_event_store::FileEventStore;
use gryphon_app::common::{EventStore, EventEnvelope, EventMetadata};
use uuid::Uuid;
use std::time::Duration;
use tokio::time::sleep;
use chrono::Utc;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct AStarPathPlanWorker {
    pub worker_id: String,
    pub planner_id: String,
    pub capabilities: Vec<PlanningAlgorithm>,
}

impl AStarPathPlanWorker {
    pub fn new(worker_id: String, planner_id: String) -> Self {
        Self {
            worker_id,
            planner_id,
            capabilities: vec![PlanningAlgorithm::AStar],
        }
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Starting path planning worker: {}", self.worker_id);
        
        // Use FileEventStore for shared events
        let event_store = Arc::new(FileEventStore::new("/tmp/gryphon-events"));
        
        loop {
            // Load all PlanAssigned events from the shared event store
            let plan_events = event_store.load_events_by_type("PlanAssigned").await?;
            
            if !plan_events.is_empty() {
                println!("Found {} plan assignments to process", plan_events.len());
                
                for plan_event in plan_events {
                    // Extract PlanAssigned data from JSON
                    if let Ok(event_data) = serde_json::from_value::<PathPlanningEvent>(plan_event.event_data.clone()) {
                        if let PathPlanningEvent::PlanAssigned { 
                            plan_id, 
                            start_position, 
                            destination_position, 
                            .. 
                        } = event_data {
                            // Check if plan is already completed
                            let completion_events = event_store.load_events_by_type("PlanCompleted").await?;
                            
                            let already_completed = completion_events.iter().any(|completion_event| {
                                if let Ok(completion_data) = serde_json::from_value::<PathPlanningEvent>(completion_event.event_data.clone()) {
                                    if let PathPlanningEvent::PlanCompleted { 
                                        plan_id: completed_plan_id, 
                                        .. 
                                    } = completion_data {
                                        completed_plan_id == plan_id
                                    } else {
                                        false
                                    }
                                } else {
                                    false
                                }
                            });
                            
                            if !already_completed {
                                println!("🔧 Processing plan assignment: {}", plan_id);
                                
                                // Simulate some work - simple path for demo
                                let waypoints = vec![start_position, destination_position];
                                
                                // Create PlanCompleted event
                                let completion_event = PathPlanningEvent::PlanCompleted {
                                    planner_id: self.planner_id.clone(),
                                    plan_id: plan_id.clone(),
                                    worker_id: Some(self.worker_id.clone()),
                                    waypoints,
                                    timestamp: Utc::now(),
                                };
                                
                                let metadata = EventMetadata {
                                    correlation_id: None,
                                    causation_id: Some(plan_event.event_id),
                                    user_id: None,
                                    source: "PathPlanWorker".to_string(),
                                };
                                
                                let completion_envelope = EventEnvelope::new(
                                    &completion_event, 
                                    "PathPlan", 
                                    metadata
                                )?;
                                
                                event_store.append_events(vec![completion_envelope]).await?;
                                println!("✅ Plan completed and event published");
                            }
                        }
                    }
                }
            }
            
            // Wait before next poll
            sleep(Duration::from_secs(2)).await;
        }
    }
}

pub async fn run_worker() -> Result<(), Box<dyn std::error::Error>> {
    let worker = AStarPathPlanWorker::new("worker-1".to_string(), "planner-1".to_string());
    worker.run().await
}
