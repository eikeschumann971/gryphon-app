use crate::common::{EventStore, EventEnvelope};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::{Message, TopicPartitionList};
use serde_json;
use std::time::Duration;
use std::collections::HashMap;
use uuid::Uuid;

/// Kafka-based EventStore implementation for distributed event-driven architecture
/// 
/// This implementation stores events in Kafka topics and allows for real-time
/// event consumption across multiple processes
pub struct KafkaEventStore {
    producer: FutureProducer,
    consumer: StreamConsumer,
    topic_name: String,
}

impl KafkaEventStore {
    pub async fn new(bootstrap_servers: &str, topic_name: &str, group_id: &str) -> Result<Self, String> {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", bootstrap_servers)
            .set("client.id", "gryphon-producer")
            .set("message.timeout.ms", "5000")
            .create()
            .map_err(|e| format!("Failed to create Kafka producer: {}", e))?;

        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", bootstrap_servers)
            .set("group.id", group_id)
            .set("client.id", "gryphon-consumer")
            .set("enable.partition.eof", "false")
            .set("session.timeout.ms", "6000")
            .set("enable.auto.commit", "true")
            .set("auto.offset.reset", "earliest")
            .create()
            .map_err(|e| format!("Failed to create Kafka consumer: {}", e))?;

        Ok(Self {
            producer,
            consumer,
            topic_name: topic_name.to_string(),
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
        // Subscribe to the topic
        self.consumer
            .subscribe(&[&self.topic_name])
            .map_err(|e| format!("Failed to subscribe to topic: {}", e))?;

        let mut events = Vec::new();
        let timeout = Duration::from_secs(2);
        
        // Read all available messages
        let start_time = std::time::Instant::now();
        while start_time.elapsed() < timeout {
            match self.consumer.recv().await {
                Ok(message) => {
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
                Err(_) => break, // No more messages or timeout
            }
        }

        Ok(events)
    }

    async fn load_events_by_type(
        &self,
        event_type: &str,
        _from_timestamp: Option<DateTime<Utc>>,
    ) -> Result<Vec<EventEnvelope>, String> {
        // Subscribe to the topic
        self.consumer
            .subscribe(&[&self.topic_name])
            .map_err(|e| format!("Failed to subscribe to topic: {}", e))?;

        let mut events = Vec::new();
        let timeout = Duration::from_secs(2);
        
        // Read all available messages
        let start_time = std::time::Instant::now();
        while start_time.elapsed() < timeout {
            match self.consumer.recv().await {
                Ok(message) => {
                    if let Some(payload) = message.payload() {
                        let payload_str = String::from_utf8_lossy(payload);
                        match serde_json::from_str::<EventEnvelope>(&payload_str) {
                            Ok(event) => {
                                if event.event_type == event_type {
                                    events.push(event);
                                }
                            }
                            Err(e) => tracing::warn!("Failed to deserialize event: {}", e),
                        }
                    }
                }
                Err(_) => break, // No more messages or timeout
            }
        }

        Ok(events)
    }
}
