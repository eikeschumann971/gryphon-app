use gryphon_app::domains::logical_agent::*;

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
