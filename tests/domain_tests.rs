use gryphon_app::domains::logical_agent::*;
use gryphon_app::domains::path_planning::*;
use gryphon_app::common::AggregateRoot;
use chrono::Utc;

#[tokio::test]
async fn test_logical_agent_creation() {
    let agent = LogicalAgent::new("agent-1".to_string(), "Test Agent".to_string());
    
    assert_eq!(agent.id, "agent-1");
    assert_eq!(agent.name, "Test Agent");
    assert_eq!(agent.uncommitted_events().len(), 1);
    
    match &agent.uncommitted_events()[0] {
        LogicalAgentEvent::AgentCreated { agent_id, name, .. } => {
            assert_eq!(agent_id, "agent-1");
            assert_eq!(name, "Test Agent");
        }
        _ => panic!("Expected AgentCreated event"),
    }
}

#[tokio::test]
async fn test_add_objective() {
    let mut agent = LogicalAgent::new("agent-1".to_string(), "Test Agent".to_string());
    
    let objective_id = agent.add_objective(
        "Navigate to target".to_string(),
        5,
        vec!["avoid_obstacles".to_string()]
    ).unwrap();
    
    assert_eq!(agent.objectives.len(), 1);
    assert_eq!(agent.objectives[0].id, objective_id);
    assert_eq!(agent.objectives[0].description, "Navigate to target");
    assert_eq!(agent.objectives[0].priority, 5);
}

#[tokio::test]
async fn test_path_planner_route_request() {
    let mut planner = PathPlanner::new("planner-1".to_string(), PlanningAlgorithm::AStar);
    
    let route_request = RouteRequest {
        request_id: "req-123".to_string(),
        agent_id: "agent-1".to_string(),
        start_position: Position2D { x: 10.0, y: 20.0 },
        destination_position: Position2D { x: 50.0, y: 80.0 },
        start_orientation: Orientation2D { angle: 0.0 },
        destination_orientation: Orientation2D { angle: 1.57 }, // 90 degrees in radians
        created_at: Utc::now(),
    };
    
    let result = planner.handle_route_request(route_request);
    assert!(result.is_ok());
    
    // Check that the RouteRequested event was emitted
    assert_eq!(planner.uncommitted_events().len(), 2); // PlannerCreated + RouteRequested
    
    match &planner.uncommitted_events()[1] {
        PathPlanningEvent::RouteRequested { 
            request_id, 
            agent_id, 
            start_position, 
            destination_position,
            .. 
        } => {
            assert_eq!(request_id, "req-123");
            assert_eq!(agent_id, "agent-1");
            assert_eq!(start_position.x, 10.0);
            assert_eq!(start_position.y, 20.0);
            assert_eq!(destination_position.x, 50.0);
            assert_eq!(destination_position.y, 80.0);
        }
        _ => panic!("Expected RouteRequested event"),
    }
    
    // Check that a plan was added to active plans after applying the event
    assert_eq!(planner.active_plans.len(), 1);
    assert_eq!(planner.active_plans[0].agent_id, "agent-1");
    assert_eq!(planner.active_plans[0].start.x, 10.0);
    assert_eq!(planner.active_plans[0].start.y, 20.0);
    assert_eq!(planner.active_plans[0].goal.x, 50.0);
    assert_eq!(planner.active_plans[0].goal.y, 80.0);
    
    // Check orientations are properly set
    assert_eq!(planner.active_plans[0].start_orientation.angle, 0.0);
    assert_eq!(planner.active_plans[0].destination_orientation.angle, 1.57);
    
    // Check that created_at timestamp is set
    assert!(planner.active_plans[0].created_at <= chrono::Utc::now());
}

#[tokio::test]
async fn test_path_planner_route_request_invalid_position() {
    let mut planner = PathPlanner::new("planner-1".to_string(), PlanningAlgorithm::AStar);
    
    // Create a route request with start position outside workspace bounds
    let route_request = RouteRequest {
        request_id: "req-123".to_string(),
        agent_id: "agent-1".to_string(),
        start_position: Position2D { x: -200.0, y: 20.0 }, // Outside bounds
        destination_position: Position2D { x: 50.0, y: 80.0 },
        start_orientation: Orientation2D { angle: 0.0 },
        destination_orientation: Orientation2D { angle: 1.57 },
        created_at: Utc::now(),
    };
    
    let result = planner.handle_route_request(route_request);
    assert!(result.is_err());
    
    if let Err(error) = result {
        match error {
            gryphon_app::common::DomainError::InvalidCommand { reason } => {
                assert!(reason.contains("Start position is outside workspace bounds"));
            }
            _ => panic!("Expected InvalidCommand error"),
        }
    }
}
