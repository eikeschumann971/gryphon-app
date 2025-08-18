// Dynamics actors - simplified implementation
use tokio::sync::mpsc;
use super::events::DynamicsEvent;
use super::aggregate::PhysicsModel;

pub struct DynamicsCommandActor {
    event_sender: mpsc::Sender<DynamicsEvent>,
}

impl DynamicsCommandActor {
    pub fn new(event_sender: mpsc::Sender<DynamicsEvent>) -> Self {
        Self { event_sender }
    }

    pub async fn create_simulator(&self, simulator_id: String, physics_model: PhysicsModel) -> Result<(), String> {
        let event = DynamicsEvent::SimulatorCreated {
            simulator_id,
            physics_model,
            timestamp: chrono::Utc::now(),
        };

        self.event_sender
            .send(event)
            .await
            .map_err(|e| format!("Failed to send event: {}", e))
    }
}
