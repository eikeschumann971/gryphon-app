// Path planning projections - simplified implementation
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathPlanningProjection {
    pub planner_id: String,
    pub active_plans_count: usize,
    pub completed_plans_count: usize,
    pub failed_plans_count: usize,
}
