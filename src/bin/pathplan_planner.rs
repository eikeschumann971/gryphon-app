use chrono::Utc;
use gryphon_app::adapters::inbound::file_event_store::FileEventStore;
use gryphon_app::common::{DomainEvent, EventEnvelope, EventMetadata, EventStore};
use gryphon_app::config::Config;
use gryphon_app::domains::path_planning::*;
use gryphon_app::domains::DynLogger;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::time::{interval, Duration};
use uuid::Uuid;
use gryphon_app::adapters::inbound::esrs_pg_store::build_pg_store_with_bus;
use gryphon_app::adapters::outbound::esrs_kafka_bus::KafkaEventBus;
use gryphon_app::esrs::path_planning::PathPlanner as EsrsPathPlanner;
// esrs EventStore trait is used via fully-qualified paths in this module

/// Path Planning Planner Process (Event-Driven)
///
/// This process manages PathPlanner aggregates using event sourcing and coordinates
/// between clients and workers through the event store.
/// It handles:
/// - Loading PathPlanner state from event store
/// - Processing PathPlanRequested events
/// - Publishing WorkerAssignment events
/// - Managing worker registrations through events
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🗺️  Starting Path Planning Planner Service (Event-Driven)");
    // Initialize combined logger (file + console fallback)
    let logger = gryphon_app::adapters::outbound::init_combined_logger("./domain.log");
    logger.info("Starting Path Planning Planner Service (Event-Driven)");

    // Tracing/global logger initialization is handled by the injected DomainLogger adapters.

    let mut planner_service = PathPlannerService::new(logger.clone()).await?;
    planner_service.run().await?;

    Ok(())
}

pub struct PathPlannerService {
    planners: HashMap<String, PathPlanner>,
    event_store: Arc<dyn EventStore>,
    last_processed_version: HashMap<String, u64>,
    available_workers: HashMap<String, WorkerInfo>,
    logger: DynLogger,
    esrs_store: Option<esrs::store::postgres::PgStore<EsrsPathPlanner>>,
}

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

#[derive(Debug, Clone)]
pub enum WorkerEvent {
    WorkerRegistered {
        worker_id: String,
        capabilities: Vec<PlanningAlgorithm>,
    },
    WorkerReady {
        worker_id: String,
    },
    PlanAssignmentAccepted {
        worker_id: String,
        plan_id: String,
    },
    PlanCompleted {
        worker_id: String,
        plan_id: String,
        waypoints: Vec<Position2D>,
    },
    PlanFailed {
        worker_id: String,
        plan_id: String,
        reason: String,
    },
}

#[derive(Debug, Clone)]
pub struct PlanResponse {
    pub request_id: String,
    pub plan_id: String,
    pub status: PlanResponseStatus,
    pub waypoints: Option<Vec<Position2D>>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone)]
pub enum PlanResponseStatus {
    Accepted,
    InProgress,
    Completed,
    Failed,
}

