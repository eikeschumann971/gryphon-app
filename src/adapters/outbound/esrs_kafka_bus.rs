#![cfg(feature = "esrs_migration")]

use esrs::EventBus;
use esrs::store::StoreEvent;
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use std::time::Duration;
use std::sync::Arc;

pub struct KafkaEventBus {
    producer: FutureProducer,
    topic: String,
}

impl KafkaEventBus {
    pub fn new(brokers: &str, topic: &str) -> Self {
        let producer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("client.id", "esrs-kafka-bus")
            .create()
            .expect("Failed to create Kafka producer");
        Self { producer, topic: topic.to_string() }
    }
}

#[async_trait::async_trait]
impl<T> EventBus<T> for KafkaEventBus
where
    T: serde::Serialize + Send + Sync + 'static,
{
    async fn publish(&self, store_event: &StoreEvent<T>) {
        let payload = serde_json::to_string(store_event).unwrap_or_default();
        let record: FutureRecord<&str, _> = FutureRecord::to(&self.topic)
            .payload(&payload)
            .key(&store_event.aggregate_id);
        let _ = self.producer.send(record, Duration::from_secs(5)).await;
    }
}
