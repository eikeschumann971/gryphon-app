// Path planning actors - simplified implementation
use super::aggregate::PlanningAlgorithm;
use super::events::PathPlanningEvent;
use tokio::sync::mpsc;

pub struct PathPlanningCommandActor {
    event_sender: mpsc::Sender<PathPlanningEvent>,
}

impl PathPlanningCommandActor {
    pub fn new(event_sender: mpsc::Sender<PathPlanningEvent>) -> Self {
        Self { event_sender }
    }

    pub async fn create_planner(
        &self,
        planner_id: String,
        algorithm: PlanningAlgorithm,
    ) -> Result<(), String> {
        let event = PathPlanningEvent::PlannerCreated {
            planner_id,
            algorithm,
            timestamp: chrono::Utc::now(),
        };

        self.event_sender
            .send(event)
            .await
            .map_err(|e| format!("Failed to send event: {}", e))
    }
}
