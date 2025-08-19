// Dynamics projections - simplified implementation
use super::aggregate::SimulationState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicsProjection {
    pub simulator_id: String,
    pub current_state: SimulationState,
    pub entities_count: usize,
}
