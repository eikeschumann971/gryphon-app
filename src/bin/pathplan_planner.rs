use gryphon_app::domains::path_planning::*;
use gryphon_app::common::*;
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};
use std::collections::HashMap;
use uuid::Uuid;

/// Path Planning Planner Process
/// 
/// This process manages PathPlanner aggregates and coordinates between clients and workers.
/// It handles:
/// - Creating and managing PathPlanner instances
/// - Processing path plan requests from clients
/// - Managing worker registrations and assignments
/// - Broadcasting events to interested parties
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🗺️  Starting Path Planning Planner Service");
    
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    let mut planner_service = PathPlannerService::new().await;
    planner_service.run().await?;
    
    Ok(())
}

pub struct PathPlannerService {
    planners: HashMap<String, PathPlanner>,
    request_receiver: mpsc::Receiver<PathPlanRequest>,
    request_sender: mpsc::Sender<PathPlanRequest>,
    worker_events_receiver: mpsc::Receiver<WorkerEvent>,
    worker_events_sender: mpsc::Sender<WorkerEvent>,
    client_responses_sender: mpsc::Sender<PlanResponse>,
}

#[derive(Debug, Clone)]
pub enum WorkerEvent {
    WorkerRegistered {
        worker_id: String,
        capabilities: Vec<PlanningAlgorithm>,
    },
    WorkerReady {
        worker_id: String,
    },
    PlanAssignmentAccepted {
        worker_id: String,
        plan_id: String,
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
pub struct PlanResponse {
    pub request_id: String,
    pub plan_id: String,
    pub status: PlanResponseStatus,
    pub waypoints: Option<Vec<Position2D>>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone)]
pub enum PlanResponseStatus {
    Accepted,
    InProgress,
    Completed,
    Failed,
}

impl PathPlannerService {
    pub async fn new() -> Self {
        let (request_sender, request_receiver) = mpsc::channel(100);
        let (worker_events_sender, worker_events_receiver) = mpsc::channel(100);
        let (client_responses_sender, _client_responses_receiver) = mpsc::channel(100);
        
        let mut planners = HashMap::new();
        
        // Create a default planner
        let main_planner = PathPlanner::new("main-planner".to_string(), PlanningAlgorithm::AStar);
        planners.insert("main-planner".to_string(), main_planner);
        
        println!("✅ Created main PathPlanner with A* algorithm");
        
        Self {
            planners,
            request_receiver,
            request_sender,
            worker_events_receiver,
            worker_events_sender,
            client_responses_sender,
        }
    }
    
    pub fn get_request_sender(&self) -> mpsc::Sender<PathPlanRequest> {
        self.request_sender.clone()
    }
    
    pub fn get_worker_events_sender(&self) -> mpsc::Sender<WorkerEvent> {
        self.worker_events_sender.clone()
    }
    
    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🚀 Path Planning Planner Service is running");
        println!("📡 Listening for path plan requests and worker events...");
        
        // Set up a heartbeat timer
        let mut heartbeat = interval(Duration::from_secs(30));
        
        loop {
            tokio::select! {
                // Handle incoming path plan requests
                Some(request) = self.request_receiver.recv() => {
                    self.handle_path_plan_request(request).await?;
                }
                
                // Handle worker events
                Some(worker_event) = self.worker_events_receiver.recv() => {
                    self.handle_worker_event(worker_event).await?;
                }
                
                // Periodic heartbeat and status update
                _ = heartbeat.tick() => {
                    self.print_status().await;
                }
                
                // Handle shutdown signal
                _ = tokio::signal::ctrl_c() => {
                    println!("🛑 Shutting down Path Planning Planner Service");
                    break;
                }
            }
        }
        
        Ok(())
    }
    
