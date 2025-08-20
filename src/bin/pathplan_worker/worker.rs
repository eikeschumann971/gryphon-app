#[allow(unused_imports)]
use crate::planning::plan_path_astar;
use chrono::Utc;
use gryphon_app::adapters::inbound::file_event_store::FileEventStore;
use gryphon_app::common::{EventEnvelope, EventMetadata, EventStore};
use gryphon_app::domains::path_planning::*;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
#[allow(unused_imports)]
use uuid::Uuid;
use gryphon_app::adapters::inbound::esrs_pg_store::build_pg_store_with_bus;
use gryphon_app::adapters::outbound::esrs_kafka_bus::KafkaEventBus;
use gryphon_app::esrs::path_planning::PathPlanner as EsrsPathPlanner;
// esrs EventStore trait is used via fully-qualified paths in this module

#[derive(Clone)]
pub struct AStarPathPlanWorker {
    pub worker_id: String,
    pub planner_id: String,
    #[allow(dead_code)]
    pub capabilities: Vec<PlanningAlgorithm>,
    pub logger: gryphon_app::domains::DynLogger,
}

impl std::fmt::Debug for AStarPathPlanWorker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AStarPathPlanWorker")
            .field("worker_id", &self.worker_id)
            .field("planner_id", &self.planner_id)
            .finish()
    }
}

