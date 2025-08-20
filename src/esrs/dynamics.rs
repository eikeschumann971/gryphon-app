use esrs::Aggregate;
use serde::{Deserialize, Serialize};
use crate::domains::dynamics::events::DynamicsEvent;
use chrono::Utc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicsState {
    pub id: String,
    pub simulation_state: crate::domains::dynamics::aggregate::SimulationState,
}

impl Default for DynamicsState {
    fn default() -> Self {
        Self {
            id: String::new(),
            simulation_state: crate::domains::dynamics::aggregate::SimulationState::Stopped,
        }
    }
}

#[derive(Debug, Clone)]
pub enum DynamicsCommand {
    CreateSimulator { simulator_id: String, physics_model: crate::domains::dynamics::aggregate::PhysicsModel },
    Start { simulator_id: String },
    Stop { simulator_id: String },
    UpdateState { simulator_id: String, new_state: crate::domains::dynamics::aggregate::SimulationState },
}

#[derive(Debug, thiserror::Error)]
pub enum DynamicsError {
    #[error("invalid command: {0}")]
    InvalidCommand(String),
}

pub struct DynamicsAggregate;

impl Aggregate for DynamicsAggregate {
    const NAME: &'static str = "dynamics_simulator";
    type State = DynamicsState;
    type Command = DynamicsCommand;
    type Event = DynamicsEvent;
    type Error = DynamicsError;

    fn handle_command(state: &Self::State, command: Self::Command) -> Result<Vec<Self::Event>, Self::Error> {
        match command {
            DynamicsCommand::CreateSimulator { simulator_id, physics_model } => Ok(vec![DynamicsEvent::SimulatorCreated { simulator_id, physics_model, timestamp: Utc::now() }]),
            DynamicsCommand::Start { simulator_id } => {
                if matches!(state.simulation_state, crate::domains::dynamics::aggregate::SimulationState::Running) {
                    return Err(DynamicsError::InvalidCommand("Simulator already running".to_string()));
                }
                Ok(vec![DynamicsEvent::SimulationStarted { simulator_id, timestamp: Utc::now() }])
            }
            DynamicsCommand::Stop { simulator_id } => Ok(vec![DynamicsEvent::SimulationStopped { simulator_id, timestamp: Utc::now() }]),
            DynamicsCommand::UpdateState { simulator_id, new_state } => Ok(vec![DynamicsEvent::StateUpdated { simulator_id, new_state, timestamp: Utc::now() }]),
        }
    }

    fn apply_event(_state: Self::State, event: Self::Event) -> Self::State {
        match event {
            DynamicsEvent::SimulatorCreated { simulator_id, .. } => DynamicsState { id: simulator_id, simulation_state: crate::domains::dynamics::aggregate::SimulationState::Stopped },
            DynamicsEvent::SimulationStarted { simulator_id, .. } => DynamicsState { id: simulator_id, simulation_state: crate::domains::dynamics::aggregate::SimulationState::Running },
            DynamicsEvent::SimulationStopped { simulator_id, .. } => DynamicsState { id: simulator_id, simulation_state: crate::domains::dynamics::aggregate::SimulationState::Stopped },
            DynamicsEvent::StateUpdated { simulator_id, new_state, .. } => DynamicsState { id: simulator_id, simulation_state: new_state },
        }
    }
}
