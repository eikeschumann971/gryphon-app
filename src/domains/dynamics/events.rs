use crate::common::DomainEvent;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use super::aggregate::{PhysicsModel, SimulationState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DynamicsEvent {
    SimulatorCreated {
        simulator_id: String,
        physics_model: PhysicsModel,
        timestamp: DateTime<Utc>,
    },
    SimulationStarted {
        simulator_id: String,
        timestamp: DateTime<Utc>,
    },
    SimulationStopped {
        simulator_id: String,
        timestamp: DateTime<Utc>,
    },
    StateUpdated {
        simulator_id: String,
        new_state: SimulationState,
        timestamp: DateTime<Utc>,
    },
}

impl DomainEvent for DynamicsEvent {
    fn event_type(&self) -> &'static str {
        match self {
            DynamicsEvent::SimulatorCreated { .. } => "SimulatorCreated",
            DynamicsEvent::SimulationStarted { .. } => "SimulationStarted",
            DynamicsEvent::SimulationStopped { .. } => "SimulationStopped",
            DynamicsEvent::StateUpdated { .. } => "StateUpdated",
        }
    }

    fn aggregate_id(&self) -> &str {
        match self {
            DynamicsEvent::SimulatorCreated { simulator_id, .. } => simulator_id,
            DynamicsEvent::SimulationStarted { simulator_id, .. } => simulator_id,
            DynamicsEvent::SimulationStopped { simulator_id, .. } => simulator_id,
            DynamicsEvent::StateUpdated { simulator_id, .. } => simulator_id,
        }
    }

    fn event_version(&self) -> u64 { 1 }

    fn occurred_at(&self) -> DateTime<Utc> {
        match self {
            DynamicsEvent::SimulatorCreated { timestamp, .. } => *timestamp,
            DynamicsEvent::SimulationStarted { timestamp, .. } => *timestamp,
            DynamicsEvent::SimulationStopped { timestamp, .. } => *timestamp,
            DynamicsEvent::StateUpdated { timestamp, .. } => *timestamp,
        }
    }
}
