use esrs::bus::EventBus;
use esrs::store::StoreEvent;
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::marker::PhantomData;

pub struct KafkaEventBus<A> {
    producer: FutureProducer,
    topic: String,
    _marker: PhantomData<A>,
}

impl<A> KafkaEventBus<A> {
    pub fn new(brokers: &str, topic: &str) -> Self {
        let producer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("client.id", "esrs-kafka-bus")
            .create()
            .expect("Failed to create Kafka producer");
        Self { producer, topic: topic.to_string(), _marker: PhantomData }
    }
}

// Simple in-memory EventBus for tests and integration helpers
pub struct InMemoryBus<A>
where
    A: esrs::Aggregate,
    A::Event: Clone,
{
    pub published: Arc<Mutex<Vec<esrs::store::StoreEvent<A::Event>>>>,
    _marker: PhantomData<A>,
}

impl<A> InMemoryBus<A>
where
    A: esrs::Aggregate,
    A::Event: Clone,
{
    pub fn new() -> Self {
        Self { published: Arc::new(Mutex::new(Vec::new())), _marker: PhantomData }
    }
}

impl<A> Default for InMemoryBus<A>
where
    A: esrs::Aggregate,
    A::Event: Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

// Manual Clone impl to avoid imposing Clone on the aggregate type A
impl<A> Clone for InMemoryBus<A>
where
    A: esrs::Aggregate,
    A::Event: Clone,
{
    fn clone(&self) -> Self {
        Self { published: Arc::clone(&self.published), _marker: PhantomData }
    }
}

#[async_trait::async_trait]
impl<A> EventBus<A> for InMemoryBus<A>
where
    A: esrs::Aggregate + Sync,
    A::Event: Clone + Send + Sync + 'static,
{
    async fn publish(&self, store_event: &esrs::store::StoreEvent<A::Event>) {
        let mut lock = self.published.lock().unwrap();
        // construct owned StoreEvent by cloning fields (StoreEvent doesn't implement Clone)
        let owned = esrs::store::StoreEvent {
            id: store_event.id,
            aggregate_id: store_event.aggregate_id,
            payload: store_event.payload.clone(),
            occurred_on: store_event.occurred_on,
            sequence_number: *store_event.sequence_number(),
            version: store_event.version,
        };
        lock.push(owned);
    }
}

#[async_trait::async_trait]
impl<A> EventBus<A> for KafkaEventBus<A>
where
    A: esrs::Aggregate + Sync,
    A::Event: serde::Serialize + Send + Sync + 'static,
{
    async fn publish(&self, store_event: &StoreEvent<A::Event>) {
        let payload = serde_json::to_string(store_event).unwrap_or_default();
        // use string forms for payload and key
        let key = store_event.aggregate_id.to_string();
        let record = FutureRecord::to(&self.topic)
            .payload(&payload)
            .key(&key);
        let _ = self.producer.send(record, Duration::from_secs(5)).await;
    }
}
