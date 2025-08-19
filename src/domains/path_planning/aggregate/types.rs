use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Position2D {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Orientation2D {
    pub angle: f64, // Angle in radians
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathPlanRequest {
    pub request_id: String,
    pub agent_id: String,
    pub start_position: Position2D,
    pub destination_position: Position2D,
    pub start_orientation: Orientation2D,
    pub destination_orientation: Orientation2D,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PlanningAlgorithm {
    AStar,
    RRT,
    PRM,
    Dijkstra,
    DynamicWindow,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlanningScenario {
    pub name: String,
    pub description: String,
    pub agent_id: String,
    pub start_position: Position2D,
    pub destination_position: Position2D,
    pub start_orientation: Orientation2D,
    pub destination_orientation: Orientation2D,
}
