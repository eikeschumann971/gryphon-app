use esrs::Aggregate;
use serde::{Deserialize, Serialize};
use crate::domains::technical_agent::events::TechnicalAgentEvent;
use crate::domains::technical_agent::aggregate::AgentType;
use chrono::Utc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalAgentState {
    pub id: String,
    pub name: String,
    pub agent_type: Option<AgentType>,
}

impl Default for TechnicalAgentState {
    fn default() -> Self {
        Self { id: String::new(), name: String::new(), agent_type: None }
    }
}

#[derive(Debug, Clone)]
pub enum TechnicalAgentCommand {
    CreateAgent { agent_id: String, name: String, agent_type: AgentType },
    AddCapability { agent_id: String, capability_id: Uuid, name: String, description: String },
}

#[derive(Debug, thiserror::Error)]
pub enum TechnicalAgentError {
    #[error("invalid command: {0}")]
    InvalidCommand(String),
}

pub struct TechnicalAgentAggregate;

impl Aggregate for TechnicalAgentAggregate {
    const NAME: &'static str = "technical_agent";
    type State = TechnicalAgentState;
    type Command = TechnicalAgentCommand;
    type Event = TechnicalAgentEvent;
    type Error = TechnicalAgentError;

    fn handle_command(_state: &Self::State, command: Self::Command) -> Result<Vec<Self::Event>, Self::Error> {
        match command {
            TechnicalAgentCommand::CreateAgent { agent_id, name, agent_type } => Ok(vec![TechnicalAgentEvent::AgentCreated { agent_id, name, agent_type, timestamp: Utc::now() }]),
            TechnicalAgentCommand::AddCapability { agent_id, capability_id, name, description } => Ok(vec![TechnicalAgentEvent::CapabilityAdded { agent_id, capability_id, name, description, timestamp: Utc::now() }]),
        }
    }

    fn apply_event(_state: Self::State, event: Self::Event) -> Self::State {
        match event {
            TechnicalAgentEvent::AgentCreated { agent_id, name, agent_type, .. } => TechnicalAgentState { id: agent_id, name, agent_type: Some(agent_type) },
            _ => TechnicalAgentState::default(),
        }
    }
}
