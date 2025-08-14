use serde::{Deserialize, Serialize};
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;
use rdkafka::config::ClientConfig;
use chrono::Utc;
use tokio::sync::mpsc;
use tokio::task;
use std::error::Error;
use rdkafka::producer::{FutureProducer, FutureRecord};

#[derive(Serialize, Deserialize, Debug)]
struct EventTrajectoryRequest {
    id: String,
    timestamp: String,
    position_start: (f64, f64),
    velocity_start: (f64, f64),
    position_end: (f64, f64),
    velocity_end: (f64, f64),
}

#[derive(Serialize, Deserialize, Debug)]
struct EventTrajectoryReply {
    id: String,
    timestamp: String,
    data_points: Vec<DataPoint>,
}

#[derive(Serialize, Deserialize, Debug)]
struct DataPoint {
    position: (f64, f64),
    velocity: (f64, f64),
    acceleration: (f64, f64),
}

#[derive(Debug)]
enum TrajectoryResult {
    Success(EventTrajectoryReply),
    Error(String),
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let (tx, mut rx) = mpsc::channel::<EventTrajectoryRequest>(100);

    let consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", "path_planner_group")
        .set("bootstrap.servers", "localhost:9092")
        .create()
        .expect("Consumer creation error");

    consumer.subscribe(&["events-trajectory"]).unwrap();

    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", "localhost:9092")
        .create()
        .expect("Producer creation error");

    let tx_clone = tx.clone();
    task::spawn(async move {
        for message in consumer.iter() {
            if let Ok(m) = message {
                if let Some(payload) = m.payload() {
                    let request: EventTrajectoryRequest = serde_json::from_slice(payload).unwrap();
                    println!("Received: {:?}", request);

                    tx_clone.send(request).await.unwrap();
                }
            }
        }
    });

    while let Some(internal_request) = rx.recv().await {
        println!("Processing internal request: {:?}", internal_request);

        let result: TrajectoryResult = compute_trajectory(internal_request).await;

        match result {
            TrajectoryResult::Success(reply) => {
                let payload = serde_json::to_string(&reply).unwrap();
                producer
                    .send(
                        FutureRecord::to("events-trajectory").payload(&payload),
                        0,
                    )
                    .await
                    .unwrap();
            }
            TrajectoryResult::Error(err) => println!("Error computing trajectory: {}", err),
        }
    }

    Ok(())
}

async fn compute_trajectory(request: EventTrajectoryRequest) -> TrajectoryResult {
    // Simulate computation
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    if request.id.is_empty() {
        TrajectoryResult::Error("Invalid request ID".to_string())
    } else {
        TrajectoryResult::Success(EventTrajectoryReply {
            id: request.id,
            timestamp: Utc::now().to_rfc3339(),
            data_points: vec![
                DataPoint {
                    position: (1.0, 1.0),
                    velocity: (0.5, 0.5),
                    acceleration: (0.1, 0.1),
                },
            ],
        })
    }
}
