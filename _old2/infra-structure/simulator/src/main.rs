use serde::{Deserialize, Serialize};
use rdkafka::producer::{FutureProducer, FutureRecord};
use chrono::Utc;
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug)]
struct TrajectoryRequest {
    id: String,
    timestamp: String,
    position_start: (f64, f64),
    velocity_start: (f64, f64),
    position_end: (f64, f64),
    velocity_end: (f64, f64),
}

fn main() {
    let producer: FutureProducer = rdkafka::config::ClientConfig::new()
        .set("bootstrap.servers", "localhost:9092")
        .create()
        .expect("Producer creation error");

    let request = TrajectoryRequest {
        id: uuid::Uuid::new_v4().to_string(),
        timestamp: Utc::now().to_rfc3339(),
        position_start: (0.0, 0.0),
        velocity_start: (0.0, 0.0),
        position_end: (0.0, 0.0),
        velocity_end: (0.0, 0.0),
    };

    let payload = serde_json::to_string(&request).unwrap();

    producer
        .send(
            FutureRecord::to("trajectory-request").payload(&payload),
            Duration::from_secs(0),
        )
        .await
        .unwrap();

    println!("Sent: {:?}", request);
}
