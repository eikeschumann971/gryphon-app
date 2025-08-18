// Dynamics projections - simplified implementation
use serde::{Deserialize, Serialize};
use super::aggregate::SimulationState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicsProjection {
    pub simulator_id: String,
    pub current_state: SimulationState,
    pub entities_count: usize,
}
