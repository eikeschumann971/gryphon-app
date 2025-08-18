// Path Planning Service - simplified implementation
use std::sync::Arc;
use crate::common::ApplicationResult;
use crate::domains::path_planning::{PathPlanningCommandActor, PlanningAlgorithm, PathPlanningDataSource, GraphStoreAsync};
use async_trait::async_trait;

pub struct PathPlanningService {
    command_actor: PathPlanningCommandActor,
    data_source: Arc<dyn PathPlanningDataSource>,
    graph_store: Arc<dyn GraphStoreAsync>,
}

impl PathPlanningService {
    pub fn new(
        command_actor: PathPlanningCommandActor,
        data_source: Arc<dyn PathPlanningDataSource>,
        graph_store: Arc<dyn GraphStoreAsync>,
    ) -> Self {
        Self { command_actor, data_source, graph_store }
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

    pub async fn save_graph_bytes(&self, name: &str, bytes: &[u8]) -> Result<(), crate::common::DomainError> {
        self.graph_store.save_graph_bytes(name, bytes).await
    }

    pub async fn load_graph_bytes_async(&self, name: &str) -> Result<Vec<u8>, crate::common::DomainError> {
        self.graph_store.load_graph_bytes(name).await
    }
}