    async fn handle_path_plan_request(&mut self, request: PathPlanRequest) -> DomainResult<()> {
        println!("📥 Received path plan request: {}", request.request_id);
        println!("   Agent: {}", request.agent_id);
        println!("   From: ({:.1}, {:.1}) -> To: ({:.1}, {:.1})", 
                 request.start_position.x, request.start_position.y,
                 request.destination_position.x, request.destination_position.y);
        
        // For now, use the main planner. In a real system, you might have planner selection logic
        let planner = self.planners.get_mut("main-planner")
            .ok_or_else(|| DomainError::InvalidCommand { 
                reason: "Main planner not found".to_string() 
            })?;
        
        // Request the path plan
        match planner.request_path_plan(request.clone()) {
            Ok(()) => {
                println!("✅ Path plan request accepted for agent: {}", request.agent_id);
                
                // Send acceptance response
                let response = PlanResponse {
                    request_id: request.request_id.clone(),
                    plan_id: format!("plan-{}", Uuid::new_v4()),
                    status: PlanResponseStatus::Accepted,
                    waypoints: None,
                    error_message: None,
                };
                
                if let Err(e) = self.client_responses_sender.send(response).await {
                    eprintln!("Failed to send response to client: {}", e);
                }
                
                // Process any events that were generated
                let events: Vec<_> = planner.uncommitted_events().to_vec();
                for event in events {
                    println!("📝 Event: {:?}", event.event_type());
                }
                planner.mark_events_as_committed();
            }
            Err(e) => {
                println!("❌ Path plan request rejected: {:?}", e);
                
                // Send rejection response
                let response = PlanResponse {
                    request_id: request.request_id,
                    plan_id: String::new(),
                    status: PlanResponseStatus::Failed,
                    waypoints: None,
                    error_message: Some(format!("{:?}", e)),
                };
                
                if let Err(e) = self.client_responses_sender.send(response).await {
                    eprintln!("Failed to send error response to client: {}", e);
                }
            }
        }
        
        Ok(())
    }
    
    async fn handle_worker_event(&mut self, worker_event: WorkerEvent) -> DomainResult<()> {
        println!("🔧 Received worker event: {:?}", worker_event);
        
        let planner = self.planners.get_mut("main-planner")
            .ok_or_else(|| DomainError::InvalidCommand { 
                reason: "Main planner not found".to_string() 
            })?;
        
        match worker_event {
            WorkerEvent::WorkerRegistered { worker_id, capabilities } => {
                planner.register_worker(worker_id.clone(), capabilities)?;
                println!("✅ Worker {} registered", worker_id);
            }
            
            WorkerEvent::WorkerReady { worker_id } => {
                planner.handle_worker_ready(worker_id.clone())?;
                println!("✅ Worker {} is ready", worker_id);
            }
            
            WorkerEvent::PlanAssignmentAccepted { worker_id, plan_id } => {
                planner.handle_plan_assignment_accepted(worker_id.clone(), plan_id.clone())?;
                println!("✅ Worker {} accepted plan {}", worker_id, plan_id);
            }
            
            WorkerEvent::PlanCompleted { worker_id, plan_id, waypoints } => {
                planner.handle_plan_completed(worker_id.clone(), plan_id.clone(), waypoints.clone())?;
                println!("🎉 Worker {} completed plan {} with {} waypoints", 
                         worker_id, plan_id, waypoints.len());
                
                // Send completion response to client
                if let Some(_plan) = planner.active_plans.iter().find(|p| p.id == plan_id) {
                    let response = PlanResponse {
                        request_id: format!("req-for-{}", plan_id), // In real system, track this mapping
                        plan_id: plan_id.clone(),
                        status: PlanResponseStatus::Completed,
                        waypoints: Some(waypoints),
                        error_message: None,
                    };
                    
                    if let Err(e) = self.client_responses_sender.send(response).await {
                        eprintln!("Failed to send completion response: {}", e);
                    }
                }
            }
            
            WorkerEvent::PlanFailed { worker_id, plan_id, reason } => {
                planner.handle_plan_failed(worker_id.clone(), plan_id.clone(), reason.clone())?;
                println!("❌ Worker {} failed plan {}: {}", worker_id, plan_id, reason);
                
                // Send failure response to client
                let response = PlanResponse {
                    request_id: format!("req-for-{}", plan_id), // In real system, track this mapping
                    plan_id: plan_id.clone(),
                    status: PlanResponseStatus::Failed,
                    waypoints: None,
                    error_message: Some(reason),
                };
                
                if let Err(e) = self.client_responses_sender.send(response).await {
                    eprintln!("Failed to send failure response: {}", e);
                }
            }
        }
        
        // Process any events that were generated
        let events: Vec<_> = planner.uncommitted_events().to_vec();
        for event in events {
            println!("📝 Event: {:?}", event.event_type());
        }
        planner.mark_events_as_committed();
        
        Ok(())
    }
    
    async fn print_status(&self) {
        let planner = &self.planners["main-planner"];
        println!("📊 Status Report:");
        println!("   🔧 Registered workers: {}", planner.registered_workers.len());
        println!("   📋 Active plans: {}", planner.active_plans.len());
        println!("   🎯 Plan assignments: {}", planner.plan_assignments.len());
        
        for worker in &planner.registered_workers {
            println!("   Worker {}: {:?} (Plan: {:?})", 
                     worker.worker_id, worker.status, worker.current_plan_id);
        }
        
        for plan in &planner.active_plans {
            println!("   Plan {}: {:?} (Agent: {})", 
                     plan.id, plan.status, plan.agent_id);
        }
    }
}
