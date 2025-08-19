// GUI actors - simplified implementation
use super::events::GUIEvent;
use tokio::sync::mpsc;

pub struct GUICommandActor {
    event_sender: mpsc::Sender<GUIEvent>,
}

impl GUICommandActor {
    pub fn new(event_sender: mpsc::Sender<GUIEvent>) -> Self {
        Self { event_sender }
    }

    pub async fn create_application(&self, app_id: String, name: String) -> Result<(), String> {
        let event = GUIEvent::ApplicationCreated {
            app_id,
            name,
            timestamp: chrono::Utc::now(),
        };

        self.event_sender
            .send(event)
            .await
            .map_err(|e| format!("Failed to send event: {}", e))
    }
}
