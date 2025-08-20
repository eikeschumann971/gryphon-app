use esrs::aggregate::Aggregate;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// Reuse domain types where appropriate
use crate::domains::path_planning::aggregate::types::{Orientation2D, Position2D, PlanningAlgorithm};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PathPlannerState {
    pub id: String,
    pub algorithm: PlanningAlgorithm,
    // ... keep minimal for scaffold
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PathPlannerCommand {
    CreatePlanner { planner_id: String, algorithm: PlanningAlgorithm },
    RequestPlan { /* fields omitted for scaffold */ },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PathPlannerEvent {
    PlannerCreated { planner_id: String, algorithm: PlanningAlgorithm, timestamp: DateTime<Utc> },
    // other events will be added in follow-up commits
}

#[derive(Debug, thiserror::Error)]
pub enum PathPlannerError {
    #[error("invalid command")] InvalidCommand,
}

pub struct PathPlanner;

impl Aggregate for PathPlanner {
    const NAME: &'static str = "path_planner";

    type State = PathPlannerState;
    type Command = PathPlannerCommand;
    type Event = PathPlannerEvent;
    type Error = PathPlannerError;

    fn handle_command(_state: &Self::State, _command: Self::Command) -> Result<Vec<Self::Event>, Self::Error> {
        // Minimal scaffold: handle CreatePlanner only
        match _command {
            PathPlannerCommand::CreatePlanner { planner_id, algorithm } => Ok(vec![PathPlannerEvent::PlannerCreated { planner_id, algorithm, timestamp: chrono::Utc::now() }]),
            _ => Err(PathPlannerError::InvalidCommand),
        }
    }

    fn apply_event(mut _state: Self::State, event: Self::Event) -> Self::State {
        match event {
            PathPlannerEvent::PlannerCreated { planner_id, algorithm, .. } => {
                _state.id = planner_id;
                _state.algorithm = algorithm;
                _state
            }
        }
    }
}
