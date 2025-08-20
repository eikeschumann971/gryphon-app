use chrono::Utc;
use gryphon_app::adapters::inbound::kafka_event_store::KafkaEventStore;
use gryphon_app::common::{DomainEvent, EventEnvelope, EventMetadata, EventStore};
use gryphon_app::domains::path_planning::*;
use rdkafka::consumer::Consumer;
use rdkafka::Message;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};
use uuid::Uuid;
#[cfg(feature = "esrs_migration")]
use gryphon_app::adapters::inbound::esrs_pg_store::build_pg_store_with_bus;
#[cfg(feature = "esrs_migration")]
use gryphon_app::adapters::outbound::esrs_kafka_bus::KafkaEventBus;
#[cfg(feature = "esrs_migration")]
use esrs::store::postgres::PgStore;
#[cfg(feature = "esrs_migration")]
use gryphon_app::esrs::PathPlanner as EsrsPathPlanner;
#[cfg(feature = "esrs_migration")]
use esrs::store::EventStore as EsrsEventStore;

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
    #[cfg(feature = "esrs_migration")]
    #[allow(dead_code)]
    esrs_store: Option<PgStore<EsrsPathPlanner>>,
}

impl PathPlanningPlannerService {
    pub async fn new(
        logger: gryphon_app::domains::DynLogger,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Emit structured domain logs for lifecycle events
        logger.info("🗺️  Starting Path Planning Planner Service (Kafka Event-Driven)");
        logger.info("📋 Using default configuration for demo");

        // Initialize Kafka Event Store
        let event_store = Arc::new(
            KafkaEventStore::new("localhost:9092", "path-planning-events", "planner-group").await?,
        ) as Arc<dyn EventStore>;

        logger.info("✅ Connected to Kafka event store for distributed event communication");

        #[cfg(feature = "esrs_migration")]
        logger.info("🔁 esrs migration feature enabled: will attempt to build PgStore + KafkaEventBus");

        #[cfg(feature = "esrs_migration")]
        let esrs_store: Option<PgStore<EsrsPathPlanner>> = build_pg_store_with_bus::<EsrsPathPlanner, _>(
            &std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://postgres:password@127.0.0.1:5432/gryphon_app".to_string()),
            KafkaEventBus::<EsrsPathPlanner>::new("localhost:9092", "path-planning-events"),
        )
        .await
        .map(|s| {
            logger.info("✅ Built esrs PgStore with KafkaEventBus");
            Some(s)
        })
        .unwrap_or_else(|e| {
            logger.warn(&format!("⚠️ Failed to build esrs PgStore: {}", e));
            None
        });

        let mut planners = HashMap::new();

        // Create new planner (skip event restoration for now to avoid hanging)
        let planner_id = "main-path-planner".to_string();
        let planner = PathPlanner::new(planner_id.clone(), PlanningAlgorithm::AStar);
        planners.insert(planner_id.clone(), planner);
        logger.info("✅ Created new PathPlanner with A* algorithm");

        Ok(Self {
            planners,
            event_store,
            last_processed_version: HashMap::new(),
            available_workers: HashMap::new(),
            logger,
            #[cfg(feature = "esrs_migration")]
            esrs_store,
        })
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🚀 Path Planning Planner Service is running (Kafka Event-Driven)");
        self.logger
            .info("🚀 Path Planning Planner Service is running (Kafka Event-Driven)");

        // Workers will register themselves via Kafka events
        // No mock workers in production!

        self.logger.info("📡 Polling Kafka for new events");

        // Set up a dedicated consumer task for event-driven processing
        let consumer: rdkafka::consumer::StreamConsumer = rdkafka::ClientConfig::new()
            .set("group.id", "planner-requests-group")
            .set("bootstrap.servers", "localhost:9092")
            .set("enable.partition.eof", "false")
            .set("session.timeout.ms", "6000")
            .set("enable.auto.commit", "true")
            .set("auto.offset.reset", "earliest") // Read from beginning to catch worker registrations
            .create()
            .map_err(|e| format!("Failed to create planner consumer: {}", e))?;

        consumer.subscribe(&["path-planning-events"])?;

        let logger = self.logger.clone();
        let mut health_check_timer = interval(Duration::from_secs(60));

        // Create a reply event store so we can publish responses to a shared replies topic
        let reply_store: Arc<dyn EventStore> = Arc::new(
            KafkaEventStore::new(
                "localhost:9092",
                "path-planning-replies",
                "planner-reply-publisher",
            )
            .await?,
        );

        // Channel to receive EventEnvelope from consumer task
        let (tx, mut rx) = mpsc::channel::<EventEnvelope>(128);

        // Spawn consumer task that pushes EventEnvelopes into the mpsc channel
        let tx_clone = tx.clone();
        let logger_clone = logger.clone();
        tokio::spawn(async move {
            let recv_consumer = consumer;
            loop {
                match recv_consumer.recv().await {
                    Ok(message) => {
                        if let Some(payload) = message.payload() {
                            if let Ok(event_envelope) =
                                serde_json::from_slice::<EventEnvelope>(payload)
                            {
                                let _ = tx_clone.send(event_envelope).await;
                            }
                        }
                    }
                    Err(e) => {
                        logger_clone.warn(&format!("Planner consumer error: {}", e));
                        tokio::time::sleep(Duration::from_millis(200)).await;
                    }
                }
            }
        });

        // Main loop: process incoming EventEnvelopes and periodic tasks
        loop {
            tokio::select! {
                Some(envelope) = rx.recv() => {
                    // Process the envelope and optionally publish replies using reply_store
                    self.logger.info(&format!("Received envelope in planner: {} for aggregate {}", envelope.event_type, envelope.aggregate_id));
                    if envelope.event_type == "PathPlanRequested" {
                        self.logger.info("Planner received PathPlanRequested - attempting assignment");
                    }
                    if let Err(e) = self.process_event_envelope(envelope, Some(reply_store.clone())).await {
                        self.logger.warn(&format!("Failed to process envelope: {}", e));
                    }
                }
                _ = health_check_timer.tick() => {
                    self.check_worker_health().await?;
                    self.print_status().await;
                }
                _ = tokio::signal::ctrl_c() => {
                    println!("🛑 Shutting down Path Planning Planner Service");
                    break;
                }
            }
        }

        Ok(())
    }

