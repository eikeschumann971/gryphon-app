use crate::common::{EventEnvelope, EventStore};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use tokio::sync::RwLock;

/// In-memory event store implementation for testing and development
#[derive(Debug, Default)]
pub struct InMemoryEventStore {
    events: RwLock<HashMap<String, Vec<EventEnvelope>>>,
}

impl InMemoryEventStore {
    pub fn new() -> Self {
        Self {
            events: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl EventStore for InMemoryEventStore {
    async fn append_events(
        &self,
        aggregate_id: &str,
        expected_version: u64,
        events: Vec<EventEnvelope>,
    ) -> Result<(), String> {
        let mut store = self.events.write().await;

        let aggregate_events = store
            .entry(aggregate_id.to_string())
            .or_insert_with(Vec::new);

        // Check expected version
        let current_version = aggregate_events.len() as u64;
        if current_version != expected_version {
            return Err(format!(
                "Version mismatch: expected {}, got {}",
                expected_version, current_version
            ));
        }

        aggregate_events.extend(events);
        Ok(())
    }

    async fn load_events(
        &self,
        aggregate_id: &str,
        from_version: u64,
    ) -> Result<Vec<EventEnvelope>, String> {
        let store = self.events.read().await;

        if let Some(events) = store.get(aggregate_id) {
            let filtered_events: Vec<EventEnvelope> =
                events.iter().skip(from_version as usize).cloned().collect();
            Ok(filtered_events)
        } else {
            Ok(Vec::new())
        }
    }

    async fn load_events_by_type(
        &self,
        event_type: &str,
        from_timestamp: Option<DateTime<Utc>>,
    ) -> Result<Vec<EventEnvelope>, String> {
        let store = self.events.read().await;

        let mut filtered_events = Vec::new();

        for events in store.values() {
            for event in events {
                if event.event_type == event_type {
                    if let Some(from_ts) = from_timestamp {
                        if event.occurred_at >= from_ts {
                            filtered_events.push(event.clone());
                        }
                    } else {
                        filtered_events.push(event.clone());
                    }
                }
            }
        }

        // Sort by timestamp
        filtered_events.sort_by(|a, b| a.occurred_at.cmp(&b.occurred_at));

        Ok(filtered_events)
    }
}
