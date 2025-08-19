use gryphon_app::domains::path_planning::*;
use gryphon_app::adapters::inbound::kafka_event_store::KafkaEventStore;
use gryphon_app::common::{EventStore, EventEnvelope, EventMetadata, DomainEvent};
use tokio::time::{interval, Duration};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::Utc;
use std::sync::Arc;
use rdkafka::consumer::Consumer;
use rdkafka::Message;

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
    #[allow(dead_code)]
    last_processed_version: HashMap<String, u64>,
    available_workers: HashMap<String, WorkerInfo>,
    logger: gryphon_app::domains::DynLogger,
}

impl PathPlanningPlannerService {
    pub async fn new(logger: gryphon_app::domains::DynLogger) -> Result<Self, Box<dyn std::error::Error>> {
        // keep println for console UX, but also emit structured domain logs
        println!("🗺️  Starting Path Planning Planner Service (Kafka Event-Driven)");
        logger.info("Starting Path Planning Planner Service (Kafka Event-Driven)");
        println!("📋 Using default configuration for demo");
        logger.info("Using default configuration for demo");

        // Initialize Kafka Event Store
        let event_store = Arc::new(
            KafkaEventStore::new(
                "localhost:9092",
                "path-planning-events",
                "planner-group"
            ).await?
        ) as Arc<dyn EventStore>;

        println!("✅ Connected to Kafka event store for distributed event communication");
        logger.info("Connected to Kafka event store for distributed event communication");

        let mut planners = HashMap::new();

        // Create new planner (skip event restoration for now to avoid hanging)
        let planner_id = "main-path-planner".to_string();
        let planner = PathPlanner::new(planner_id.clone(), PlanningAlgorithm::AStar);
        planners.insert(planner_id.clone(), planner);
        println!("✅ Created new PathPlanner with A* algorithm");
        logger.info("Created new PathPlanner with A* algorithm");

        Ok(Self {
            planners,
            event_store,
            last_processed_version: HashMap::new(),
            available_workers: HashMap::new(),
            logger,
        })
    }
    
    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Path Planning Planner Service is running (Kafka Event-Driven)");
    self.logger.info("Path Planning Planner Service is running (Kafka Event-Driven)");
        
        // Workers will register themselves via Kafka events
        // No mock workers in production!
        
    println!("📡 Polling Kafka for new events...");
    self.logger.info("Polling Kafka for new events");
        
        // Set up a polling timer for new events from Kafka
        let mut event_poll_timer = interval(Duration::from_millis(500)); // More frequent polling
        let mut heartbeat = interval(Duration::from_secs(30));
        let mut health_check_timer = interval(Duration::from_secs(60)); // Check worker health every minute
        
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
                
