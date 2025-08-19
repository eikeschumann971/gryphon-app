use crate::adapters::inbound::file_event_store::FileEventStore;
use crate::common::EventStore;
use crate::config::Config;
use crate::domains::path_planning::aggregate::types::PlanningScenario;
use crate::domains::path_planning::aggregate::types::{Orientation2D, Position2D};
use std::f64::consts::PI;
use std::sync::Arc;

pub struct PathPlanClient {
    pub scenarios: Vec<PlanningScenario>,
    pub event_store: Arc<dyn EventStore>,
    pub planner_id: String,
    pub logger: crate::domains::DynLogger,
}

impl PathPlanClient {
    pub async fn new(
        logger: crate::domains::DynLogger,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let _config = Config::default();
        logger.info("Using default configuration for demo");

        let event_store: Arc<dyn EventStore> = Arc::new(FileEventStore::new("/tmp/gryphon-events"));
        logger.info("Using file-based event store for demo (shared between processes)");

        let planner_id = "main-path-planner".to_string();
        let scenarios = vec![PlanningScenario {
            name: "Office Navigation".to_string(),
            description: "Robot navigating from office entrance to meeting room".to_string(),
            agent_id: "office-robot-001".to_string(),
            start_position: Position2D { x: -50.0, y: -30.0 },
            destination_position: Position2D { x: 40.0, y: 25.0 },
            start_orientation: Orientation2D { angle: 0.0 },
            destination_orientation: Orientation2D {
                angle: 90.0 / 180.0 * PI,
            },
        }];

        Ok(Self {
            scenarios,
            event_store,
            planner_id,
            logger,
        })
    }
}
