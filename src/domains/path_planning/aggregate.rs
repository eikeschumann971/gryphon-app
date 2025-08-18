use crate::common::{AggregateRoot, DomainResult};
use serde::{Deserialize, Serialize};
use super::events::PathPlanningEvent;
use crate::domains::kinematic_agent::Position3D;

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
}

impl AggregateRoot for PathPlanner {
    type Event = PathPlanningEvent;

    fn aggregate_id(&self) -> &str { &self.id }
    fn version(&self) -> u64 { self.version }
    fn apply(&mut self, _event: &Self::Event) -> DomainResult<()> { 
        self.version += 1; 
        Ok(()) 
    }
    fn uncommitted_events(&self) -> &[Self::Event] { &self.uncommitted_events }
    fn mark_events_as_committed(&mut self) { self.uncommitted_events.clear(); }
    fn add_event(&mut self, event: Self::Event) { self.uncommitted_events.push(event); }
}