                // Check worker health and mark stale workers as offline
                _ = health_check_timer.tick() => {
                    self.check_worker_health().await?;
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
        // Use dedicated consumer for events with better offset management
        let consumer: rdkafka::consumer::StreamConsumer = rdkafka::ClientConfig::new()
            .set("group.id", "planner-requests-group")
            .set("bootstrap.servers", "localhost:9092")
            .set("enable.partition.eof", "false")
            .set("session.timeout.ms", "6000")
            .set("enable.auto.commit", "true")
            .set("auto.offset.reset", "earliest") // Read from beginning to catch worker registrations
            .create()
            .map_err(|e| format!("Failed to create planner consumer: {}", e))?;
        
        consumer.subscribe(&["path-planning-events"])
            .map_err(|e| format!("Failed to subscribe to events: {}", e))?;
        
        // Poll for a longer time to process all available events including worker registrations
        let timeout = Duration::from_millis(2000); // Increased from 100ms to 2 seconds
        let start_time = std::time::Instant::now();
        
        while start_time.elapsed() < timeout {
            match tokio::time::timeout(Duration::from_millis(500), consumer.recv()).await {
                Ok(Ok(message)) => {
                    if let Some(payload) = message.payload() {
                        let payload_str = String::from_utf8_lossy(payload);
                        if let Ok(event_envelope) = serde_json::from_str::<EventEnvelope>(&payload_str) {
                            // Process different event types
                            match event_envelope.event_type.as_str() {
                                "PathPlanRequested" => {
                                                    println!("📥 Found PathPlanRequested event from Kafka: {}", event_envelope.aggregate_id);
                                                    self.logger.info(&format!("Found PathPlanRequested event from Kafka: {}", event_envelope.aggregate_id));
                                    if let Ok(event_data) = serde_json::from_value::<PathPlanningEvent>(event_envelope.event_data.clone()) {
                                        self.process_event(event_data).await?;
                                    }
                                }
                                "WorkerRegistered" | "WorkerReady" | "WorkerHeartbeat" | "WorkerOffline" => {
                                    println!("📥 Found worker event: {} from Kafka for aggregate {}", event_envelope.event_type, event_envelope.aggregate_id);
                                    self.logger.info(&format!("Found worker event: {} from Kafka for aggregate {}", event_envelope.event_type, event_envelope.aggregate_id));
                                    if let Ok(event_data) = serde_json::from_value::<PathPlanningEvent>(event_envelope.event_data.clone()) {
                                        self.process_event(event_data).await?;
                                    } else {
                                        println!("⚠️ Failed to deserialize worker event");
                                    }
                                }
                                _ => {
                                    // Ignore other events
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
                self.logger.info(&format!("Processing PathPlanRequested event: request_id={}, plan_id={}, agent={}", request_id, plan_id, agent_id));
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
                        self.logger.info(&format!("Assigned plan {} to worker {} via Kafka", plan_id, worker_id));
                    }
                    None => {
                        println!("⚠️  No available workers for plan {}. Request queued.", plan_id);
                    }
                }
            }
            
            PathPlanningEvent::WorkerRegistered { worker_id, capabilities, .. } => {
                println!("👷 Worker registered via Kafka: {} with capabilities: {:?}", worker_id, capabilities);
                self.logger.info(&format!("Worker registered via Kafka: {} capabilities={:?}", worker_id, capabilities));
                
                let worker_info = WorkerInfo {
                    worker_id: worker_id.clone(),
                    capabilities,
                    status: WorkerStatus::Ready,
                    last_heartbeat: Utc::now(),
                };
                
                self.available_workers.insert(worker_id.clone(), worker_info);
                println!("✅ Worker {} added to available workers list", worker_id);
                self.logger.info(&format!("Worker {} added to available workers list", worker_id));
            }
            
            PathPlanningEvent::WorkerReady { worker_id, .. } => {
                println!("✅ Worker ready via Kafka: {}", worker_id);
                self.logger.info(&format!("Worker ready via Kafka: {}", worker_id));
                
                // Update worker status to ready if it exists
                if let Some(worker_info) = self.available_workers.get_mut(&worker_id) {
                    worker_info.status = WorkerStatus::Ready;
                    worker_info.last_heartbeat = Utc::now();
                    println!("✅ Updated worker {} status to Ready", worker_id);
                    self.logger.info(&format!("Updated worker {} status to Ready", worker_id));
                } else {
                    println!("⚠️ Worker {} not found in available workers list", worker_id);
                }
            }
            
            PathPlanningEvent::WorkerHeartbeat { worker_id, timestamp, .. } => {
                // Update last heartbeat timestamp
                if let Some(worker_info) = self.available_workers.get_mut(&worker_id) {
                    worker_info.last_heartbeat = timestamp;
                    println!("💓 Received heartbeat from worker {} at {}", worker_id, timestamp.format("%H:%M:%S"));
                    self.logger.info(&format!("Received heartbeat from worker {} at {}", worker_id, timestamp.format("%H:%M:%S")));
                } else {
                    println!("⚠️ Received heartbeat from unknown worker: {}", worker_id);
                }
            }
            
            PathPlanningEvent::WorkerOffline { worker_id, reason, .. } => {
                println!("❌ Worker {} went offline: {}", worker_id, reason);
                self.logger.warn(&format!("Worker {} went offline: {}", worker_id, reason));
                
                // Mark worker as offline
                if let Some(worker_info) = self.available_workers.get_mut(&worker_id) {
                    worker_info.status = WorkerStatus::Offline;
                    println!("✅ Marked worker {} as offline", worker_id);
                } else {
                    println!("⚠️ Worker {} not found in available workers list", worker_id);
                }
            }
            
            PathPlanningEvent::PlanCompleted { plan_id, worker_id, .. } => {
                println!("🎉 Plan completed via Kafka: {} by worker {:?}", plan_id, worker_id);
                self.logger.info(&format!("Plan completed via Kafka: {} by worker {:?}", plan_id, worker_id));
                
                // Mark worker as available again
                if let Some(worker_id) = worker_id {
                    if let Some(worker_info) = self.available_workers.get_mut(&worker_id) {
                        worker_info.status = WorkerStatus::Ready;
                    }
                }
            }
            
            _ => {
                println!("📝 Processed other event type from Kafka");
                self.logger.info("Processed other event type from Kafka");
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
    
    #[allow(clippy::too_many_arguments)]
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
    
    async fn check_worker_health(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let now = Utc::now();
        let heartbeat_timeout = chrono::Duration::seconds(90); // 90 seconds timeout (3 missed heartbeats)
        let mut workers_to_mark_offline = Vec::new();
        
        for (worker_id, worker_info) in &self.available_workers {
            let time_since_heartbeat = now - worker_info.last_heartbeat;
            
            if time_since_heartbeat > heartbeat_timeout && !matches!(worker_info.status, WorkerStatus::Offline) {
                println!("⚠️  Worker {} hasn't sent heartbeat for {} seconds, marking as offline", 
                        worker_id, time_since_heartbeat.num_seconds());
                workers_to_mark_offline.push(worker_id.clone());
            }
        }
        
        // Mark stale workers as offline and publish WorkerOffline events
        for worker_id in workers_to_mark_offline {
            if let Some(worker_info) = self.available_workers.get_mut(&worker_id) {
                worker_info.status = WorkerStatus::Offline;
                
                // Publish WorkerOffline event to Kafka
                let offline_event = PathPlanningEvent::WorkerOffline {
                    planner_id: "main-path-planner".to_string(),
                    worker_id: worker_id.clone(),
                    reason: "Heartbeat timeout".to_string(),
                    timestamp: now,
                };
                
                let event_envelope = EventEnvelope {
                    event_id: Uuid::new_v4(),
                    aggregate_id: "main-path-planner".to_string(),
                    aggregate_type: "PathPlanner".to_string(),
                    event_type: offline_event.event_type().to_string(),
                    event_version: 1,
                    event_data: serde_json::to_value(&offline_event)?,
                    metadata: EventMetadata {
                        correlation_id: None,
                        causation_id: None,
                        user_id: Some("health_monitor".to_string()),
                        source: "pathplan_planner_kafka".to_string(),
                    },
                    occurred_at: now,
                };
                
                self.event_store.append_events("main-path-planner", 1, vec![event_envelope]).await?;
                println!("📤 Published WorkerOffline event for worker {}", worker_id);
            }
        }
        
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize combined logger (file + console fallback)
    let logger = gryphon_app::adapters::outbound::init_combined_logger("./domain.log");
    logger.info("Starting Path Planning Planner Service (Kafka Event-Driven)");

    let mut service = PathPlanningPlannerService::new(logger.clone()).await?;
    service.run().await
}
