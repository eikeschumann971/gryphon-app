use esrs::Aggregate;
use serde::{Deserialize, Serialize};
use crate::domains::logical_agent::events::LogicalAgentEvent;
use chrono::Utc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogicalAgentState {
    pub id: String,
    pub name: String,
    pub objectives: Vec<Uuid>,
}

impl Default for LogicalAgentState {
    fn default() -> Self {
        Self { id: String::new(), name: String::new(), objectives: Vec::new() }
    }
}

#[derive(Debug, Clone)]
pub enum LogicalAgentCommand {
    CreateAgent { agent_id: String, name: String },
    AddObjective { agent_id: String, objective_id: Uuid, description: String, priority: u8 },
}

#[derive(Debug, thiserror::Error)]
pub enum LogicalAgentError {
    #[error("invalid command: {0}")]
    InvalidCommand(String),
}

pub struct LogicalAgentAggregate;

impl Aggregate for LogicalAgentAggregate {
    const NAME: &'static str = "logical_agent";
    type State = LogicalAgentState;
    type Command = LogicalAgentCommand;
    type Event = LogicalAgentEvent;
    type Error = LogicalAgentError;

    fn handle_command(_state: &Self::State, command: Self::Command) -> Result<Vec<Self::Event>, Self::Error> {
        match command {
            LogicalAgentCommand::CreateAgent { agent_id, name } => Ok(vec![LogicalAgentEvent::AgentCreated { agent_id, name, timestamp: Utc::now() }]),
            LogicalAgentCommand::AddObjective { agent_id, objective_id, description, priority } => Ok(vec![LogicalAgentEvent::ObjectiveAdded { agent_id, objective_id, description, priority, constraints: vec![], timestamp: Utc::now() }]),
        }
    }

    fn apply_event(_state: Self::State, event: Self::Event) -> Self::State {
        match event {
            LogicalAgentEvent::AgentCreated { agent_id, name, .. } => LogicalAgentState { id: agent_id, name, objectives: Vec::new() },
            LogicalAgentEvent::ObjectiveAdded { agent_id, objective_id, .. } => {
                let mut s = LogicalAgentState::default();
                s.id = agent_id;
                s.objectives.push(objective_id);
                s
            }
            _ => LogicalAgentState::default(),
        }
    }
}