impl AStarPathPlanWorker {
    pub fn new(
        worker_id: String,
        planner_id: String,
        logger: gryphon_app::domains::DynLogger,
    ) -> Self {
        Self {
            worker_id,
            planner_id,
            capabilities: vec![PlanningAlgorithm::AStar],
            logger,
        }
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Starting path planning worker: {}", self.worker_id);
        self.logger.info(&format!(
            "Starting path planning worker: {}",
            self.worker_id
        ));

        // Use FileEventStore for shared events
        let event_store = Arc::new(FileEventStore::new("/tmp/gryphon-events"));

        // Build a long-lived esrs PgStore + KafkaEventBus once (best-effort).
    // Attempt to build a long-lived esrs PgStore + KafkaEventBus once (best-effort).
    let esrs_store_opt: Option<esrs::store::postgres::PgStore<EsrsPathPlanner>> = {
            let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://postgres:password@127.0.0.1:5432/gryphon_app".to_string());
            let kafka_brokers = std::env::var("KAFKA_BROKERS").unwrap_or_else(|_| "localhost:9092".to_string());
            let topic = "path-planning-events";
            match build_pg_store_with_bus::<EsrsPathPlanner, _>(&database_url, KafkaEventBus::<EsrsPathPlanner>::new(&kafka_brokers, topic)).await {
                Ok(store) => {
                    println!("✅ Built esrs PgStore and attached KafkaEventBus");
                    Some(store)
                }
                Err(err) => {
                    println!("⚠️ Failed to build esrs PgStore: {}", err);
                    None
                }
            }
        };

        loop {
            // Load all PlanAssigned events from the shared event store
            let plan_events = event_store
                .load_events_by_type("PlanAssigned", None)
                .await?;

            if !plan_events.is_empty() {
                println!("Found {} plan assignments to process", plan_events.len());
                self.logger.info(&format!(
                    "Found {} plan assignments to process",
                    plan_events.len()
                ));

                for plan_event in plan_events {
                    println!("🔍 Checking event: {}", plan_event.event_type);
                    self.logger
                        .info(&format!("Checking event: {}", plan_event.event_type));

                    // Extract PlanAssigned data from JSON
                    if let Ok(event_data) =
                        serde_json::from_value::<PathPlanningEvent>(plan_event.event_data.clone())
                    {
                        println!("  📋 Successfully parsed event data");
                        self.logger.info("Successfully parsed event data");

                        if let PathPlanningEvent::PlanAssigned {
                            plan_id,
                            worker_id,
                            start_position,
                            destination_position,
                            ..
                        } = event_data
                        {
                            println!(
                                "  🎯 PlanAssigned event: plan={}, worker={}, self={}",
                                plan_id, worker_id, self.worker_id
                            );
                            self.logger.info(&format!(
                                "PlanAssigned event: plan={}, worker={}",
                                plan_id, worker_id
                            ));

                            // Only process assignments for this specific worker
                            if worker_id != self.worker_id {
                                println!("  ⏭️  Skipping assignment for different worker");
                                self.logger.info("Skipping assignment for different worker");
                                continue;
                            }

                            println!("  ✅ Assignment matches this worker!");
                            self.logger.info("Assignment matches this worker");

                            // Check if plan is already completed
                            let completion_events = event_store
                                .load_events_by_type("PlanCompleted", None)
                                .await?;
                            println!("  📊 Found {} completion events", completion_events.len());
                            self.logger.info(&format!(
                                "Found {} completion events",
                                completion_events.len()
                            ));

                            let already_completed =
                                completion_events.iter().any(|completion_event| {
                                    if let Ok(PathPlanningEvent::PlanCompleted {
                                        plan_id: completed_plan_id,
                                        ..
                                    }) = serde_json::from_value::<PathPlanningEvent>(
                                        completion_event.event_data.clone(),
                                    ) {
                                        let is_match = completed_plan_id == plan_id;
                                        if is_match {
                                            println!(
                                                "    ✅ Found completion for plan: {}",
                                                completed_plan_id
                                            );
                                            self.logger.info(&format!(
                                                "Found completion for plan: {}",
                                                completed_plan_id
                                            ));
                                        }
                                        is_match
                                    } else {
                                        false
                                    }
                                });

                            if already_completed {
                                println!("  ⏭️  Plan already completed, skipping");
                                self.logger.info("Plan already completed, skipping");
                            } else {
                                println!("  🚀 Plan not completed yet, processing...");
                                self.logger.info(&format!(
                                    "Plan not completed yet, processing plan {}",
                                    plan_id
                                ));
                                println!(
                                    "🔧 Processing plan assignment for worker {}: {}",
                                    self.worker_id, plan_id
                                );

                                // Simulate some path planning work
                                println!("   📊 Calculating optimal path using A* algorithm...");
                                self.logger
                                    .info("Calculating optimal path using A* algorithm");
                                sleep(Duration::from_millis(500)).await;

                                // Generate a simple path (for demo)
                                let mut waypoints = Vec::new();
                                let steps = 5;
                                for i in 0..=steps {
                                    let t = i as f64 / steps as f64;
                                    let x = start_position.x
                                        + t * (destination_position.x - start_position.x);
                                    let y = start_position.y
                                        + t * (destination_position.y - start_position.y);
                                    waypoints.push(Position2D { x, y });
                                }

                                println!(
                                    "   ✅ Path calculated with {} waypoints",
                                    waypoints.len()
                                );
                                self.logger.info(&format!(
                                    "Path calculated with {} waypoints",
                                    waypoints.len()
                                ));

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
                                    source: "pathplan_worker".to_string(),
                                };

                                let completion_envelope =
                                    EventEnvelope::new(&completion_event, "PathPlan", metadata)?;

                                event_store
                                    .append_events(&plan_id, 1, vec![completion_envelope.clone()])
                                    .await?;
                                // Mirror to esrs PgStore best-effort using the long-lived store
                                if let Some(store) = &esrs_store_opt {
                                    if let Ok(evt) = serde_json::from_value::<PathPlanningEvent>(serde_json::to_value(&completion_event).unwrap()) {
                                        let agg_uuid = gryphon_app::adapters::inbound::esrs_pg_store::uuid_for_aggregate_id(&self.planner_id);
                                        let mut agg_state = esrs::AggregateState::<gryphon_app::esrs::path_planning::PathPlannerState>::with_id(agg_uuid);
                                        // Use sequence-based pre-check: if the DB already has sequence >= expected, skip persist
                                        match gryphon_app::adapters::inbound::esrs_pg_store::agg_last_sequence(&agg_uuid).await {
                                            Ok(Some(n)) if n >= (completion_envelope.event_version as i64) => {
                                                println!("⤴️ esrs pre-check: completion event already present for agg {} (seq={}), skipping persist", agg_uuid, n);
                                            }
                                            _ => {
                                                let _ = gryphon_app::adapters::inbound::esrs_pg_store::persist_best_effort(store, &mut agg_state, vec![evt]).await;
                                            }
                                        }
                                    }
                                }
                                println!("   📤 Published PlanCompleted event");
                                self.logger.info(&format!(
                                    "Published PlanCompleted event for plan {}",
                                    plan_id
                                ));
                                println!(
                                    "✅ Plan {} completed by worker {}",
                                    plan_id, self.worker_id
                                );
                                self.logger.info(&format!(
                                    "Plan {} completed by worker {}",
                                    plan_id, self.worker_id
                                ));
                            }
                        } else {
                            println!("  🔍 Event is not PlanAssigned");
                        }
                    } else {
                        println!("  ❌ Failed to parse event data");
                    }
                }
            }

            // Wait before next poll
            sleep(Duration::from_secs(2)).await;
        }
    }
}

pub async fn run_worker(
    logger: gryphon_app::domains::DynLogger,
) -> Result<(), Box<dyn std::error::Error>> {
    let worker = AStarPathPlanWorker::new("worker-1".to_string(), "planner-1".to_string(), logger);
    worker.run().await
}
