// Kinematic agent actors - simplified implementation
use tokio::sync::mpsc;
use super::events::KinematicAgentEvent;
use super::aggregate::{Position3D, KinematicsModel};

pub struct KinematicAgentCommandActor {
    event_sender: mpsc::Sender<KinematicAgentEvent>,
}

impl KinematicAgentCommandActor {
    pub fn new(event_sender: mpsc::Sender<KinematicAgentEvent>) -> Self {
        Self { event_sender }
    }

    pub async fn create_agent(&self, agent_id: String, initial_position: Position3D, kinematics_model: KinematicsModel) -> Result<(), String> {
        let event = KinematicAgentEvent::AgentCreated {
            agent_id,
            initial_position,
            kinematics_model,
            timestamp: chrono::Utc::now(),
        };

        self.event_sender
            .send(event)
            .await
            .map_err(|e| format!("Failed to send event: {}", e))
    }
}
