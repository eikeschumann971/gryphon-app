use crate::common::{AggregateRoot, DomainResult, DomainError};
use serde::{Deserialize, Serialize};
use super::events::PathPlanningEvent;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position2D {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Orientation2D {
    pub angle: f64, // Angle in radians
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteRequest {
    pub request_id: String,
    pub agent_id: String,
    pub start_position: Position2D,
    pub destination_position: Position2D,
    pub start_orientation: Orientation2D,
    pub destination_orientation: Orientation2D,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathPlanWorker {
    pub worker_id: String,
    pub status: WorkerStatus,
    pub algorithm_capabilities: Vec<PlanningAlgorithm>,
    pub last_heartbeat: DateTime<Utc>,
    pub current_plan_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WorkerStatus {
    Idle,
    Busy,
    Offline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanAssignment {
    pub plan_id: String,
    pub worker_id: String,
    pub assigned_at: DateTime<Utc>,
    pub timeout_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathPlanner {
    pub id: String,
    pub algorithm: PlanningAlgorithm,
    pub workspace: Workspace,
    pub active_plans: Vec<PathPlan>,
    pub registered_workers: Vec<PathPlanWorker>,
    pub plan_assignments: Vec<PlanAssignment>,
    pub version: u64,
    #[serde(skip)]
    uncommitted_events: Vec<PathPlanningEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PlanningAlgorithm {
    AStar,
    RRT,
    PRM,
    Dijkstra,
    DynamicWindow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub bounds: WorkspaceBounds,
    pub obstacles: Vec<Obstacle>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceBounds {
    pub min_x: f64, pub max_x: f64,
    pub min_y: f64, pub max_y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Obstacle {
    pub id: String,
    pub shape: ObstacleShape,
    pub position: Position2D,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObstacleShape {
    Circle { radius: f64 },
    Rectangle { width: f64, height: f64 },
    Polygon { vertices: Vec<Position2D> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathPlan {
    pub id: String,
    pub agent_id: String,
    pub start: Position2D,
    pub goal: Position2D,
    pub start_orientation: Orientation2D,
    pub destination_orientation: Orientation2D,
    pub waypoints: Vec<Position2D>,
    pub status: PlanStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PlanStatus {
    Planning,        // Waiting for assignment
    Assigned,        // Assigned to a worker but not started
    InProgress,      // Being processed by a worker
    Complete,        // Successfully completed
    Failed(String),  // Failed with reason
    Executing,       // Being executed by agent
}

impl PathPlanner {
    pub fn new(id: String, algorithm: PlanningAlgorithm) -> Self {
        let mut planner = Self {
            id: id.clone(),
            algorithm: algorithm.clone(),
            workspace: Workspace {
                bounds: WorkspaceBounds {
                    min_x: -100.0, max_x: 100.0,
                    min_y: -100.0, max_y: 100.0,
                },
                obstacles: Vec::new(),
            },
            active_plans: Vec::new(),
            registered_workers: Vec::new(),
            plan_assignments: Vec::new(),
            version: 0,
            uncommitted_events: Vec::new(),
        };

        let event = PathPlanningEvent::PlannerCreated {
            planner_id: id,
            algorithm,
            timestamp: chrono::Utc::now(),
        };
        
        planner.add_event(event);
        planner
    }

    /// Handle a route request command
    pub fn handle_route_request(&mut self, route_request: RouteRequest) -> DomainResult<()> {
        // Validate the route request
        if !self.is_position_in_workspace(&route_request.start_position) {
            return Err(DomainError::InvalidCommand {
                reason: "Start position is outside workspace bounds".to_string(),
            });
        }

        if !self.is_position_in_workspace(&route_request.destination_position) {
            return Err(DomainError::InvalidCommand {
                reason: "Destination position is outside workspace bounds".to_string(),
            });
        }

        // Generate a unique plan ID
        let plan_id = Uuid::new_v4().to_string();

        // Emit the RouteRequested event - let apply() handle state changes
        let event = PathPlanningEvent::RouteRequested {
            planner_id: self.id.clone(),
            request_id: route_request.request_id,
            plan_id: plan_id.clone(),
            agent_id: route_request.agent_id,
            start_position: route_request.start_position,
            destination_position: route_request.destination_position,
            start_orientation: route_request.start_orientation,
            destination_orientation: route_request.destination_orientation,
            timestamp: Utc::now(),
        };

        self.add_event(event.clone());
        self.apply(&event)?; // Apply the event to update the state
        
        // Try to assign the plan to an available worker
        self.try_assign_plan(&plan_id)?;
        
        Ok(())
    }

    /// Register a new worker with the planner
    pub fn register_worker(&mut self, worker_id: String, algorithm_capabilities: Vec<PlanningAlgorithm>) -> DomainResult<()> {
        // Check if worker is already registered
        if self.registered_workers.iter().any(|w| w.worker_id == worker_id) {
            return Err(DomainError::InvalidCommand {
                reason: format!("Worker {} is already registered", worker_id),
            });
        }

        let event = PathPlanningEvent::WorkerRegistered {
            planner_id: self.id.clone(),
            worker_id,
            algorithm_capabilities,
            timestamp: Utc::now(),
        };

        self.add_event(event.clone());
        self.apply(&event)?;
        Ok(())
    }

    /// Handle worker ready signal
    pub fn handle_worker_ready(&mut self, worker_id: String) -> DomainResult<()> {
        // Verify worker is registered
        if !self.registered_workers.iter().any(|w| w.worker_id == worker_id) {
            return Err(DomainError::InvalidCommand {
                reason: format!("Worker {} is not registered", worker_id),
            });
        }

        let event = PathPlanningEvent::WorkerReady {
            planner_id: self.id.clone(),
            worker_id: worker_id.clone(),
            timestamp: Utc::now(),
        };

        self.add_event(event.clone());
        self.apply(&event)?;
        
        // Try to assign work to this newly available worker
        self.try_assign_work_to_worker(&worker_id)?;
        
        Ok(())
    }

    /// Handle plan assignment acceptance
    pub fn handle_plan_assignment_accepted(&mut self, worker_id: String, plan_id: String) -> DomainResult<()> {
        let event = PathPlanningEvent::PlanAssignmentAccepted {
            planner_id: self.id.clone(),
            plan_id,
            worker_id,
            timestamp: Utc::now(),
        };

        self.add_event(event.clone());
        self.apply(&event)?;
        Ok(())
    }

    /// Handle plan completion from worker
    pub fn handle_plan_completed(&mut self, worker_id: String, plan_id: String, waypoints: Vec<Position2D>) -> DomainResult<()> {
        let event = PathPlanningEvent::PlanCompleted {
            planner_id: self.id.clone(),
            plan_id,
            worker_id: Some(worker_id.clone()),
            waypoints,
            timestamp: Utc::now(),
        };

        self.add_event(event.clone());
        self.apply(&event)?;
        
        // Mark worker as ready again
        let ready_event = PathPlanningEvent::WorkerReady {
            planner_id: self.id.clone(),
            worker_id: worker_id.clone(),
            timestamp: Utc::now(),
        };

        self.add_event(ready_event.clone());
        self.apply(&ready_event)?;
        
        // Try to assign new work to this worker
        self.try_assign_work_to_worker(&worker_id)?;
        
        Ok(())
    }

    /// Handle plan failure from worker
    pub fn handle_plan_failed(&mut self, worker_id: String, plan_id: String, reason: String) -> DomainResult<()> {
        let event = PathPlanningEvent::PlanFailed {
            planner_id: self.id.clone(),
            plan_id,
            worker_id: Some(worker_id.clone()),
            reason,
            timestamp: Utc::now(),
        };

        self.add_event(event.clone());
        self.apply(&event)?;
        
        // Mark worker as ready again
        let ready_event = PathPlanningEvent::WorkerReady {
            planner_id: self.id.clone(),
            worker_id: worker_id.clone(),
            timestamp: Utc::now(),
        };

        self.add_event(ready_event.clone());
        self.apply(&ready_event)?;
        
        // Try to assign new work to this worker
        self.try_assign_work_to_worker(&worker_id)?;
        
        Ok(())
    }

    /// Try to assign a specific plan to any available worker
    fn try_assign_plan(&mut self, plan_id: &str) -> DomainResult<()> {
        // Find an idle worker
        if let Some(worker) = self.registered_workers.iter().find(|w| w.status == WorkerStatus::Idle && w.current_plan_id.is_none()) {
            let worker_id = worker.worker_id.clone();
            
            let timeout_seconds = 300; // 5 minutes timeout
            let event = PathPlanningEvent::PlanAssigned {
                planner_id: self.id.clone(),
                plan_id: plan_id.to_string(),
                worker_id,
                timeout_seconds,
                timestamp: Utc::now(),
            };

            self.add_event(event.clone());
            self.apply(&event)?;
        }
        Ok(())
    }

    /// Try to assign work to a specific worker
    fn try_assign_work_to_worker(&mut self, worker_id: &str) -> DomainResult<()> {
        // Find a plan that needs assignment
        if let Some(plan) = self.active_plans.iter().find(|p| p.status == PlanStatus::Planning) {
            let plan_id = plan.id.clone();
            
            let timeout_seconds = 300; // 5 minutes timeout
            let event = PathPlanningEvent::PlanAssigned {
                planner_id: self.id.clone(),
                plan_id,
                worker_id: worker_id.to_string(),
                timeout_seconds,
                timestamp: Utc::now(),
            };

            self.add_event(event.clone());
            self.apply(&event)?;
        }
        Ok(())
    }

    /// Check if a 2D position is within the workspace bounds
    fn is_position_in_workspace(&self, position: &Position2D) -> bool {
        let bounds = &self.workspace.bounds;
        position.x >= bounds.min_x && position.x <= bounds.max_x &&
        position.y >= bounds.min_y && position.y <= bounds.max_y
    }
}

impl AggregateRoot for PathPlanner {
    type Event = PathPlanningEvent;

    fn aggregate_id(&self) -> &str { &self.id }
    fn version(&self) -> u64 { self.version }
    
    fn apply(&mut self, event: &Self::Event) -> DomainResult<()> { 
        match event {
            PathPlanningEvent::PlannerCreated { .. } => {
                // Planner creation is handled in constructor
            }
            PathPlanningEvent::RouteRequested { 
                plan_id, 
                agent_id, 
                start_position, 
                destination_position,
                start_orientation,
                destination_orientation,
                timestamp,
                .. 
            } => {
                // Create new path plan and add to active plans
                let path_plan = PathPlan {
                    id: plan_id.clone(),
                    agent_id: agent_id.clone(),
                    start: start_position.clone(),
                    goal: destination_position.clone(),
                    start_orientation: start_orientation.clone(),
                    destination_orientation: destination_orientation.clone(),
                    waypoints: Vec::new(),
                    status: PlanStatus::Planning,
                    created_at: *timestamp,
                };

                self.active_plans.push(path_plan);
            }
            PathPlanningEvent::WorkerRegistered { worker_id, algorithm_capabilities, timestamp, .. } => {
                let worker = PathPlanWorker {
                    worker_id: worker_id.clone(),
                    status: WorkerStatus::Idle,
                    algorithm_capabilities: algorithm_capabilities.clone(),
                    last_heartbeat: *timestamp,
                    current_plan_id: None,
                };
                self.registered_workers.push(worker);
            }
            PathPlanningEvent::WorkerReady { worker_id, timestamp, .. } => {
                if let Some(worker) = self.registered_workers.iter_mut().find(|w| w.worker_id == *worker_id) {
                    worker.status = WorkerStatus::Idle;
                    worker.last_heartbeat = *timestamp;
                    worker.current_plan_id = None;
                }
            }
            PathPlanningEvent::WorkerBusy { worker_id, plan_id, timestamp, .. } => {
                if let Some(worker) = self.registered_workers.iter_mut().find(|w| w.worker_id == *worker_id) {
                    worker.status = WorkerStatus::Busy;
                    worker.last_heartbeat = *timestamp;
                    worker.current_plan_id = Some(plan_id.clone());
                }
            }
            PathPlanningEvent::WorkerOffline { worker_id, .. } => {
                if let Some(worker) = self.registered_workers.iter_mut().find(|w| w.worker_id == *worker_id) {
                    worker.status = WorkerStatus::Offline;
                    worker.current_plan_id = None;
                }
                // Remove any assignments for this worker
                self.plan_assignments.retain(|a| a.worker_id != *worker_id);
            }
            PathPlanningEvent::PlanAssigned { plan_id, worker_id, timeout_seconds, timestamp, .. } => {
                // Create assignment record
                let assignment = PlanAssignment {
                    plan_id: plan_id.clone(),
                    worker_id: worker_id.clone(),
                    assigned_at: *timestamp,
                    timeout_at: *timestamp + chrono::Duration::seconds(*timeout_seconds as i64),
                };
                self.plan_assignments.push(assignment);

                // Update plan status
                if let Some(plan) = self.active_plans.iter_mut().find(|p| p.id == *plan_id) {
                    plan.status = PlanStatus::Assigned;
                }

                // Update worker status
                if let Some(worker) = self.registered_workers.iter_mut().find(|w| w.worker_id == *worker_id) {
                    worker.current_plan_id = Some(plan_id.clone());
                }
            }
            PathPlanningEvent::PlanAssignmentAccepted { plan_id, worker_id, .. } => {
                // Update plan status to in progress
                if let Some(plan) = self.active_plans.iter_mut().find(|p| p.id == *plan_id) {
                    plan.status = PlanStatus::InProgress;
                }

                // Update worker status to busy
                if let Some(worker) = self.registered_workers.iter_mut().find(|w| w.worker_id == *worker_id) {
                    worker.status = WorkerStatus::Busy;
                }
            }
            PathPlanningEvent::PlanAssignmentRejected { plan_id, worker_id, .. } => {
                // Remove assignment and return plan to planning state
                self.plan_assignments.retain(|a| !(a.plan_id == *plan_id && a.worker_id == *worker_id));
                
                if let Some(plan) = self.active_plans.iter_mut().find(|p| p.id == *plan_id) {
                    plan.status = PlanStatus::Planning;
                }

                // Free up the worker
                if let Some(worker) = self.registered_workers.iter_mut().find(|w| w.worker_id == *worker_id) {
                    worker.current_plan_id = None;
                    worker.status = WorkerStatus::Idle;
                }
            }
            PathPlanningEvent::PlanAssignmentTimedOut { plan_id, worker_id, .. } => {
                // Remove assignment and return plan to planning state
                self.plan_assignments.retain(|a| !(a.plan_id == *plan_id && a.worker_id == *worker_id));
                
                if let Some(plan) = self.active_plans.iter_mut().find(|p| p.id == *plan_id) {
                    plan.status = PlanStatus::Planning;
                }

                // Mark worker as offline (they didn't respond)
                if let Some(worker) = self.registered_workers.iter_mut().find(|w| w.worker_id == *worker_id) {
                    worker.status = WorkerStatus::Offline;
                    worker.current_plan_id = None;
                }
            }
            PathPlanningEvent::PlanRequested { .. } => {
                // Handle legacy plan requested events if needed
            }
            PathPlanningEvent::PlanCompleted { plan_id, waypoints, worker_id, .. } => {
                if let Some(plan) = self.active_plans.iter_mut().find(|p| p.id == *plan_id) {
                    plan.status = PlanStatus::Complete;
                    plan.waypoints = waypoints.clone();
                }
                
                // Remove assignment
                self.plan_assignments.retain(|a| a.plan_id != *plan_id);
                
                // Free up worker if specified
                if let Some(wid) = worker_id {
                    if let Some(worker) = self.registered_workers.iter_mut().find(|w| w.worker_id == *wid) {
                        worker.status = WorkerStatus::Idle;
                        worker.current_plan_id = None;
                    }
                }
            }
            PathPlanningEvent::PlanFailed { plan_id, worker_id, .. } => {
                if let Some(plan) = self.active_plans.iter_mut().find(|p| p.id == *plan_id) {
                    plan.status = PlanStatus::Failed(format!("Planning failed"));
                }
                
                // Remove assignment
                self.plan_assignments.retain(|a| a.plan_id != *plan_id);
                
                // Free up worker if specified
                if let Some(wid) = worker_id {
                    if let Some(worker) = self.registered_workers.iter_mut().find(|w| w.worker_id == *wid) {
                        worker.status = WorkerStatus::Idle;
                        worker.current_plan_id = None;
                    }
                }
            }
        }
        
        self.version += 1; 
        Ok(()) 
    }
    
    fn uncommitted_events(&self) -> &[Self::Event] { &self.uncommitted_events }
    fn mark_events_as_committed(&mut self) { self.uncommitted_events.clear(); }
    fn add_event(&mut self, event: Self::Event) { self.uncommitted_events.push(event); }
}
