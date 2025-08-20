 
use super::path_planning::{PathPlanner, PathPlannerState, PathPlannerCommand};
use crate::domains::path_planning::aggregate::types::{Position2D, Orientation2D, PlanningAlgorithm};

#[tokio::test]
async fn test_create_planner_happy_path() {
    let state = PathPlannerState::default();
    let cmd = PathPlannerCommand::CreatePlanner { planner_id: "test-planner".to_string(), algorithm: PlanningAlgorithm::AStar };
    let events = PathPlanner::handle_command(&state, cmd).expect("should create event");
    assert_eq!(events.len(), 1);
}

#[tokio::test]
async fn test_request_plan_out_of_bounds() {
    // create a state with tight bounds
    let mut state = PathPlannerState::default();
    state.workspace.bounds.min_x = 0.0;
    state.workspace.bounds.max_x = 1.0;
    state.workspace.bounds.min_y = 0.0;
    state.workspace.bounds.max_y = 1.0;

    let cmd = PathPlannerCommand::RequestPathPlan {
        request_id: "r1".to_string(),
        agent_id: "a1".to_string(),
        start_position: Position2D { x: -1.0, y: 0.5 },
        destination_position: Position2D { x: 0.5, y: 0.5 },
        start_orientation: Orientation2D { angle: 0.0 },
        destination_orientation: Orientation2D { angle: 0.0 },
    };

    let res = PathPlanner::handle_command(&state, cmd);
    assert!(res.is_err());
}
