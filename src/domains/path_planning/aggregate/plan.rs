use super::types::{Orientation2D, Position2D};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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
    Planning,       // Waiting for assignment
    Assigned,       // Assigned to a worker but not started
    InProgress,     // Being processed by a worker
    Complete,       // Successfully completed
    Failed(String), // Failed with reason
    Executing,      // Being executed by agent
}
