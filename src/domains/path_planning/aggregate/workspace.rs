use serde::{Deserialize, Serialize};
use super::types::Position2D;

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
