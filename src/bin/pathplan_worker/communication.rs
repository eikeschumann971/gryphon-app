use gryphon_app::domains::path_planning::*;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use std::f64::consts::TAU;

/// Communication module for A* worker to interact with the planner service
/// In a real system, this would use proper IPC/network communication
/// For this example, we simulate the communication patterns

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum WorkerMessage {
    RegisterWorker {
        worker_id: String,
        capabilities: Vec<PlanningAlgorithm>,
    },
    WorkerReady {
        worker_id: String,
    },
    AcceptAssignment {
        worker_id: String,
        plan_id: String,
    },
    RejectAssignment {
        worker_id: String,
        plan_id: String,
        reason: String,
    },
    PlanCompleted {
        worker_id: String,
        plan_id: String,
        waypoints: Vec<Position2D>,
    },
    PlanFailed {
        worker_id: String,
        plan_id: String,
        reason: String,
    },
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum PlannerMessage {
    WorkAssignment {
        plan_id: String,
        request: PathPlanRequest,
        timeout_seconds: u64,
    },
    CancelAssignment {
        plan_id: String,
    },
}

#[allow(dead_code)]
pub struct WorkerCommunication {
    worker_id: String,
    message_sender: mpsc::Sender<WorkerMessage>,
    assignment_receiver: mpsc::Receiver<PlannerMessage>,
    assignment_sender: mpsc::Sender<PlannerMessage>,
}

impl WorkerCommunication {
    pub fn new(worker_id: String) -> Self {
        let (message_sender, _message_receiver) = mpsc::channel(100);
        let (assignment_sender, assignment_receiver) = mpsc::channel(100);
        
        Self {
            worker_id,
            message_sender,
            assignment_receiver,
            assignment_sender,
        }
    }
    
    pub async fn register_with_planner(&self, capabilities: Vec<PlanningAlgorithm>) -> Result<(), String> {
        println!("📝 Registering worker {} with planner", self.worker_id);
        println!("   Capabilities: {:?}", capabilities);
        
        let message = WorkerMessage::RegisterWorker {
            worker_id: self.worker_id.clone(),
            capabilities,
        };
        
        // Simulate network communication
        sleep(Duration::from_millis(100)).await;
        
        // In real system, send to planner service
        self.send_to_planner(message).await?;
        
        println!("✅ Registration sent to planner");
        Ok(())
    }
    
    pub async fn signal_ready(&self) -> Result<(), String> {
        println!("📡 Signaling worker {} is ready for assignments", self.worker_id);
        
        let message = WorkerMessage::WorkerReady {
            worker_id: self.worker_id.clone(),
        };
        
        self.send_to_planner(message).await?;
        Ok(())
    }
    
    pub async fn wait_for_assignment(&mut self) -> Option<PlannerMessage> {
        println!("👂 Worker {} waiting for assignment...", self.worker_id);
        
        // Simulate receiving assignment from planner
        // In real system, this would listen on actual communication channel
        self.simulate_assignment_reception().await
    }
    
    pub async fn accept_assignment(&self, plan_id: String) -> Result<(), String> {
        println!("✅ Worker {} accepting assignment for plan {}", self.worker_id, plan_id);
        
        let message = WorkerMessage::AcceptAssignment {
            worker_id: self.worker_id.clone(),
            plan_id,
        };
        
        self.send_to_planner(message).await?;
        Ok(())
    }
    
    pub async fn report_completion(&self, plan_id: String, waypoints: Vec<Position2D>) -> Result<(), String> {
        println!("🎉 Worker {} reporting completion of plan {}", self.worker_id, plan_id);
        println!("   Waypoints: {} points", waypoints.len());
        
        let message = WorkerMessage::PlanCompleted {
            worker_id: self.worker_id.clone(),
            plan_id,
            waypoints,
        };
        
        self.send_to_planner(message).await?;
        Ok(())
    }
    
    pub async fn report_failure(&self, plan_id: String, reason: String) -> Result<(), String> {
        println!("❌ Worker {} reporting failure for plan {}: {}", self.worker_id, plan_id, reason);
        
        let message = WorkerMessage::PlanFailed {
            worker_id: self.worker_id.clone(),
            plan_id,
            reason,
        };
        
        self.send_to_planner(message).await?;
        Ok(())
    }
    
    async fn send_to_planner(&self, message: WorkerMessage) -> Result<(), String> {
        // Simulate network latency
        sleep(Duration::from_millis(50)).await;
        
        // In real system, serialize and send over network/IPC
        println!("📤 Sending to planner: {:?}", message);
        
        // Simulate successful send
        Ok(())
    }
    
    async fn simulate_assignment_reception(&mut self) -> Option<PlannerMessage> {
        use rand::Rng;
        
        // Simulate different timing patterns
        let mut rng = rand::thread_rng();
        let wait_time = rng.gen_range(2..8);
        
        println!("   ⏳ Waiting {} seconds for potential assignment...", wait_time);
        sleep(Duration::from_secs(wait_time)).await;
        
        // 60% chance of receiving an assignment
        if rng.gen_bool(0.6) {
            let plan_id = format!("plan-{}", uuid::Uuid::new_v4());
            let request = self.generate_sample_request().await;
            
            println!("📥 Received assignment for plan: {}", plan_id);
            
            Some(PlannerMessage::WorkAssignment {
                plan_id,
                request,
                timeout_seconds: 300,
            })
        } else {
            println!("   ⏸️  No assignment received, continuing to wait...");
            None
        }
    }
    
    async fn generate_sample_request(&self) -> PathPlanRequest {
        use rand::Rng;
        use chrono::Utc;
        
        let mut rng = rand::thread_rng();
        
        PathPlanRequest {
            request_id: format!("req-{}", uuid::Uuid::new_v4()),
            agent_id: format!("agent-{}", rng.gen_range(1..100)),
            start_position: Position2D {
                x: rng.gen_range(-80.0..80.0),
                y: rng.gen_range(-80.0..80.0),
            },
            destination_position: Position2D {
                x: rng.gen_range(-80.0..80.0),
                y: rng.gen_range(-80.0..80.0),
            },
            start_orientation: Orientation2D {
                angle: rng.gen_range(0.0..TAU),
            },
            destination_orientation: Orientation2D {
                angle: rng.gen_range(0.0..TAU),
            },
            created_at: Utc::now(),
        }
    }
}
