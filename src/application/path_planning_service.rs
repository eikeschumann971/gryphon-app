// Path Planning Service - simplified implementation
use std::sync::Arc;
use crate::common::ApplicationResult;
use crate::domains::path_planning::{PathPlanningCommandActor, PlanningAlgorithm, PathPlanningDataSource};

pub struct PathPlanningService {
    command_actor: PathPlanningCommandActor,
    data_source: Arc<dyn PathPlanningDataSource>,
}

impl PathPlanningService {
    pub fn new(command_actor: PathPlanningCommandActor, data_source: Arc<dyn PathPlanningDataSource>) -> Self {
        Self { command_actor, data_source }
    }

    pub async fn create_planner(&self, planner_id: String, algorithm: PlanningAlgorithm) -> ApplicationResult<()> {
        self.command_actor
            .create_planner(planner_id, algorithm)
            .await
            .map_err(|e| crate::common::ApplicationError::EventStore(e))?;
        Ok(())
    }

    // Example: expose a method that reads map source via the data source
    pub fn load_map_source(&self, name: &str) -> Result<String, crate::common::DomainError> {
        self.data_source.load_geojson(name)
    }
}
