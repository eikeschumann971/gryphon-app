use crate::common::{ApplicationResult, EventStore, SnapshotStore};
use crate::domains::logical_agent::LogicalAgentCommandActor;
use std::sync::Arc;
use uuid::Uuid;

pub struct LogicalAgentService {
    _event_store: Arc<dyn EventStore + Send + Sync>,
    _snapshot_store: Arc<dyn SnapshotStore + Send + Sync>,
    command_actor: LogicalAgentCommandActor,
}

impl LogicalAgentService {
    pub fn new(
        event_store: Arc<dyn EventStore + Send + Sync>,
        snapshot_store: Arc<dyn SnapshotStore + Send + Sync>,
        command_actor: LogicalAgentCommandActor,
    ) -> Self {
        Self {
            _event_store: event_store,
            _snapshot_store: snapshot_store,
            command_actor,
        }
    }

    pub async fn create_agent(&self, agent_id: String, name: String) -> ApplicationResult<()> {
        self.command_actor
            .create_agent(agent_id, name)
            .await
            .map_err(crate::common::ApplicationError::EventStore)?;

        Ok(())
    }

    pub async fn add_objective(
        &self,
        agent_id: String,
        description: String,
        priority: u8,
        constraints: Vec<String>,
    ) -> ApplicationResult<Uuid> {
        let objective_id = self
            .command_actor
            .add_objective(agent_id, description, priority, constraints)
            .await
            .map_err(crate::common::ApplicationError::EventStore)?;

        Ok(objective_id)
    }

    pub async fn complete_objective(
        &self,
        agent_id: String,
        objective_id: Uuid,
    ) -> ApplicationResult<()> {
        self.command_actor
            .complete_objective(agent_id, objective_id)
            .await
            .map_err(crate::common::ApplicationError::EventStore)?;

        Ok(())
    }

    // Additional service methods would be implemented here
}