    async fn process_event(
        &mut self,
        event: PathPlanningEvent,
    ) -> Result<(), Box<dyn std::error::Error>> {
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
                self.logger.info(&format!("🎯 Processing PathPlanRequested event: request_id={} plan_id={} agent={} from=({:.1},{:.1}) to=({:.1},{:.1})", 
                    request_id, plan_id, agent_id,
                    start_position.x, start_position.y,
                    destination_position.x, destination_position.y));

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
                            &destination_orientation,
                        )
                        .await?;

                        // Update worker status
                        if let Some(worker_info) = self.available_workers.get_mut(&worker_id) {
                            worker_info.status = WorkerStatus::Busy {
                                plan_id: plan_id.clone(),
                            };
                        }

                        println!(
                            "✅ Assigned plan {} to worker {} via Kafka",
                            plan_id, worker_id
                        );
                        self.logger.info(&format!(
                            "Assigned plan {} to worker {} via Kafka",
                            plan_id, worker_id
                        ));
                    }
                    None => {
                        println!(
                            "⚠️  No available workers for plan {}. Request queued.",
                            plan_id
                        );
                    }
                }
            }

            PathPlanningEvent::WorkerRegistered {
                worker_id,
                capabilities,
                ..
            } => {
                println!(
                    "👷 Worker registered via Kafka: {} with capabilities: {:?}",
                    worker_id, capabilities
                );
                self.logger.info(&format!(
                    "Worker registered via Kafka: {} capabilities={:?}",
                    worker_id, capabilities
                ));

                let worker_info = WorkerInfo {
                    worker_id: worker_id.clone(),
                    capabilities,
                    status: WorkerStatus::Ready,
                    last_heartbeat: Utc::now(),
                };

                self.available_workers
                    .insert(worker_id.clone(), worker_info);
                println!("✅ Worker {} added to available workers list", worker_id);
                self.logger.info(&format!(
                    "Worker {} added to available workers list",
                    worker_id
                ));
            }

            PathPlanningEvent::WorkerReady { worker_id, .. } => {
                println!("✅ Worker ready via Kafka: {}", worker_id);
                self.logger
                    .info(&format!("Worker ready via Kafka: {}", worker_id));

                // Update worker status to ready if it exists
                if let Some(worker_info) = self.available_workers.get_mut(&worker_id) {
                    worker_info.status = WorkerStatus::Ready;
                    worker_info.last_heartbeat = Utc::now();
                    println!("✅ Updated worker {} status to Ready", worker_id);
                    self.logger
                        .info(&format!("Updated worker {} status to Ready", worker_id));
                } else {
                    println!(
                        "⚠️ Worker {} not found in available workers list",
                        worker_id
                    );
                }
            }

            PathPlanningEvent::WorkerHeartbeat {
                worker_id,
                timestamp,
                ..
            } => {
                // Update last heartbeat timestamp
                if let Some(worker_info) = self.available_workers.get_mut(&worker_id) {
                    worker_info.last_heartbeat = timestamp;
                    println!(
                        "💓 Received heartbeat from worker {} at {}",
                        worker_id,
                        timestamp.format("%H:%M:%S")
                    );
                    self.logger.info(&format!(
                        "Received heartbeat from worker {} at {}",
                        worker_id,
                        timestamp.format("%H:%M:%S")
                    ));
                } else {
                    println!("⚠️ Received heartbeat from unknown worker: {}", worker_id);
                }
            }

            PathPlanningEvent::WorkerOffline {
                worker_id, reason, ..
            } => {
                println!("❌ Worker {} went offline: {}", worker_id, reason);
                self.logger
                    .warn(&format!("Worker {} went offline: {}", worker_id, reason));

                // Mark worker as offline
                if let Some(worker_info) = self.available_workers.get_mut(&worker_id) {
                    worker_info.status = WorkerStatus::Offline;
                    println!("✅ Marked worker {} as offline", worker_id);
                } else {
                    println!(
                        "⚠️ Worker {} not found in available workers list",
                        worker_id
                    );
                }
            }

            PathPlanningEvent::PlanCompleted {
                plan_id, worker_id, ..
            } => {
                println!(
                    "🎉 Plan completed via Kafka: {} by worker {:?}",
                    plan_id, worker_id
                );
                self.logger.info(&format!(
                    "Plan completed via Kafka: {} by worker {:?}",
                    plan_id, worker_id
                ));

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

    // New higher-level helper to process raw EventEnvelope and optionally publish replies
    async fn process_event_envelope(
        &mut self,
        envelope: EventEnvelope,
        reply_store: Option<Arc<dyn EventStore>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Convert envelope.event_data to PathPlanningEvent and call process_event
        if let Ok(event) = serde_json::from_value::<PathPlanningEvent>(envelope.event_data.clone())
        {
            match &event {
                PathPlanningEvent::PathPlanRequested { plan_id, .. } => {
                    // When assigning a plan, publish PlanAssigned to both main topic and replies
                    if let Some(worker_id) = self.find_available_worker() {
                        // preserve the incoming request's correlation id so the client can match replies
                        let corr = envelope.metadata.correlation_id;
                        let reply_envelope =
                            self.build_plan_assigned_envelope(plan_id, &worker_id, corr)?;
                        // persist and publish to main topic
                        self.event_store
                            .append_events(
                                &reply_envelope.aggregate_id,
                                1,
                                vec![reply_envelope.clone()],
                            )
                            .await?;
                        // also publish to replies topic if provided (copying correlation id)
                        if let Some(store) = reply_store {
                            // append a cloned copy to replies topic
                            let en = reply_envelope.clone();
                            let en_agg = en.aggregate_id.clone();
                            store.append_events(&en_agg, 1, vec![en]).await?;
                        }
                    }
                }
                _ => {
                    // Default: just call process_event for side-effects
                    self.process_event(event).await?;
                }
            }
        }
        Ok(())
    }

    fn build_plan_assigned_envelope(
        &self,
        plan_id: &str,
        worker_id: &str,
        correlation_id: Option<uuid::Uuid>,
    ) -> Result<EventEnvelope, Box<dyn std::error::Error>> {
        // Build minimal PlanAssigned event envelope copying planner_id
        let event = PathPlanningEvent::PlanAssigned {
            planner_id: "main-path-planner".to_string(),
            plan_id: plan_id.to_string(),
            worker_id: worker_id.to_string(),
            request_id: "".to_string(),
            agent_id: "".to_string(),
            start_position: Position2D { x: 0.0, y: 0.0 },
            destination_position: Position2D { x: 0.0, y: 0.0 },
            start_orientation: Orientation2D { angle: 0.0 },
            destination_orientation: Orientation2D { angle: 0.0 },
            timeout_seconds: 300,
            timestamp: Utc::now(),
        };

        let envelope = EventEnvelope {
            event_id: Uuid::new_v4(),
            aggregate_id: "main-path-planner".to_string(),
            aggregate_type: "PathPlanner".to_string(),
            event_type: event.event_type().to_string(),
            event_version: 1,
            event_data: serde_json::to_value(&event)?,
            metadata: EventMetadata {
                correlation_id: correlation_id.or(Some(Uuid::new_v4())),
                causation_id: None,
                user_id: Some(worker_id.to_string()),
                source: "pathplan_planner_kafka".to_string(),
            },
            occurred_at: Utc::now(),
        };

        Ok(envelope)
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

        self.event_store
            .append_events(planner_id, 1, vec![event_envelope])
            .await?;
        self.logger.info(&format!(
            "📤 Published PlanAssigned event to Kafka for plan {} to worker {}",
            plan_id, worker_id
        ));

        // Mirror event to esrs PgStore if the esrs migration feature is enabled and store exists
        #[cfg(feature = "esrs_migration")]
        if let Some(store) = &self.esrs_store {
            // Build an AggregateState for the esrs PathPlanner and persist the domain event
            use esrs::AggregateState;
            let mut agg_state = AggregateState::<gryphon_app::esrs::path_planning::PathPlannerState>::with_id(
                gryphon_app::adapters::inbound::esrs_pg_store::uuid_for_aggregate_id(&planner_id),
            );
            // Deserialize the domain event and persist via esrs store
            if let Ok(evt) = serde_json::from_value::<gryphon_app::domains::path_planning::events::PathPlanningEvent>(serde_json::to_value(&event).unwrap()) {
                let events = vec![evt];
                // Call the EsrsEventStore::persist method
                let _ = EsrsEventStore::persist(store, &mut agg_state, events).await;
            }
        }

        Ok(())
    }

    async fn print_status(&self) {
        self.logger.info(&format!(
            "📊 Kafka Planner Status: active_planners={} available_workers={} busy_workers={}",
            self.planners.len(),
            self.available_workers
                .values()
                .filter(|w| matches!(w.status, WorkerStatus::Ready))
                .count(),
            self.available_workers
                .values()
                .filter(|w| matches!(w.status, WorkerStatus::Busy { .. }))
                .count()
        ));

        for (worker_id, worker_info) in &self.available_workers {
            match &worker_info.status {
                WorkerStatus::Ready => self.logger.info(&format!("     ✅ {}: Ready", worker_id)),
                WorkerStatus::Busy { plan_id } => self
                    .logger
                    .info(&format!("     🔄 {}: Working on {}", worker_id, plan_id)),
                WorkerStatus::Offline => {
                    self.logger.info(&format!("     ❌ {}: Offline", worker_id))
                }
            }
        }
    }

    async fn check_worker_health(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let now = Utc::now();
        let heartbeat_timeout = chrono::Duration::seconds(90); // 90 seconds timeout (3 missed heartbeats)
        let mut workers_to_mark_offline = Vec::new();

        for (worker_id, worker_info) in &self.available_workers {
            let time_since_heartbeat = now - worker_info.last_heartbeat;

            if time_since_heartbeat > heartbeat_timeout
                && !matches!(worker_info.status, WorkerStatus::Offline)
            {
                self.logger.warn(&format!(
                    "⚠️  Worker {} hasn't sent heartbeat for {} seconds, marking as offline",
                    worker_id,
                    time_since_heartbeat.num_seconds()
                ));
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

                self.event_store
                    .append_events("main-path-planner", 1, vec![event_envelope])
                    .await?;
                self.logger.info(&format!(
                    "📤 Published WorkerOffline event for worker {}",
                    worker_id
                ));

                #[cfg(feature = "esrs_migration")]
                if let Some(store) = &self.esrs_store {
                    use esrs::AggregateState;
                    let mut agg_state = AggregateState::<gryphon_app::esrs::path_planning::PathPlannerState>::with_id(
                        gryphon_app::adapters::inbound::esrs_pg_store::uuid_for_aggregate_id("main-path-planner"),
                    );
                    if let Ok(evt) = serde_json::from_value::<gryphon_app::domains::path_planning::events::PathPlanningEvent>(serde_json::to_value(&offline_event).unwrap()) {
                        let _ = EsrsEventStore::persist(store, &mut agg_state, vec![evt]).await;
                    }
                }
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
