use gryphon_app::domains::path_planning::*;
use gryphon_app::adapters::inbound::kafka_event_store::KafkaEventStore;
use gryphon_app::common::{EventStore, EventEnvelope, EventMetadata, DomainEvent};
use tokio::time::{interval, Duration};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::Utc;
use std::sync::Arc;
use rdkafka::consumer::{StreamConsumer, Consumer};
use rdkafka::{ClientConfig, Message};

#[derive(Debug, Clone)]
pub struct WorkerInfo {
    pub worker_id: String,
    pub capabilities: Vec<PlanningAlgorithm>,
    pub status: WorkerStatus,
    pub last_heartbeat: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub enum WorkerStatus {
    Ready,
    Busy { plan_id: String },
    Offline,
}

pub struct PathPlanningPlannerService {
    planners: HashMap<String, PathPlanner>,
    event_store: Arc<dyn EventStore>,
    last_processed_version: HashMap<String, u64>,
    available_workers: HashMap<String, WorkerInfo>,
}

impl PathPlanningPlannerService {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        println!("🗺️  Starting Path Planning Planner Service (Kafka Event-Driven)");
        println!("📋 Using default configuration for demo");
        
        // Initialize Kafka Event Store
        let event_store = Arc::new(
            KafkaEventStore::new(
                "localhost:9092", 
                "path-planning-events", 
                "planner-group"
            ).await?
        ) as Arc<dyn EventStore>;
        
        println!("✅ Connected to Kafka event store for distributed event communication");
        
        let mut planners = HashMap::new();
        
        // Create new planner (skip event restoration for now to avoid hanging)
        let planner_id = "main-path-planner".to_string();
        let planner = PathPlanner::new(planner_id.clone(), PlanningAlgorithm::AStar);
        planners.insert(planner_id.clone(), planner);
        println!("✅ Created new PathPlanner with A* algorithm");
        
        Ok(Self {
            planners,
            event_store,
            last_processed_version: HashMap::new(),
            available_workers: HashMap::new(),
        })
    }
    
    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🚀 Path Planning Planner Service is running (Kafka Event-Driven)");
        
        // Register a mock worker for demo purposes
        self.register_mock_worker().await;
        
        println!("📡 Polling Kafka for new events...");
        
        // Set up a polling timer for new events from Kafka
        let mut event_poll_timer = interval(Duration::from_millis(500)); // More frequent polling
        let mut heartbeat = interval(Duration::from_secs(30));
        
        loop {
            tokio::select! {
                // Poll for new events from Kafka
                _ = event_poll_timer.tick() => {
                    self.poll_and_process_kafka_events().await?;
                }
                
                // Periodic heartbeat and status update
                _ = heartbeat.tick() => {
                    self.print_status().await;
                }
                
                // Handle shutdown signal
                _ = tokio::signal::ctrl_c() => {
                    println!("🛑 Shutting down Path Planning Planner Service");
                    break;
                }
            }
        }
        
