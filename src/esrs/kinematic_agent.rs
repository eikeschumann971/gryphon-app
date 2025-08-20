use esrs::Aggregate;
use serde::{Deserialize, Serialize};
use crate::domains::kinematic_agent::events::KinematicAgentEvent;
use crate::domains::kinematic_agent::aggregate::{Position3D, Velocity3D, KinematicsModel};
use chrono::Utc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KinematicAgentState {
    pub id: String,
    pub position: Option<Position3D>,
    pub velocity: Option<Velocity3D>,
    pub model: Option<KinematicsModel>,
}

impl Default for KinematicAgentState {
    fn default() -> Self {
        Self { id: String::new(), position: None, velocity: None, model: None }
    }
}

#[derive(Debug, Clone)]
pub enum KinematicAgentCommand {
    CreateAgent { agent_id: String, initial_position: Position3D, model: KinematicsModel },
    UpdatePosition { agent_id: String, new_position: Position3D },
    UpdateVelocity { agent_id: String, new_velocity: Velocity3D },
}

#[derive(Debug, thiserror::Error)]
pub enum KinematicAgentError {
    #[error("invalid command: {0}")]
    InvalidCommand(String),
}

pub struct KinematicAgentAggregate;

impl Aggregate for KinematicAgentAggregate {
    const NAME: &'static str = "kinematic_agent";
    type State = KinematicAgentState;
    type Command = KinematicAgentCommand;
    type Event = KinematicAgentEvent;
    type Error = KinematicAgentError;

    fn handle_command(_state: &Self::State, command: Self::Command) -> Result<Vec<Self::Event>, Self::Error> {
        match command {
            KinematicAgentCommand::CreateAgent { agent_id, initial_position, model } => Ok(vec![KinematicAgentEvent::AgentCreated { agent_id, initial_position, kinematics_model: model, timestamp: Utc::now() }]),
            KinematicAgentCommand::UpdatePosition { agent_id, new_position } => Ok(vec![KinematicAgentEvent::PositionUpdated { agent_id, new_position, timestamp: Utc::now() }]),
            KinematicAgentCommand::UpdateVelocity { agent_id, new_velocity } => Ok(vec![KinematicAgentEvent::VelocityUpdated { agent_id, new_velocity, timestamp: Utc::now() }]),
        }
    }

    fn apply_event(_state: Self::State, event: Self::Event) -> Self::State {
        match event {
            KinematicAgentEvent::AgentCreated { agent_id, initial_position, kinematics_model, .. } => KinematicAgentState { id: agent_id, position: Some(initial_position), velocity: None, model: Some(kinematics_model) },
            KinematicAgentEvent::PositionUpdated { agent_id, new_position, .. } => {
                let mut s = KinematicAgentState::default();
                s.id = agent_id;
                s.position = Some(new_position);
                s
            }
            KinematicAgentEvent::VelocityUpdated { agent_id, new_velocity, .. } => {
                let mut s = KinematicAgentState::default();
                s.id = agent_id;
                s.velocity = Some(new_velocity);
                s
            }
            _ => KinematicAgentState::default(),
        }
    }
}
