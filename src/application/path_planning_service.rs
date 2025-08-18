// Path Planning Service - simplified implementation
use crate::common::ApplicationResult;
use crate::domains::path_planning::{PathPlanningCommandActor, PlanningAlgorithm};

pub struct PathPlanningService {
    command_actor: PathPlanningCommandActor,
}

impl PathPlanningService {
    pub fn new(command_actor: PathPlanningCommandActor) -> Self {
        Self { command_actor }
    }

    pub async fn create_planner(&self, planner_id: String, algorithm: PlanningAlgorithm) -> ApplicationResult<()> {
        self.command_actor
            .create_planner(planner_id, algorithm)
            .await
            .map_err(|e| crate::common::ApplicationError::EventStore(e))?;
        Ok(())
    }
}
