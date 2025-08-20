use crate::common::{EventEnvelope, EventStore};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::Message;
use serde_json;
use std::time::Duration;
use uuid::Uuid;

/// Kafka-based EventStore implementation for distributed event-driven architecture
///
/// This implementation stores events in Kafka topics and allows for real-time
/// event consumption across multiple processes
pub struct KafkaEventStore {
    producer: FutureProducer,
    topic_name: String,
    bootstrap_servers: String,
}

impl KafkaEventStore {
    pub async fn new(
        bootstrap_servers: &str,
        topic_name: &str,
        _group_id: &str,
    ) -> Result<Self, String> {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", bootstrap_servers)
            .set("client.id", "gryphon-producer")
            .set("message.timeout.ms", "5000")
            .create()
            .map_err(|e| format!("Failed to create Kafka producer: {}", e))?;

        Ok(Self {
            producer,
            topic_name: topic_name.to_string(),
            bootstrap_servers: bootstrap_servers.to_string(),
        })
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
            let key = format!("{}:{}", event.aggregate_type, aggregate_id);

            let payload = serde_json::to_string(&event)
                .map_err(|e| format!("Failed to serialize event: {}", e))?;

            let record = FutureRecord::to(&self.topic_name)
                .key(&key)
                .payload(&payload);

            self.producer
                .send(record, Duration::from_secs(5))
                .await
                .map_err(|(e, _)| format!("Failed to send event to Kafka: {}", e))?;
        }

        Ok(())
    }

    async fn load_events(
        &self,
        aggregate_id: &str,
        _from_version: u64,
    ) -> Result<Vec<EventEnvelope>, String> {
        // Create a short-lived consumer with a unique group id so we read from the beginning
        let temp_group = format!("temp-reader-{}", Uuid::new_v4());
        let temp_consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", &self.bootstrap_servers)
            .set("group.id", &temp_group)
            .set("client.id", "gryphon-temp-consumer")
            .set("enable.partition.eof", "false")
            .set("session.timeout.ms", "6000")
            .set("enable.auto.commit", "false")
            .set("auto.offset.reset", "earliest")
            .create()
            .map_err(|e| format!("Failed to create temp consumer: {}", e))?;

        temp_consumer
            .subscribe(&[&self.topic_name])
            .map_err(|e| format!("Failed to subscribe to topic: {}", e))?;

        let mut events = Vec::new();
        let timeout = Duration::from_secs(2);

        // Read all available messages within the timeout
        let start_time = std::time::Instant::now();
        while start_time.elapsed() < timeout {
            match tokio::time::timeout(Duration::from_millis(200), temp_consumer.recv()).await {
                Ok(Ok(message)) => {
                    if let Some(key) = message.key() {
                        let key_str = String::from_utf8_lossy(key);
                        if key_str.contains(aggregate_id) {
                            if let Some(payload) = message.payload() {
                                let payload_str = String::from_utf8_lossy(payload);
                                match serde_json::from_str::<EventEnvelope>(&payload_str) {
                                    Ok(event) => events.push(event),
                                    Err(e) => tracing::warn!("Failed to deserialize event: {}", e),
                                }
                            }
                        }
                    }
                }
                _ => break, // Timeout or error - stop reading
            }
        }

        Ok(events)
    }

    async fn load_events_by_type(
        &self,
        event_type: &str,
        _from_timestamp: Option<DateTime<Utc>>,
    ) -> Result<Vec<EventEnvelope>, String> {
        // Create ephemeral consumer for polling new events. Use 'latest' so the
        // ephemeral consumer starts at the end of the log and only receives
        // messages produced after the request. Make the read window slightly
        // longer to allow subscription and partition assignment to complete.
        let temp_group = format!("temp-reader-{}", Uuid::new_v4());
        let temp_consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", &self.bootstrap_servers)
            .set("group.id", &temp_group)
            .set("client.id", format!("gryphon-temp-consumer-{}", Uuid::new_v4()))
            .set("enable.partition.eof", "false")
            .set("session.timeout.ms", "6000")
            .set("enable.auto.commit", "false")
            .set("auto.offset.reset", "latest")
            .create()
            .map_err(|e| format!("Failed to create temp consumer: {}", e))?;

        temp_consumer
            .subscribe(&[&self.topic_name])
            .map_err(|e| format!("Failed to subscribe to topic: {}", e))?;

        let mut events = Vec::new();
        // Increase the read window so subscription/assignment can settle and
        // newly produced events are received.
        let timeout = Duration::from_millis(1500);

        // Read all available messages within the timeout
        let start_time = std::time::Instant::now();
        while start_time.elapsed() < timeout {
            // Wait a bit longer per recv to tolerate scheduling delays
            match tokio::time::timeout(Duration::from_millis(300), temp_consumer.recv()).await {
                Ok(Ok(message)) => {
                    if let Some(payload) = message.payload() {
                        let payload_str = String::from_utf8_lossy(payload);
                        match serde_json::from_str::<EventEnvelope>(&payload_str) {
                            Ok(event) => {
                                if event.event_type == event_type {
                                    events.push(event);
                                }
                            }
                            Err(e) => println!("⚠️  Failed to deserialize event: {}", e),
                        }
                    }
                }
                _ => break, // Timeout or error - stop
            }
        }

        Ok(events)
    }
}
