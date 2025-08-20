use chrono::Utc;
use gryphon_app::adapters::inbound::kafka_event_store::KafkaEventStore;
use gryphon_app::common::{DomainEvent, EventEnvelope, EventMetadata, EventStore};
use gryphon_app::domains::path_planning::*;
use rdkafka::consumer::Consumer;
use rdkafka::consumer::StreamConsumer;
use rdkafka::Message;
use std::f64::consts::PI;
use std::time::Instant;
use tokio::time::Duration;
use uuid::Uuid;
#[cfg(feature = "esrs_migration")]
use gryphon_app::adapters::inbound::esrs_pg_store::build_pg_store_with_bus;
#[cfg(feature = "esrs_migration")]
use gryphon_app::adapters::outbound::esrs_kafka_bus::KafkaEventBus;
#[cfg(feature = "esrs_migration")]
use gryphon_app::esrs::path_planning::PathPlanner as EsrsPathPlanner;
#[cfg(feature = "esrs_migration")]
use esrs::store::EventStore as EsrsEventStore;

async fn run_kafka_client() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize combined logger (file + console fallback)
    let logger = gryphon_app::adapters::outbound::init_combined_logger("./domain.log");
    logger.info("Starting Path Planning Client (Kafka Event-Driven)");

    // Initialize Kafka Event Store (for publishing requests)
    let event_store =
        KafkaEventStore::new("localhost:9092", "path-planning-events", "client-group").await?;

    // Create a dedicated consumer for replies with a unique group id and subscribe
    // to the shared replies topic before publishing the request so we don't miss replies.
    let reply_group = format!("client-{}", Uuid::new_v4());
    let reply_consumer: StreamConsumer = rdkafka::config::ClientConfig::new()
        .set("group.id", &reply_group)
        .set("bootstrap.servers", "localhost:9092")
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true")
        .set("auto.offset.reset", "latest")
        .create()
        .map_err(|e| format!("Failed to create reply consumer: {}", e))?;

    reply_consumer
        .subscribe(&["path-planning-replies"])
        .map_err(|e| format!("Failed to subscribe to replies topic: {}", e))?;

    // Wait for assignment to complete so we don't miss replies produced
    // immediately after publishing the request. Drive the consumer by
    // briefly polling and check `assignment()` until partitions are
    // assigned or a short timeout elapses.
    let assign_deadline = Instant::now() + Duration::from_secs(3);
    loop {
        match reply_consumer.assignment() {
            Ok(tpl) => {
                if tpl.count() > 0 {
                    logger.info("Reply consumer partition assignment complete");
                    break;
                }
            }
            Err(e) => {
                logger.warn(&format!("Failed to query consumer assignment: {}", e));
            }
        }

        if Instant::now() > assign_deadline {
            logger.warn(
                "Timed out waiting for reply consumer partition assignment; proceeding anyway",
            );
            break;
        }

        // Sleep briefly to allow background assignment to settle
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    logger.info("Connected to Kafka event store for distributed event communication");
    logger.info("Path Planning Client is running");

    // Send a single path planning request
    logger.info("Sending path planning request via Kafka");

    let request_id = format!("req-{}", Uuid::new_v4());
    let plan_id = format!("plan-{}", Uuid::new_v4());
    let agent_id = "kafka-robot-001".to_string();
    let planner_id = "main-path-planner".to_string();

    let start_position = Position2D { x: -50.0, y: -30.0 };
    let destination_position = Position2D { x: 40.0, y: 25.0 };
    let start_orientation = Orientation2D { angle: 0.0 };
    let destination_orientation = Orientation2D { angle: PI / 2.0 };

    logger.info(&format!(
        "Publishing path plan request: {} agent={}",
        request_id, agent_id
    ));
    println!("   🆔 Request ID: {}", request_id);
    println!("   🤖 Agent: {}", agent_id);
    println!(
        "   📍 Start: ({:.1}, {:.1}) @ {:.2}rad",
        start_position.x, start_position.y, start_orientation.angle
    );
    println!(
        "   🎯 Goal:  ({:.1}, {:.1}) @ {:.2}rad",
        destination_position.x, destination_position.y, destination_orientation.angle
    );

    let distance = ((destination_position.x - start_position.x).powi(2)
        + (destination_position.y - start_position.y).powi(2))
    .sqrt();
    println!("   📏 Distance: {:.1} units", distance);

    // Create the PathPlanRequested event
    let event = PathPlanningEvent::PathPlanRequested {
        planner_id: planner_id.clone(),
        request_id: request_id.clone(),
        plan_id: plan_id.clone(),
        agent_id: agent_id.clone(),
        start_position,
        destination_position,
        start_orientation,
        destination_orientation,
        timestamp: Utc::now(),
    };

    let event_envelope = EventEnvelope {
        event_id: Uuid::new_v4(),
        aggregate_id: planner_id.clone(),
        aggregate_type: "PathPlanner".to_string(),
        event_type: event.event_type().to_string(),
        event_version: 1,
        event_data: serde_json::to_value(&event)?,
        metadata: EventMetadata {
            correlation_id: Some(Uuid::new_v4()),
            causation_id: None,
            user_id: Some(agent_id.clone()),
            source: "pathplan_client_kafka".to_string(),
        },
        occurred_at: Utc::now(),
    };

    logger.info("Publishing event to Kafka");
    event_store
        .append_events(&planner_id, 1, vec![event_envelope.clone()])
        .await?;
    #[cfg(feature = "esrs_migration")]
    {
        // Try to mirror to esrs PgStore (best-effort)
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://postgres:password@127.0.0.1:5432/gryphon_app".to_string());
        let kafka_brokers = std::env::var("KAFKA_BROKERS").unwrap_or_else(|_| "localhost:9092".to_string());
        let topic = "path-planning-events";
        if let Ok(store) = build_pg_store_with_bus::<EsrsPathPlanner, _>(&database_url, KafkaEventBus::<EsrsPathPlanner>::new(&kafka_brokers, topic)).await {
            if let Ok(evt) = serde_json::from_value::<gryphon_app::domains::path_planning::events::PathPlanningEvent>(serde_json::to_value(&event).unwrap()) {
                use esrs::AggregateState;
                let mut agg_state = esrs::AggregateState::<gryphon_app::esrs::path_planning::PathPlannerState>::with_id(gryphon_app::adapters::inbound::esrs_pg_store::uuid_for_aggregate_id(&planner_id));
                let _ = EsrsEventStore::persist(&store, &mut agg_state, vec![evt]).await;
            }
        }
    }
    logger.info(&format!(
        "Event published successfully to Kafka: plan_id={}",
        plan_id
    ));
    println!("   🎯 Plan ID: {}", plan_id);
    println!("   📝 Event: PathPlanRequested");

    // Wait and listen for response events from the replies topic
    logger.info("Waiting for response events from Kafka replies topic");

    let mut assigned_found = false;
    let mut completed_found = false;
    let correlation_to_match = event_envelope.metadata.correlation_id;

    // Wait up to 30 seconds for replies
    let overall_deadline = tokio::time::Instant::now() + Duration::from_secs(30);
    while tokio::time::Instant::now() < overall_deadline {
        match tokio::time::timeout(Duration::from_secs(3), reply_consumer.recv()).await {
            Ok(Ok(message)) => {
                if let Some(payload) = message.payload() {
                    if let Ok(envelope) = serde_json::from_slice::<EventEnvelope>(payload) {
                        if envelope.metadata.correlation_id == correlation_to_match {
                            match envelope.event_type.as_str() {
                                "PlanAssigned" => {
                                    if let Ok(PathPlanningEvent::PlanAssigned {
                                        plan_id: assigned_plan_id,
                                        worker_id,
                                        ..
                                    }) = serde_json::from_value::<PathPlanningEvent>(
                                        envelope.event_data,
                                    ) {
                                        if assigned_plan_id == plan_id {
                                            logger.info(&format!(
                                                "Received PlanAssigned for plan {} worker={}",
                                                assigned_plan_id, worker_id
                                            ));
                                            assigned_found = true;
                                        }
                                    }
                                }
                                "PlanCompleted" => {
                                    if let Ok(PathPlanningEvent::PlanCompleted {
                                        plan_id: completed_plan_id,
                                        waypoints,
                                        worker_id,
                                        ..
                                    }) = serde_json::from_value::<PathPlanningEvent>(
                                        envelope.event_data,
                                    ) {
                                        if completed_plan_id == plan_id {
                                            logger.info(&format!("Received PlanCompleted for plan {} waypoints={} worker={:?}", completed_plan_id, waypoints.len(), worker_id));
                                            println!("   📍 Sample waypoints from completed plan:");
                                            for (idx, waypoint) in
                                                waypoints.iter().take(3).enumerate()
                                            {
                                                println!(
                                                    "      {}. ({:.1}, {:.1})",
                                                    idx + 1,
                                                    waypoint.x,
                                                    waypoint.y
                                                );
                                            }
                                            if waypoints.len() > 3 {
                                                println!(
                                                    "      ... and {} more waypoints",
                                                    waypoints.len() - 3
                                                );
                                            }
                                            completed_found = true;
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            _ => {
                // timeout or error - continue until overall deadline
            }
        }

        if assigned_found && completed_found {
            break;
        }
    }

    if !assigned_found {
        logger.warn("No PlanAssigned event received from Kafka replies topic");
    }
    if !completed_found {
        logger.warn("No PlanCompleted event received from Kafka replies topic");
    }

    logger.info("Kafka client demo completed");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_kafka_client().await
}
