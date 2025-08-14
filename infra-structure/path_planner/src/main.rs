use serde::{Deserialize, Serialize};
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;
use rdkafka::config::ClientConfig;
use chrono::Utc;

#[derive(Serialize, Deserialize, Debug)]
struct TrajectoryRequest {
    id: String,
    timestamp: String,
    position_start: (f64, f64),
    velocity_start: (f64, f64),
    position_end: (f64, f64),
    velocity_end: (f64, f64),
}

#[derive(Serialize, Deserialize, Debug)]
struct TrajectoryReply {
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

fn main() {
    let consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", "path_planner_group")
        .set("bootstrap.servers", "localhost:9092")
        .create()
        .expect("Consumer creation error");

    consumer.subscribe(&["trajectory-request"]).unwrap();

    for message in consumer.iter() {
        if let Ok(m) = message {
            if let Some(payload) = m.payload() {
                let request: TrajectoryRequest = serde_json::from_slice(payload).unwrap();
                println!("Received: {:?}", request);

                let reply = TrajectoryReply {
                    id: request.id.clone(),
                    timestamp: Utc::now().to_rfc3339(),
                    data_points: vec![
                        DataPoint {
                            position: (1.0, 1.0),
                            velocity: (0.5, 0.5),
                            acceleration: (0.1, 0.1),
                        },
                    ],
                };

                println!("Reply: {:?}", reply);
            }
        }
    }
}
