use gryphon_app::domains::path_planning::*;
use gryphon_app::adapters::inbound::kafka_event_store::KafkaEventStore;
use gryphon_app::common::{EventStore, EventEnvelope, EventMetadata};
use std::time::Duration;
use chrono::Utc;
use std::sync::Arc;
use rdkafka::consumer::{StreamConsumer, Consumer};
use rdkafka::{ClientConfig, Message};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct KafkaPathPlanWorker {
    pub worker_id: String,
    pub planner_id: String,
    pub capabilities: Vec<PlanningAlgorithm>,
}

impl KafkaPathPlanWorker {
    pub fn new(worker_id: String, planner_id: String) -> Self {
        Self {
            worker_id,
            planner_id,
            capabilities: vec![PlanningAlgorithm::AStar],
        }
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🤖 Starting Kafka Path Planning Worker: {}", self.worker_id);
        
        // Initialize Kafka Event Store for publishing events
        let event_store = Arc::new(
            KafkaEventStore::new(
                "localhost:9092", 
                "path-planning-events", 
                &format!("worker-group-{}", self.worker_id)
            ).await?
        );
        
        // Register this worker with the planner via Kafka event
        self.register_worker(&event_store).await?;
        
        // Create dedicated consumer for receiving PlanAssigned events with unique group ID
        let consumer: StreamConsumer = ClientConfig::new()
            .set("group.id", &format!("worker-group-{}", self.worker_id))
            .set("bootstrap.servers", "localhost:9092")
            .set("enable.partition.eof", "false")
            .set("session.timeout.ms", "6000")
            .set("enable.auto.commit", "true")
            .set("auto.offset.reset", "latest") // Only consume new messages
            .create()
            .map_err(|e| format!("Failed to create Kafka consumer: {}", e))?;
        
        consumer.subscribe(&["path-planning-events"])?;
        
        println!("✅ Connected to Kafka event store for distributed event communication");
        println!("📡 Polling Kafka for PlanAssigned events...");
        
        let mut processed_plans = std::collections::HashSet::new();
        
        loop {
            match tokio::time::timeout(Duration::from_millis(1000), consumer.recv()).await {
                Ok(Ok(message)) => {
                    if let Some(payload) = message.payload() {
                        let payload_str = String::from_utf8_lossy(payload);
                        if let Ok(event) = serde_json::from_str::<EventEnvelope>(&payload_str) {
                            println!("📥 Received Kafka event: {} for aggregate {}", event.event_type, event.aggregate_id);
                            
                            // Only process PlanAssigned events for this worker
                            if event.event_type == "PlanAssigned" {
                                if let Ok(event_data) = serde_json::from_value::<PathPlanningEvent>(event.event_data.clone()) {
                                    if let PathPlanningEvent::PlanAssigned { 
                                        plan_id, 
                                        worker_id,
                                        start_position, 
                                        destination_position, 
                                        .. 
                                    } = event_data {
                                        // Check if this assignment is for this worker and not already processed
                                        if worker_id == self.worker_id && !processed_plans.contains(&plan_id) {
                                            processed_plans.insert(plan_id.clone());
                                            println!("🔧 Processing plan assignment from Kafka: {}", plan_id);
                                            
                                            // Simulate path planning work
                                            println!("   📊 Calculating optimal path using A* algorithm...");
                                            tokio::time::sleep(Duration::from_millis(500)).await;
                                            
                                            // Generate a simple path (for demo)
                                            let mut waypoints = Vec::new();
                                            let steps = 4;
                                            for i in 0..=steps {
                                                let t = i as f64 / steps as f64;
                                                let x = start_position.x + t * (destination_position.x - start_position.x);
                                                let y = start_position.y + t * (destination_position.y - start_position.y);
                                                waypoints.push(Position2D { x, y });
                                            }
                                            
                                            println!("   ✅ Path calculated with {} waypoints", waypoints.len());
                                            
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
                                                causation_id: Some(event.event_id),
                                                user_id: None,
                                                source: "pathplan_worker_kafka".to_string(),
                                            };
                                            
                                            let completion_envelope = EventEnvelope::new(
                                                &completion_event, 
                                                "PathPlan", 
                                                metadata
                                            )?;
                                            
                                            // Publish completion to Kafka
                                            event_store.append_events(&plan_id, 1, vec![completion_envelope]).await?;
                                            println!("   📤 Published PlanCompleted event to Kafka");
                                            println!("✅ Plan {} completed and published to Kafka successfully", plan_id);
                                        } else if worker_id != self.worker_id {
                                            println!("🔄 Ignoring assignment for different worker: {} (this worker: {})", worker_id, self.worker_id);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Ok(Err(e)) => {
                    println!("⚠️  Kafka receive error: {}", e);
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
                Err(_) => {
                    // Timeout - continue polling
                    continue;
                }
            }
        }
    }
    
    async fn register_worker(&self, event_store: &Arc<KafkaEventStore>) -> Result<(), Box<dyn std::error::Error>> {
        println!("📝 Registering worker {} with planner via Kafka", self.worker_id);
        
        let registration_event = PathPlanningEvent::WorkerRegistered {
            planner_id: self.planner_id.clone(),
            worker_id: self.worker_id.clone(),
            capabilities: self.capabilities.clone(),
            timestamp: Utc::now(),
        };
        
        let metadata = EventMetadata {
            correlation_id: None,
            causation_id: None,
            user_id: None,
            source: format!("worker-{}", self.worker_id),
        };
        
        let event_envelope = EventEnvelope {
            event_id: uuid::Uuid::new_v4(),
            aggregate_id: self.planner_id.clone(),
            aggregate_type: "PathPlanner".to_string(),
            event_type: "WorkerRegistered".to_string(),
            event_version: 1,
            event_data: serde_json::to_value(&registration_event)?,
            metadata: metadata.clone(),
            occurred_at: Utc::now(),
        };
        
        event_store.append_events(&self.planner_id, 0, vec![event_envelope]).await?;
        
        // Also send ready status
        let ready_event = PathPlanningEvent::WorkerReady {
            planner_id: self.planner_id.clone(),
            worker_id: self.worker_id.clone(),
            timestamp: Utc::now(),
        };
        
        let ready_envelope = EventEnvelope {
            event_id: uuid::Uuid::new_v4(),
            aggregate_id: self.planner_id.clone(),
            aggregate_type: "PathPlanner".to_string(),
            event_type: "WorkerReady".to_string(),
            event_version: 1,
            event_data: serde_json::to_value(&ready_event)?,
            metadata,
            occurred_at: Utc::now(),
        };
        
        event_store.append_events(&self.planner_id, 0, vec![ready_envelope]).await?;
        
        println!("✅ Worker {} registered and marked as ready", self.worker_id);
        Ok(())
    }
}

pub async fn run_kafka_worker() -> Result<(), Box<dyn std::error::Error>> {
    // Generate unique worker ID to avoid conflicts
    let unique_id = uuid::Uuid::new_v4().to_string()[..8].to_string();
    let worker_id = format!("kafka-worker-{}", unique_id);
    let worker = KafkaPathPlanWorker::new(worker_id, "main-path-planner".to_string());
    worker.run().await
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_kafka_worker().await
}
