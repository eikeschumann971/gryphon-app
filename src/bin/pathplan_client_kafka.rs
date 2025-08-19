use chrono::Utc;
use gryphon_app::adapters::inbound::kafka_event_store::KafkaEventStore;
use gryphon_app::common::{DomainEvent, EventEnvelope, EventMetadata, EventStore};
use gryphon_app::domains::path_planning::*;
use std::f64::consts::PI;
use tokio::time::{sleep, Duration};
use uuid::Uuid;

async fn run_kafka_client() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize combined logger (file + console fallback)
    let logger = gryphon_app::adapters::outbound::init_combined_logger("./domain.log");
    logger.info("Starting Path Planning Client (Kafka Event-Driven)");

    // Initialize Kafka Event Store
    let event_store =
        KafkaEventStore::new("localhost:9092", "path-planning-events", "client-group").await?;

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
        .append_events(&planner_id, 1, vec![event_envelope])
        .await?;
    logger.info(&format!(
        "Event published successfully to Kafka: plan_id={}",
        plan_id
    ));
    println!("   🎯 Plan ID: {}", plan_id);
    println!("   📝 Event: PathPlanRequested");

    // Wait and listen for response events from Kafka
    logger.info("Waiting for response events from Kafka");

    let mut assigned_found = false;
    let mut completed_found = false;

    // Poll for events for up to 30 seconds
    for i in 1..=10 {
        sleep(Duration::from_secs(3)).await;

        // Check for PlanAssigned events
        if !assigned_found {
            let assigned_events = event_store
                .load_events_by_type("PlanAssigned", None)
                .await?;
            for event in assigned_events {
                if let Ok(PathPlanningEvent::PlanAssigned {
                    plan_id: assigned_plan_id,
                    worker_id,
                    ..
                }) = serde_json::from_value::<PathPlanningEvent>(event.event_data)
                {
                    if assigned_plan_id == plan_id {
                        logger.info(&format!(
                            "Received PlanAssigned for plan {} worker={}",
                            assigned_plan_id, worker_id
                        ));
                        assigned_found = true;
                    }
                }
            }
        }

        // Check for PlanCompleted events
        if !completed_found {
            let completed_events = event_store
                .load_events_by_type("PlanCompleted", None)
                .await?;
            for event in completed_events {
                if let Ok(PathPlanningEvent::PlanCompleted {
                    plan_id: completed_plan_id,
                    waypoints,
                    worker_id,
                    ..
                }) = serde_json::from_value::<PathPlanningEvent>(event.event_data)
                {
                    if completed_plan_id == plan_id {
                        logger.info(&format!(
                            "Received PlanCompleted for plan {} waypoints={} worker={:?}",
                            completed_plan_id,
                            waypoints.len(),
                            worker_id
                        ));
                        println!("   📍 Sample waypoints from completed plan:");
                        for (idx, waypoint) in waypoints.iter().take(3).enumerate() {
                            println!("      {}. ({:.1}, {:.1})", idx + 1, waypoint.x, waypoint.y);
                        }
                        if waypoints.len() > 3 {
                            println!("      ... and {} more waypoints", waypoints.len() - 3);
                        }
                        completed_found = true;
                    }
                }
            }
        }

        if assigned_found && completed_found {
            break;
        }

        logger.info(&format!("Polling Kafka for events attempt {}/10", i));
    }

    if !assigned_found {
        logger.warn("No PlanAssigned event received from Kafka");
    }
    if !completed_found {
        logger.warn("No PlanCompleted event received from Kafka");
    }

    logger.info("Kafka client demo completed");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_kafka_client().await
}
