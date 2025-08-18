use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use super::types::PlanningAlgorithm;

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
