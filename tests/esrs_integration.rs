use esrs::store::EventStore;
use esrs::Aggregate;
use gryphon_app::adapters::inbound::esrs_pg_store::build_pg_store_with_bus;
use gryphon_app::adapters::outbound::esrs_kafka_bus::InMemoryBus;
use gryphon_app::esrs::path_planning::PathPlanner;
use gryphon_app::esrs::path_planning::PathPlannerState;
use std::time::Duration;
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::postgres::Postgres;

#[tokio::test]
async fn pgstore_persist_and_publish() {
    let node = Postgres::default().start().await.expect("start postgres");
    let port = node.get_host_port_ipv4(5432).await.unwrap();
    let url = format!("postgres://postgres:postgres@127.0.0.1:{}/postgres", port);

    // create in-memory bus wrapped in Arc
    let bus = InMemoryBus::<PathPlanner>::new();
    let bus_clone = bus.clone();

    let store = build_pg_store_with_bus::<PathPlanner, _>(&url, bus_clone)
        .await
        .expect("store built");

    // persist an event via store.persist (needs state and events) - we will use esrs helpers
    // Start with AggregateState wrapping default inner
    let mut agg_state = esrs::AggregateState::<PathPlannerState>::with_id(uuid::Uuid::new_v4());

    let events = PathPlanner::handle_command(
        agg_state.inner(),
        gryphon_app::esrs::path_planning::PathPlannerCommand::CreatePlanner {
            planner_id: agg_state.id().to_string(),
            algorithm:
                gryphon_app::domains::path_planning::aggregate::types::PlanningAlgorithm::AStar,
        },
    )
    .expect("handle");

    // Persist using store.persist API (accepts &mut AggregateState)
    store
        .persist(&mut agg_state, events)
        .await
        .expect("persist");

    // verify that the in-memory bus received an event
    // small sleep to allow handlers run
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Access published events from bus
    let lock = bus.published.lock().unwrap();
    assert!(!lock.is_empty(), "no events published to bus");
}
