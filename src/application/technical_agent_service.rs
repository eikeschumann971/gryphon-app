// Technical Agent Service - simplified implementation
use crate::common::ApplicationResult;
use crate::domains::technical_agent::{TechnicalAgentCommandActor, AgentType};

pub struct TechnicalAgentService {
    command_actor: TechnicalAgentCommandActor,
}

impl TechnicalAgentService {
    pub fn new(command_actor: TechnicalAgentCommandActor) -> Self {
        Self { command_actor }
    }

    pub async fn create_agent(&self, agent_id: String, name: String, agent_type: AgentType) -> ApplicationResult<()> {
        self.command_actor
            .create_agent(agent_id, name, agent_type)
            .await
            .map_err(|e| crate::common::ApplicationError::EventStore(e))?;
        Ok(())
    }
}