        Ok(())
    }
    
    async fn poll_and_process_kafka_events(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Use dedicated consumer for PathPlanRequested events
        let consumer: rdkafka::consumer::StreamConsumer = rdkafka::ClientConfig::new()
            .set("group.id", "planner-requests-group")
            .set("bootstrap.servers", "localhost:9092")
            .set("enable.partition.eof", "false")
            .set("session.timeout.ms", "6000")
            .set("enable.auto.commit", "true")
            .set("auto.offset.reset", "latest")
            .create()
            .map_err(|e| format!("Failed to create planner consumer: {}", e))?;
        
        consumer.subscribe(&["path-planning-events"])
            .map_err(|e| format!("Failed to subscribe to events: {}", e))?;
        
        // Poll for a short time to get available events
        let timeout = Duration::from_millis(100);
        let start_time = std::time::Instant::now();
        
        while start_time.elapsed() < timeout {
            match tokio::time::timeout(Duration::from_millis(50), consumer.recv()).await {
                Ok(Ok(message)) => {
                    if let Some(payload) = message.payload() {
                        let payload_str = String::from_utf8_lossy(payload);
                        if let Ok(event_envelope) = serde_json::from_str::<EventEnvelope>(&payload_str) {
                            if event_envelope.event_type == "PathPlanRequested" {
                                println!("📥 Found PathPlanRequested event from Kafka: {}", event_envelope.aggregate_id);
                                if let Ok(event_data) = serde_json::from_value::<PathPlanningEvent>(event_envelope.event_data.clone()) {
                                    self.process_event(event_data).await?;
                                }
                            }
                        }
                    }
                }
                _ => break,
            }
        }
        
        Ok(())
    }
    
    async fn process_event(&mut self, event: PathPlanningEvent) -> Result<(), Box<dyn std::error::Error>> {
        match event {
            PathPlanningEvent::PathPlanRequested { 
                planner_id,
                request_id, 
                plan_id, 
                agent_id, 
                start_position, 
                destination_position, 
                start_orientation,
                destination_orientation,
                .. 
            } => {
                println!("🎯 Processing PathPlanRequested event from Kafka:");
                println!("   Request ID: {}", request_id);
                println!("   Plan ID: {}", plan_id);
                println!("   Agent: {}", agent_id);
                println!("   From: ({:.1}, {:.1}) -> To: ({:.1}, {:.1})", 
                         start_position.x, start_position.y,
                         destination_position.x, destination_position.y);
                
                // Try to assign to an available worker
                match self.find_available_worker() {
                    Some(worker_id) => {
                        // Assign to worker with full request data
                        self.assign_plan_to_worker(
                            &plan_id, 
                            &worker_id, 
                            &planner_id,
                            &request_id,
                            &agent_id,
                            &start_position,
                            &destination_position,
                            &start_orientation,
                            &destination_orientation
                        ).await?;
                        
                        // Update worker status
                        if let Some(worker_info) = self.available_workers.get_mut(&worker_id) {
                            worker_info.status = WorkerStatus::Busy { plan_id: plan_id.clone() };
                        }
                        
                        println!("✅ Assigned plan {} to worker {} via Kafka", plan_id, worker_id);
                    }
                    None => {
                        println!("⚠️  No available workers for plan {}. Request queued.", plan_id);
                    }
                }
            }
            
            PathPlanningEvent::WorkerRegistered { worker_id, capabilities, .. } => {
                println!("👷 Worker registered via Kafka: {} with capabilities: {:?}", worker_id, capabilities);
                
                let worker_info = WorkerInfo {
                    worker_id: worker_id.clone(),
                    capabilities,
                    status: WorkerStatus::Ready,
                    last_heartbeat: Utc::now(),
                };
                
                self.available_workers.insert(worker_id, worker_info);
            }
            
            PathPlanningEvent::PlanCompleted { plan_id, worker_id, .. } => {
                println!("🎉 Plan completed via Kafka: {} by worker {:?}", plan_id, worker_id);
                
                // Mark worker as available again
                if let Some(worker_id) = worker_id {
                    if let Some(worker_info) = self.available_workers.get_mut(&worker_id) {
                        worker_info.status = WorkerStatus::Ready;
                    }
                }
            }
            
            _ => {
                println!("📝 Processed other event type from Kafka");
            }
        }
        
        Ok(())
    }
    
    fn find_available_worker(&self) -> Option<String> {
        for (worker_id, worker_info) in &self.available_workers {
            if matches!(worker_info.status, WorkerStatus::Ready) {
                return Some(worker_id.clone());
            }
        }
        None
    }
    
    async fn assign_plan_to_worker(
        &self,
        plan_id: &str,
        worker_id: &str,
        planner_id: &str,
        request_id: &str,
        agent_id: &str,
        start_position: &Position2D,
        destination_position: &Position2D,
        start_orientation: &Orientation2D,
        destination_orientation: &Orientation2D,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let event = PathPlanningEvent::PlanAssigned {
            planner_id: planner_id.to_string(),
            plan_id: plan_id.to_string(),
            worker_id: worker_id.to_string(),
            request_id: request_id.to_string(),
            agent_id: agent_id.to_string(),
            start_position: start_position.clone(),
            destination_position: destination_position.clone(),
            start_orientation: start_orientation.clone(),
            destination_orientation: destination_orientation.clone(),
            timeout_seconds: 300,
            timestamp: Utc::now(),
        };
        
        let event_envelope = EventEnvelope {
            event_id: Uuid::new_v4(),
            aggregate_id: planner_id.to_string(),
            aggregate_type: "PathPlanner".to_string(),
            event_type: event.event_type().to_string(),
            event_version: 1,
            event_data: serde_json::to_value(&event)?,
            metadata: EventMetadata {
                correlation_id: Some(Uuid::new_v4()),
                causation_id: None,
                user_id: Some(worker_id.to_string()),
                source: "pathplan_planner_kafka".to_string(),
            },
            occurred_at: Utc::now(),
        };
        
        self.event_store.append_events(planner_id, 1, vec![event_envelope]).await?;
        println!("📤 Published PlanAssigned event to Kafka for plan {} to worker {}", plan_id, worker_id);
        
        Ok(())
    }
    
    async fn print_status(&self) {
        println!("📊 Kafka Planner Status:");
        println!("   🗺️  Active planners: {}", self.planners.len());
        println!("   👷 Available workers: {}", 
                 self.available_workers.values()
                     .filter(|w| matches!(w.status, WorkerStatus::Ready))
                     .count());
        println!("   🔄 Busy workers: {}", 
                 self.available_workers.values()
                     .filter(|w| matches!(w.status, WorkerStatus::Busy { .. }))
                     .count());
        
        for (worker_id, worker_info) in &self.available_workers {
            match &worker_info.status {
                WorkerStatus::Ready => println!("     ✅ {}: Ready", worker_id),
                WorkerStatus::Busy { plan_id } => println!("     🔄 {}: Working on {}", worker_id, plan_id),
                WorkerStatus::Offline => println!("     ❌ {}: Offline", worker_id),
            }
        }
    }
    
    async fn register_mock_worker(&mut self) {
        let worker_id = "kafka-worker-1".to_string();
        let worker_info = WorkerInfo {
            worker_id: worker_id.clone(),
            capabilities: vec![PlanningAlgorithm::AStar],
            status: WorkerStatus::Ready,
            last_heartbeat: Utc::now(),
        };
        
        self.available_workers.insert(worker_id.clone(), worker_info);
        println!("🤖 Registered mock Kafka worker: {} for demo purposes", worker_id);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut service = PathPlanningPlannerService::new().await?;
    service.run().await
}
