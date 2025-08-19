// Dynamics Service - simplified implementation
use crate::common::ApplicationResult;
use crate::domains::dynamics::{DynamicsCommandActor, PhysicsModel};

pub struct DynamicsService {
    command_actor: DynamicsCommandActor,
}

impl DynamicsService {
    pub fn new(command_actor: DynamicsCommandActor) -> Self {
        Self { command_actor }
    }

    pub async fn create_simulator(
        &self,
        simulator_id: String,
        physics_model: PhysicsModel,
    ) -> ApplicationResult<()> {
        self.command_actor
            .create_simulator(simulator_id, physics_model)
            .await
            .map_err(crate::common::ApplicationError::EventStore)?;
        Ok(())
    }
}
