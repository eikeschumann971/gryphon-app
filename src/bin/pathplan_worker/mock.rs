use gryphon_app::domains::path_planning::*;
use uuid::Uuid;
use chrono::Utc;
use rand::Rng;
use std::time::Duration;
use tokio::time::sleep;

pub async fn simulate_receive_work() -> Option<PathPlanRequest> {
    let mut rng = rand::thread_rng();
    if rng.gen_bool(0.7) {
        Some(PathPlanRequest {
            request_id: format!("req-{}", Uuid::new_v4()),
            agent_id: format!("agent-{}", rng.gen::<u32>() % 5),
            start_position: Position2D { 
                x: rng.gen_range(-50.0..50.0), 
                y: rng.gen_range(-50.0..50.0) 
            },
            destination_position: Position2D { 
                x: rng.gen_range(-50.0..50.0), 
                y: rng.gen_range(-50.0..50.0) 
            },
            start_orientation: Orientation2D { angle: 0.0 },
            destination_orientation: Orientation2D { angle: 1.57 },
            created_at: Utc::now(),
        })
    } else {
        None
    }
}

pub async fn simulate_send_result(worker_id: &str, request_id: &str, waypoints: Vec<Position2D>) {
    println!("📤 Worker {} sending results for request {}", worker_id, request_id);
    println!("   📍 Waypoints:");
    for (i, waypoint) in waypoints.iter().enumerate() {
        println!("      {}. ({:.1}, {:.1})", i + 1, waypoint.x, waypoint.y);
    }
    sleep(Duration::from_millis(100)).await;
    println!("✅ Results sent successfully!");
}