impl PathPlannerService {
    pub async fn new(logger: DynLogger) -> Result<Self, Box<dyn std::error::Error>> {
        // For demo purposes, use default config and in-memory event store
        let _config = Config::default();
        println!("📋 Using default configuration for demo");

        // Initialize event store - use file-based store for demo so all processes can share events
        let event_store: Arc<dyn EventStore> = Arc::new(FileEventStore::new("/tmp/gryphon-events"));
        println!("✅ Using file-based event store for demo (shared between processes)");

    let mut planners = HashMap::new();
    let planner_id = "main-path-planner".to_string();

    // We may create a new planner during initialization — capture its creation event for later mirroring
    let mut creation_event_opt: Option<PathPlanningEvent> = None;

    // Try to restore planner state from event store
        match event_store.load_events(&planner_id, 0).await {
            Ok(events) => {
            if events.is_empty() {
                    // No existing events, create new planner and persist creation event
                    let planner = PathPlanner::new(planner_id.clone(), PlanningAlgorithm::AStar);

                    let creation_event = PathPlanningEvent::PlannerCreated {
                        planner_id: planner_id.clone(),
                        algorithm: PlanningAlgorithm::AStar,
                        timestamp: Utc::now(),
                    };

                    let event_envelope = EventEnvelope {
                        event_id: Uuid::new_v4(),
                        aggregate_id: planner_id.clone(),
                        aggregate_type: "PathPlanner".to_string(),
                        event_type: creation_event.event_type().to_string(),
                        event_version: 1,
                        event_data: serde_json::to_value(&creation_event)?,
                        metadata: EventMetadata {
                            correlation_id: Some(Uuid::new_v4()),
                            causation_id: None,
                            user_id: None,
                            source: "pathplan_planner".to_string(),
                        },
                        occurred_at: Utc::now(),
                    };

                    event_store
                        .append_events(&planner_id, 0, vec![event_envelope])
                        .await?;
                    // capture for later mirroring once esrs_store is available
                    creation_event_opt = Some(creation_event.clone());
                    planners.insert(planner_id.clone(), planner);
                    println!(
                        "✅ Created new PathPlanner with A* algorithm and persisted creation event"
                    );
                } else {
                    // Restore from events
                    let planner = PathPlanner::new(planner_id.clone(), PlanningAlgorithm::AStar);
                    // TODO: In a full implementation, we would replay events to restore state
                    planners.insert(planner_id.clone(), planner);
                    println!("✅ Restored PathPlanner from {} events", events.len());
                }
            }
            Err(e) => {
                println!("⚠️  Failed to load events: {}. Creating new planner", e);
                let planner = PathPlanner::new(planner_id.clone(), PlanningAlgorithm::AStar);
                planners.insert(planner_id.clone(), planner);
            }
        }

        let esrs_store = {
            let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://postgres:password@127.0.0.1:5432/gryphon_app".to_string());
            let kafka_brokers = std::env::var("KAFKA_BROKERS").unwrap_or_else(|_| "localhost:9092".to_string());
            match build_pg_store_with_bus::<EsrsPathPlanner, _>(&database_url, KafkaEventBus::<EsrsPathPlanner>::new(&kafka_brokers, "path-planning-events")).await {
                Ok(s) => Some(s),
                Err(e) => {
                    logger.warn(&format!("Failed to build esrs PgStore for planner mirroring: {}", e));
                    None
                }
            }
        };

        // If we created a planner above and we have an esrs_store, persist the creation event into esrs
        if let (Some(store), Some(evt)) = (esrs_store.as_ref(), creation_event_opt) {
            if let Ok(evt_parsed) = serde_json::from_value::<PathPlanningEvent>(serde_json::to_value(&evt).unwrap()) {
                let agg_uuid = gryphon_app::adapters::inbound::esrs_pg_store::uuid_for_aggregate_id(&planner_id);
                // Try to fetch last sequence to avoid duplicate inserts
                let mut agg_state = esrs::AggregateState::<gryphon_app::esrs::path_planning::PathPlannerState>::with_id(agg_uuid);
                match gryphon_app::adapters::inbound::esrs_pg_store::agg_last_sequence(&agg_uuid).await {
                    Ok(Some(n)) if n >= 1 => {
                        println!("⤴️ esrs pre-check: planner creation event already present (seq={}), skipping persist", n);
                    }
                    _ => {
                        let _ = gryphon_app::adapters::inbound::esrs_pg_store::persist_best_effort(store, &mut agg_state, vec![evt_parsed]).await;
                    }
                }
            }
        }

        Ok(Self {
            planners,
            event_store,
            last_processed_version: HashMap::new(),
            available_workers: HashMap::new(),
            logger,
            esrs_store: None,
        })
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🚀 Path Planning Planner Service is running (Event-Driven)");
        self.logger
            .info("Path Planning Planner Service is running (Event-Driven)");

        // For demo purposes, register a mock worker immediately
        self.register_mock_worker().await;

        println!("📡 Polling event store for new events...");

        // Set up a polling timer for new events
        let mut event_poll_timer = interval(Duration::from_secs(2));
        let mut heartbeat = interval(Duration::from_secs(30));

        loop {
            tokio::select! {
                // Poll for new events from event store
                _ = event_poll_timer.tick() => {
                    self.poll_and_process_events().await?;
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

    async fn poll_and_process_events(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let planner_ids: Vec<String> = self.planners.keys().cloned().collect();

        for planner_id in planner_ids {
            let last_version = self.last_processed_version.get(&planner_id).unwrap_or(&0);

            match self
                .event_store
                .load_events(&planner_id, *last_version)
                .await
            {
                Ok(events) => {
                    if !events.is_empty() {
                        println!(
                            "📥 Found {} new events for planner {}",
                            events.len(),
                            planner_id
                        );
                        self.logger.info(&format!(
                            "Found {} new events for planner {}",
                            events.len(),
                            planner_id
                        ));

                        for event_envelope in events {
                            self.process_event(&planner_id, &event_envelope).await?;

                            // Update last processed version
                            self.last_processed_version
                                .insert(planner_id.clone(), event_envelope.event_version);
                        }
                    }
                }
                Err(e) => {
                    println!(
                        "⚠️  Failed to load events for planner {}: {}",
                        planner_id, e
                    );
                    self.logger.warn(&format!(
                        "Failed to load events for planner {}: {}",
                        planner_id, e
                    ));
                }
            }
        }

        Ok(())
    }

    async fn process_event(
        &mut self,
        planner_id: &str,
        event_envelope: &EventEnvelope,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Deserialize the event
        let event: PathPlanningEvent = serde_json::from_value(event_envelope.event_data.clone())?;

        match event {
            PathPlanningEvent::PathPlanRequested {
                request_id,
                plan_id,
                agent_id,
                start_position,
                destination_position,
                start_orientation,
                destination_orientation,
                timestamp: _timestamp,
                ..
            } => {
                println!("🎯 Processing PathPlanRequested event:");
                println!("   Request ID: {}", request_id);
                println!("   Plan ID: {}", plan_id);
                println!("   Agent: {}", agent_id);
                println!(
                    "   From: ({:.1}, {:.1}) -> To: ({:.1}, {:.1})",
                    start_position.x,
                    start_position.y,
                    destination_position.x,
                    destination_position.y
                );

                // Try to assign to an available worker
                match self.find_available_worker() {
                    Some(worker_id) => {
                        // Assign to worker with full request data
                        self.assign_plan_to_worker(
                            &plan_id,
                            &worker_id,
                            planner_id,
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

                        println!("✅ Assigned plan {} to worker {}", plan_id, worker_id);
                    }
                    None => {
                        println!(
                            "⚠️  No available workers for plan {}. Request queued.",
                            plan_id
                        );
                        // In a real system, we would queue the request
                    }
                }
            }

            PathPlanningEvent::WorkerRegistered {
                worker_id,
                capabilities,
                ..
            } => {
                println!(
                    "👷 Worker registered: {} with capabilities: {:?}",
                    worker_id, capabilities
                );

                let worker_info = WorkerInfo {
                    worker_id: worker_id.clone(),
                    capabilities,
                    status: WorkerStatus::Ready,
                    last_heartbeat: Utc::now(),
                };

                self.available_workers.insert(worker_id, worker_info);
            }

            PathPlanningEvent::PlanCompleted {
                plan_id,
                waypoints,
                worker_id,
                ..
            } => {
                println!(
                    "🎉 Plan completed: {} by worker {:?} with {} waypoints",
                    plan_id,
                    worker_id,
                    waypoints.len()
                );

                // Mark worker as ready again
                if let Some(worker_id) = worker_id {
                    if let Some(worker_info) = self.available_workers.get_mut(&worker_id) {
                        worker_info.status = WorkerStatus::Ready;
                        worker_info.last_heartbeat = Utc::now();
                    }
                }
            }

            PathPlanningEvent::PlanFailed {
                plan_id,
                reason,
                worker_id,
                ..
            } => {
                println!(
                    "❌ Plan failed: {} by worker {:?}. Reason: {}",
                    plan_id, worker_id, reason
                );

                // Mark worker as ready again
                if let Some(worker_id) = worker_id {
                    if let Some(worker_info) = self.available_workers.get_mut(&worker_id) {
                        worker_info.status = WorkerStatus::Ready;
                        worker_info.last_heartbeat = Utc::now();
                    }
                }
            }

            _ => {
                // Handle other events as needed
                println!("📝 Processed event: {}", event_envelope.event_type);
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
            timeout_seconds: 300, // 5 minutes timeout
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
                source: "pathplan_planner".to_string(),
            },
            occurred_at: Utc::now(),
        };

        // Publish assignment event
        let current_version = 0; // In a real system, we'd track the version properly
        self.event_store
            .append_events(planner_id, current_version, vec![event_envelope.clone()])
            .await?;
                // Best-effort mirror for other appended events in runtime using the long-lived store
                if let Some(store) = self.esrs_store.as_ref() {
                    if let Ok(evt) = serde_json::from_value::<PathPlanningEvent>(serde_json::to_value(&event_envelope.event_data).unwrap()) {
                        let agg_uuid = gryphon_app::adapters::inbound::esrs_pg_store::uuid_for_aggregate_id(planner_id);
                        let mut agg_state = esrs::AggregateState::<gryphon_app::esrs::path_planning::PathPlannerState>::with_id(agg_uuid);
                        match gryphon_app::adapters::inbound::esrs_pg_store::agg_last_sequence(&agg_uuid).await {
                            Ok(Some(n)) if n >= (event_envelope.event_version as i64) => {
                                println!("⤴️ esrs pre-check: event with version {} already present for agg {} (seq={}), skipping persist", event_envelope.event_version, agg_uuid, n);
                            }
                            _ => {
                                let _ = gryphon_app::adapters::inbound::esrs_pg_store::persist_best_effort(store, &mut agg_state, vec![evt]).await;
                            }
                        }
                    }
                }

        println!(
            "📤 Published PlanAssigned event for plan {} to worker {}",
            plan_id, worker_id
        );

        Ok(())
    }

    async fn print_status(&self) {
        println!("� Planner Status:");
        println!("   🗺️  Active planners: {}", self.planners.len());
        println!(
            "   👷 Available workers: {}",
            self.available_workers
                .values()
                .filter(|w| matches!(w.status, WorkerStatus::Ready))
                .count()
        );
        println!(
            "   🔄 Busy workers: {}",
            self.available_workers
                .values()
                .filter(|w| matches!(w.status, WorkerStatus::Busy { .. }))
                .count()
        );

        for (worker_id, worker_info) in &self.available_workers {
            match &worker_info.status {
                WorkerStatus::Ready => println!("     ✅ {}: Ready", worker_id),
                WorkerStatus::Busy { plan_id } => {
                    println!("     🔄 {}: Working on {}", worker_id, plan_id)
                }
                WorkerStatus::Offline => println!("     ❌ {}: Offline", worker_id),
            }
        }
    }

    async fn register_mock_worker(&mut self) {
        let worker_id = "worker-1".to_string();
        let worker_info = WorkerInfo {
            worker_id: worker_id.clone(),
            capabilities: vec![PlanningAlgorithm::AStar],
            status: WorkerStatus::Ready,
            last_heartbeat: Utc::now(),
        };

        self.available_workers
            .insert(worker_id.clone(), worker_info);
        println!("🤖 Registered mock worker: {} for demo purposes", worker_id);
    }
}
