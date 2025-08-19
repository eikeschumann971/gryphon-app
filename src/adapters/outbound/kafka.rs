use crate::common::{EventEnvelope, EventStore};
use crate::config::KafkaConfig;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rdkafka::config::ClientConfig;
use rdkafka::consumer::StreamConsumer;
use rdkafka::producer::{FutureProducer, FutureRecord};
use serde_json;
use std::time::Duration;

pub struct KafkaEventStore {
    producer: FutureProducer,
    _consumer: StreamConsumer,
    config: KafkaConfig,
}

impl KafkaEventStore {
    pub async fn new(config: KafkaConfig) -> Result<Self, String> {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", config.brokers.join(","))
            .set("client.id", &config.client_id)
            .set("message.timeout.ms", "5000")
            .create()
            .map_err(|e| format!("Failed to create Kafka producer: {}", e))?;

        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", config.brokers.join(","))
            .set("group.id", &config.group_id)
            .set("client.id", &config.client_id)
            .set("enable.partition.eof", "false")
            .set("session.timeout.ms", "6000")
            .set("enable.auto.commit", "true")
            .create()
            .map_err(|e| format!("Failed to create Kafka consumer: {}", e))?;

        Ok(Self {
            producer,
            _consumer: consumer,
            config,
        })
    }

    fn get_topic_for_aggregate(&self, aggregate_type: &str) -> &str {
        match aggregate_type.to_lowercase().as_str() {
            "logicalagent" => &self.config.topics.logical_agent_events,
            "technicalagent" => &self.config.topics.technical_agent_events,
            "kinematicagent" => &self.config.topics.kinematic_agent_events,
            "pathplanning" => &self.config.topics.path_planning_events,
            "dynamics" => &self.config.topics.dynamics_events,
            "gui" => &self.config.topics.gui_events,
            _ => "default-events",
        }
    }
}

#[async_trait]
impl EventStore for KafkaEventStore {
    async fn append_events(
        &self,
        aggregate_id: &str,
        _expected_version: u64,
        events: Vec<EventEnvelope>,
    ) -> Result<(), String> {
        for event in events {
            let topic = self.get_topic_for_aggregate(&event.aggregate_type);
            let key = format!("{}:{}", event.aggregate_type, aggregate_id);

            let payload = serde_json::to_string(&event)
                .map_err(|e| format!("Failed to serialize event: {}", e))?;

            let record = FutureRecord::to(topic).key(&key).payload(&payload);

            self.producer
                .send(record, Duration::from_secs(5))
                .await
                .map_err(|(e, _)| format!("Failed to send event to Kafka: {}", e))?;
        }

        Ok(())
    }

    async fn load_events(
        &self,
        _aggregate_id: &str,
        _from_version: u64,
    ) -> Result<Vec<EventEnvelope>, String> {
        // Note: This is a simplified implementation
        // In a real-world scenario, you'd need to implement proper event replay
        // from Kafka, potentially using a separate topic for event storage
        // or implementing compaction strategies

        // For now, return empty as Kafka is primarily used for event publishing
        // Consider using a separate event store (like EventStore DB) for event sourcing
        // and Kafka for event broadcasting

        Ok(Vec::new())
    }

    async fn load_events_by_type(
        &self,
        _event_type: &str,
        _from_timestamp: Option<DateTime<Utc>>,
    ) -> Result<Vec<EventEnvelope>, String> {
        // Similar to load_events, this would require implementing
        // proper event querying capabilities

        Ok(Vec::new())
    }
}

// Helper function to create all required Kafka topics
pub async fn create_kafka_topics(config: &KafkaConfig) -> Result<(), String> {
    // Note: In a real implementation, you would use Kafka Admin API
    // to create topics programmatically

    let topics = vec![
        &config.topics.logical_agent_events,
        &config.topics.technical_agent_events,
        &config.topics.kinematic_agent_events,
        &config.topics.path_planning_events,
        &config.topics.dynamics_events,
        &config.topics.gui_events,
    ];

    tracing::info!("Required Kafka topics: {:?}", topics);
    tracing::warn!("Auto-creation of Kafka topics not implemented. Please ensure topics exist.");

    Ok(())
}
