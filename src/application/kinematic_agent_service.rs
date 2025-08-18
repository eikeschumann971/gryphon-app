// Kinematic Agent Service - simplified implementation
use crate::common::ApplicationResult;
use crate::domains::kinematic_agent::{KinematicAgentCommandActor, Position3D, KinematicsModel};

pub struct KinematicAgentService {
    command_actor: KinematicAgentCommandActor,
}

impl KinematicAgentService {
    pub fn new(command_actor: KinematicAgentCommandActor) -> Self {
        Self { command_actor }
    }

    pub async fn create_agent(&self, agent_id: String, initial_position: Position3D, kinematics_model: KinematicsModel) -> ApplicationResult<()> {
        self.command_actor
            .create_agent(agent_id, initial_position, kinematics_model)
            .await
            .map_err(crate::common::ApplicationError::EventStore)?;
        Ok(())
    }
}
