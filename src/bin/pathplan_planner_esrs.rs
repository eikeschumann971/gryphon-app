use std::env;
use std::time::Duration;

use anyhow::Context;
use esrs::store::EventStore;
use esrs::Aggregate;
use esrs::AggregateState;
use uuid::Uuid;

use gryphon_app::adapters::inbound::esrs_pg_store::build_pg_store_with_bus;
use gryphon_app::adapters::outbound::esrs_kafka_bus::KafkaEventBus;
use gryphon_app::esrs::path_planning::{PathPlanner, PathPlannerCommand, PathPlannerState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Configuration via env with sensible defaults for local smoke tests
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@127.0.0.1:5432/postgres".to_string());
    let kafka_brokers = env::var("KAFKA_BROKERS").unwrap_or_else(|_| "localhost:9092".to_string());
    let topic = env::var("EVENT_TOPIC").unwrap_or_else(|_| "path-planning-events".to_string());

    println!(
        "🔌 Building PgStore (Postgres) + KafkaEventBus ({} -> {})",
        kafka_brokers, topic
    );

    // Create a KafkaEventBus and wire it into a PgStore for the PathPlanner aggregate
    let kafka_bus = KafkaEventBus::<PathPlanner>::new(&kafka_brokers, &topic);
    let store = build_pg_store_with_bus::<PathPlanner, _>(&database_url, kafka_bus)
        .await
        .context("failed to build PgStore with Kafka bus")?;

    println!("✅ Built PgStore and attached KafkaEventBus");

    // Create a new aggregate state and persist a CreatePlanner command to exercise persist+publish
    let mut agg_state = AggregateState::<PathPlannerState>::with_id(Uuid::new_v4());

    let cmd = PathPlannerCommand::CreatePlanner {
        planner_id: agg_state.id().to_string(),
        algorithm: gryphon_app::domains::path_planning::aggregate::types::PlanningAlgorithm::AStar,
    };

    let events =
        PathPlanner::handle_command(agg_state.inner(), cmd).context("handle command failed")?;
    store
        .persist(&mut agg_state, events)
        .await
        .context("persist failed")?;

    println!(
        "📥 Persisted CreatePlanner event for aggregate {}",
        agg_state.id()
    );

    // give the async bus a moment to publish
    tokio::time::sleep(Duration::from_secs(1)).await;

    println!("✨ Smoke run complete");
    Ok(())
}
