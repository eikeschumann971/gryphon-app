use crate::common::{AggregateRoot, DomainResult, DomainError};
use serde::{Deserialize, Serialize};
use super::events::PathPlanningEvent;
use crate::domains::kinematic_agent::Position3D;
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
pub struct PathPlanner {
    pub id: String,
    pub algorithm: PlanningAlgorithm,
    pub workspace: Workspace,
    pub active_plans: Vec<PathPlan>,
    pub version: u64,
    #[serde(skip)]
    uncommitted_events: Vec<PathPlanningEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub min_z: f64, pub max_z: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Obstacle {
    pub id: String,
    pub shape: ObstacleShape,
    pub position: Position3D,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObstacleShape {
    Sphere { radius: f64 },
    Box { width: f64, height: f64, depth: f64 },
    Cylinder { radius: f64, height: f64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathPlan {
    pub id: String,
    pub agent_id: String,
    pub start: Position3D,
    pub goal: Position3D,
    pub waypoints: Vec<Position3D>,
    pub status: PlanStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlanStatus {
    Planning,
    Complete,
    Failed(String),
    Executing,
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
                    min_z: 0.0, max_z: 50.0,
                },
                obstacles: Vec::new(),
            },
            active_plans: Vec::new(),
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

        // Convert 2D positions to 3D for internal processing (assuming z=0 for ground-level planning)
        let start_3d = Position3D {
            x: route_request.start_position.x,
            y: route_request.start_position.y,
            z: 0.0,
            timestamp: route_request.created_at,
        };

        let goal_3d = Position3D {
            x: route_request.destination_position.x,
            y: route_request.destination_position.y,
            z: 0.0,
            timestamp: route_request.created_at,
        };

        // Create and add the new plan to active plans
        let _path_plan = PathPlan {
            id: plan_id.clone(),
            agent_id: route_request.agent_id.clone(),
            start: start_3d.clone(),
            goal: goal_3d.clone(),
            waypoints: Vec::new(), // Will be populated when planning completes
            status: PlanStatus::Planning,
        };

        // Emit the RouteRequested event
        let event = PathPlanningEvent::RouteRequested {
            planner_id: self.id.clone(),
            request_id: route_request.request_id,
            plan_id,
            agent_id: route_request.agent_id,
            start_position: route_request.start_position,
            destination_position: route_request.destination_position,
            start_orientation: route_request.start_orientation,
            destination_orientation: route_request.destination_orientation,
            timestamp: Utc::now(),
        };

        self.add_event(event.clone());
        self.apply(&event)?; // Apply the event to update the state
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
                timestamp,
                .. 
            } => {
                // Convert 2D positions to 3D for internal storage
                let start_3d = Position3D {
                    x: start_position.x,
                    y: start_position.y,
                    z: 0.0,
                    timestamp: *timestamp,
                };

                let goal_3d = Position3D {
                    x: destination_position.x,
                    y: destination_position.y,
                    z: 0.0,
                    timestamp: *timestamp,
                };

                // Create new path plan and add to active plans
                let path_plan = PathPlan {
                    id: plan_id.clone(),
                    agent_id: agent_id.clone(),
                    start: start_3d,
                    goal: goal_3d,
                    waypoints: Vec::new(),
                    status: PlanStatus::Planning,
                };

                self.active_plans.push(path_plan);
            }
            PathPlanningEvent::PlanRequested { .. } => {
                // Handle legacy plan requested events if needed
            }
            PathPlanningEvent::PlanCompleted { plan_id, waypoints, .. } => {
                if let Some(plan) = self.active_plans.iter_mut().find(|p| p.id == *plan_id) {
                    plan.status = PlanStatus::Complete;
                    plan.waypoints = waypoints.clone();
                }
            }
            PathPlanningEvent::PlanFailed { plan_id, reason, .. } => {
                if let Some(plan) = self.active_plans.iter_mut().find(|p| p.id == *plan_id) {
                    plan.status = PlanStatus::Failed(reason.clone());
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
