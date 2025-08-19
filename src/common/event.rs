use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub trait DomainEvent: Send + Sync + Clone {
    fn event_type(&self) -> &'static str;
    fn aggregate_id(&self) -> &str;
    fn event_version(&self) -> u64;
    fn occurred_at(&self) -> DateTime<Utc>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub event_id: Uuid,
    pub aggregate_id: String,
    pub aggregate_type: String,
    pub event_type: String,
    pub event_version: u64,
    pub event_data: serde_json::Value,
    pub metadata: EventMetadata,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    pub correlation_id: Option<Uuid>,
    pub causation_id: Option<Uuid>,
    pub user_id: Option<String>,
    pub source: String,
}

impl EventEnvelope {
    pub fn new<E: DomainEvent + Serialize>(
        event: &E,
        aggregate_type: &str,
        metadata: EventMetadata,
    ) -> Result<Self, serde_json::Error> {
        Ok(Self {
            event_id: Uuid::new_v4(),
            aggregate_id: event.aggregate_id().to_string(),
            aggregate_type: aggregate_type.to_string(),
            event_type: event.event_type().to_string(),
            event_version: event.event_version(),
            event_data: serde_json::to_value(event)?,
            metadata,
            occurred_at: event.occurred_at(),
        })
    }
}

#[async_trait::async_trait]
pub trait EventStore {
    async fn append_events(
        &self,
        aggregate_id: &str,
        expected_version: u64,
        events: Vec<EventEnvelope>,
    ) -> Result<(), String>;

    async fn load_events(
        &self,
        aggregate_id: &str,
        from_version: u64,
    ) -> Result<Vec<EventEnvelope>, String>;

    async fn load_events_by_type(
        &self,
        event_type: &str,
        from_timestamp: Option<DateTime<Utc>>,
    ) -> Result<Vec<EventEnvelope>, String>;
}
