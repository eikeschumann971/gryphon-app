use esrs::store::EventStore as EsrsEventStore;
use esrs::Aggregate;
use esrs::AggregateState;
use gryphon_app::adapters::inbound::esrs_pg_store::build_pg_store_with_bus;
use gryphon_app::adapters::outbound::esrs_kafka_bus::InMemoryBus;
use gryphon_app::esrs::path_planning::PathPlanner;
use gryphon_app::esrs::path_planning::PathPlannerState;
use std::time::Duration;
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::postgres::Postgres;

#[tokio::test]
async fn esrs_pgstore_and_bus_end_to_end() {
    // Start a Postgres container for the test
    let node = Postgres::default().start().await.expect("start postgres");
    let port = node.get_host_port_ipv4(5432).await.unwrap();
    let url = format!("postgres://postgres:postgres@127.0.0.1:{}/postgres", port);

    // In-memory bus to capture published events
    let bus = InMemoryBus::<PathPlanner>::new();
    let bus_clone = bus.clone();

    // Build PgStore wired to in-memory bus
    let store = build_pg_store_with_bus::<PathPlanner, _>(&url, bus_clone)
        .await
        .expect("store built");

    // Create AggregateState and persist a CreatePlanner command
    let mut agg_state = AggregateState::<PathPlannerState>::with_id(uuid::Uuid::new_v4());
    let cmd = gryphon_app::esrs::path_planning::PathPlannerCommand::CreatePlanner {
        planner_id: agg_state.id().to_string(),
        algorithm: gryphon_app::domains::path_planning::aggregate::types::PlanningAlgorithm::AStar,
    };

    let events = PathPlanner::handle_command(agg_state.inner(), cmd).expect("handle");
    EsrsEventStore::persist(&store, &mut agg_state, events)
        .await
        .expect("persist");

    // Allow background bus handlers to run
    tokio::time::sleep(Duration::from_millis(300)).await;

    // Assert that the in-memory bus got at least one published event.
    // Scope the mutex guard so it is dropped before any subsequent `.await` calls.
    {
        let lock = bus.published.lock().unwrap();
        assert!(!lock.is_empty(), "no events published to in-memory bus");
    }

    // Also assert that Postgres contains events for esrs
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(2)
        .connect(&url)
        .await
        .expect("connect to pg");

    // Inspect pg_tables to find candidate event tables created by the PgStore migrations
    let table_names: Vec<(String,)> =
        sqlx::query_as("SELECT tablename FROM pg_tables WHERE schemaname = 'public'")
            .fetch_all(&pool)
            .await
            .expect("fetch table names");

    // Collect names and look for likely event table candidates
    let names: Vec<String> = table_names.into_iter().map(|t| t.0).collect();
    let candidate = names
        .iter()
        .find(|n| n.contains("esrs") || n.contains("event") || n.contains("events"))
        .cloned();

    let table = match candidate {
        Some(t) => t,
        None => panic!(
            "no candidate event table found in Postgres; tables: {:?}",
            names
        ),
    };

    // Count rows in the discovered table
    let query = format!("SELECT COUNT(*) FROM {}", table);
    let count: i64 = sqlx::query_scalar(&query)
        .fetch_one(&pool)
        .await
        .expect("count rows");
    assert!(
        count > 0,
        "no rows found in detected events table {}",
        table
    );
}
