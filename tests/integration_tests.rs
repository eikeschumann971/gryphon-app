use gryphon_app::adapters::*;
use gryphon_app::common::*;

#[tokio::test]
async fn test_in_memory_event_store() {
    let event_store = InMemoryEventStore::new();

    // Test appending and loading events
    let metadata = EventMetadata {
        correlation_id: Some(uuid::Uuid::new_v4()),
        causation_id: None,
        user_id: Some("test-user".to_string()),
        source: "test".to_string(),
    };

    let event_envelope = EventEnvelope {
        event_id: uuid::Uuid::new_v4(),
        aggregate_id: "test-aggregate".to_string(),
        aggregate_type: "TestAggregate".to_string(),
        event_type: "TestEvent".to_string(),
        event_version: 1,
        event_data: serde_json::json!({"test": "data"}),
        metadata,
        occurred_at: chrono::Utc::now(),
    };

    // Test append
    event_store
        .append_events("test-aggregate", 0, vec![event_envelope.clone()])
        .await
        .unwrap();

    // Test load
    let events = event_store.load_events("test-aggregate", 0).await.unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].aggregate_id, "test-aggregate");
}

#[tokio::test]
async fn test_in_memory_snapshot_store() {
    let snapshot_store = InMemorySnapshotStore::new();

    let snapshot = Snapshot {
        snapshot_id: uuid::Uuid::new_v4(),
        aggregate_id: "test-aggregate".to_string(),
        aggregate_type: "TestAggregate".to_string(),
        aggregate_version: 1,
        snapshot_data: serde_json::json!({"state": "test"}),
        created_at: chrono::Utc::now(),
    };

    // Test save
    snapshot_store
        .save_snapshot(snapshot.clone())
        .await
        .unwrap();

    // Test load
    let loaded_snapshot = snapshot_store
        .load_snapshot("test-aggregate", None)
        .await
        .unwrap();
    assert!(loaded_snapshot.is_some());
    assert_eq!(loaded_snapshot.unwrap().aggregate_id, "test-aggregate");
}
