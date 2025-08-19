use chrono::Utc;
use gryphon_app::common::{AggregateRoot, DomainError, DomainEvent};
use gryphon_app::domains::path_planning::*;

#[cfg(test)]
mod path_planner_tests {
    use super::*;

    #[test]
    fn test_path_planner_creation() {
        let planner = PathPlanner::new("planner-1".to_string(), PlanningAlgorithm::AStar);

        assert_eq!(planner.id, "planner-1");
        assert_eq!(planner.algorithm, PlanningAlgorithm::AStar);
        assert_eq!(planner.version, 0);
        assert_eq!(planner.active_plans.len(), 0);
        assert_eq!(planner.registered_workers.len(), 0);
        assert_eq!(planner.plan_assignments.len(), 0);

        // Should have one uncommitted event (PlannerCreated)
        assert_eq!(planner.uncommitted_events().len(), 1);

        match &planner.uncommitted_events()[0] {
            PathPlanningEvent::PlannerCreated {
                planner_id,
                algorithm,
                ..
            } => {
                assert_eq!(planner_id, "planner-1");
                assert_eq!(*algorithm, PlanningAlgorithm::AStar);
            }
            _ => panic!("Expected PlannerCreated event"),
        }
    }

    #[test]
    fn test_worker_registration_success() {
        let mut planner = PathPlanner::new("planner-1".to_string(), PlanningAlgorithm::AStar);
        let worker_id = "worker-1".to_string();
        let capabilities = vec![PlanningAlgorithm::AStar, PlanningAlgorithm::Dijkstra];

        let result = planner.register_worker(worker_id.clone(), capabilities.clone());
        assert!(result.is_ok());

        // Check worker was added
        assert_eq!(planner.registered_workers.len(), 1);
        let worker = &planner.registered_workers[0];
        assert_eq!(worker.worker_id, worker_id);
        assert_eq!(worker.status, WorkerStatus::Idle);
        assert_eq!(worker.algorithm_capabilities, capabilities);
        assert!(worker.current_plan_id.is_none());

        // Check event was emitted
        assert_eq!(planner.uncommitted_events().len(), 2);
        match &planner.uncommitted_events()[1] {
            PathPlanningEvent::WorkerRegistered {
                worker_id: event_worker_id,
                capabilities: event_capabilities,
                ..
            } => {
                assert_eq!(*event_worker_id, worker_id);
                assert_eq!(*event_capabilities, capabilities);
            }
            _ => panic!("Expected WorkerRegistered event"),
        }
    }

    #[test]
    fn test_worker_registration_duplicate_worker() {
        let mut planner = PathPlanner::new("planner-1".to_string(), PlanningAlgorithm::AStar);
        let worker_id = "worker-1".to_string();
        let capabilities = vec![PlanningAlgorithm::AStar];

        // Register worker first time
        planner
            .register_worker(worker_id.clone(), capabilities.clone())
            .unwrap();

        // Try to register same worker again
        let result = planner.register_worker(worker_id.clone(), capabilities);
        assert!(result.is_err());

        match result.unwrap_err() {
            DomainError::InvalidCommand { reason } => {
                assert!(reason.contains("Worker worker-1 is already registered"));
            }
            _ => panic!("Expected InvalidCommand error"),
        }

        // Should still have only one worker
        assert_eq!(planner.registered_workers.len(), 1);
    }

    #[test]
    fn test_path_plan_request_valid_positions() {
        let mut planner = PathPlanner::new("planner-1".to_string(), PlanningAlgorithm::AStar);

        let request = PathPlanRequest {
            request_id: "req-123".to_string(),
            agent_id: "agent-1".to_string(),
            start_position: Position2D { x: 10.0, y: 20.0 },
            destination_position: Position2D { x: 50.0, y: 80.0 },
            start_orientation: Orientation2D { angle: 0.0 },
            destination_orientation: Orientation2D { angle: 1.57 },
            created_at: Utc::now(),
        };

        let result = planner.request_path_plan(request);
        assert!(result.is_ok());

        // Check plan was added
        assert_eq!(planner.active_plans.len(), 1);
        let plan = &planner.active_plans[0];
        assert_eq!(plan.agent_id, "agent-1");
        assert_eq!(plan.status, PlanStatus::Planning);
        assert_eq!(plan.start.x, 10.0);
        assert_eq!(plan.start.y, 20.0);
        assert_eq!(plan.goal.x, 50.0);
        assert_eq!(plan.goal.y, 80.0);

        // Check event was emitted
        assert_eq!(planner.uncommitted_events().len(), 2);
        match &planner.uncommitted_events()[1] {
            PathPlanningEvent::PathPlanRequested {
                request_id,
                agent_id,
                ..
            } => {
                assert_eq!(*request_id, "req-123");
                assert_eq!(*agent_id, "agent-1");
            }
            _ => panic!("Expected PathPlanRequested event"),
        }
    }

    #[test]
    fn test_path_plan_request_invalid_start_position() {
        let mut planner = PathPlanner::new("planner-1".to_string(), PlanningAlgorithm::AStar);

        let request = PathPlanRequest {
            request_id: "req-123".to_string(),
            agent_id: "agent-1".to_string(),
            start_position: Position2D { x: -200.0, y: 20.0 }, // Outside workspace bounds
            destination_position: Position2D { x: 50.0, y: 80.0 },
            start_orientation: Orientation2D { angle: 0.0 },
            destination_orientation: Orientation2D { angle: 1.57 },
            created_at: Utc::now(),
        };

        let result = planner.request_path_plan(request);
        assert!(result.is_err());

        match result.unwrap_err() {
            DomainError::InvalidCommand { reason } => {
                assert!(reason.contains("Start position is outside workspace bounds"));
            }
            _ => panic!("Expected InvalidCommand error"),
        }

        // No plan should be added
        assert_eq!(planner.active_plans.len(), 0);
    }

    #[test]
    fn test_path_plan_request_invalid_destination_position() {
        let mut planner = PathPlanner::new("planner-1".to_string(), PlanningAlgorithm::AStar);

        let request = PathPlanRequest {
            request_id: "req-123".to_string(),
            agent_id: "agent-1".to_string(),
            start_position: Position2D { x: 10.0, y: 20.0 },
            destination_position: Position2D { x: 200.0, y: 80.0 }, // Outside workspace bounds
            start_orientation: Orientation2D { angle: 0.0 },
            destination_orientation: Orientation2D { angle: 1.57 },
            created_at: Utc::now(),
        };

        let result = planner.request_path_plan(request);
        assert!(result.is_err());

        match result.unwrap_err() {
            DomainError::InvalidCommand { reason } => {
                assert!(reason.contains("Destination position is outside workspace bounds"));
            }
            _ => panic!("Expected InvalidCommand error"),
        }
    }

    #[test]
    fn test_worker_ready_unregistered_worker() {
        let mut planner = PathPlanner::new("planner-1".to_string(), PlanningAlgorithm::AStar);

        let result = planner.handle_worker_ready("unknown-worker".to_string());
        assert!(result.is_err());

        match result.unwrap_err() {
            DomainError::InvalidCommand { reason } => {
                assert!(reason.contains("Worker unknown-worker is not registered"));
            }
            _ => panic!("Expected InvalidCommand error"),
        }
    }

    #[test]
    fn test_worker_ready_success() {
        let mut planner = PathPlanner::new("planner-1".to_string(), PlanningAlgorithm::AStar);
        let worker_id = "worker-1".to_string();

        // Register worker first
        planner
            .register_worker(worker_id.clone(), vec![PlanningAlgorithm::AStar])
            .unwrap();

        // Make worker ready
        let result = planner.handle_worker_ready(worker_id.clone());
        assert!(result.is_ok());

        // Check worker status is updated
        let worker = &planner.registered_workers[0];
        assert_eq!(worker.status, WorkerStatus::Idle);
        assert!(worker.current_plan_id.is_none());

        // Check event was emitted
        assert_eq!(planner.uncommitted_events().len(), 3); // PlannerCreated + WorkerRegistered + WorkerReady
        match &planner.uncommitted_events()[2] {
            PathPlanningEvent::WorkerReady {
                worker_id: event_worker_id,
                ..
            } => {
                assert_eq!(*event_worker_id, worker_id);
            }
            _ => panic!("Expected WorkerReady event"),
        }
    }

    #[test]
    fn test_plan_assignment_with_idle_worker() {
        let mut planner = PathPlanner::new("planner-1".to_string(), PlanningAlgorithm::AStar);
        let worker_id = "worker-1".to_string();

        // Register and ready a worker
        planner
            .register_worker(worker_id.clone(), vec![PlanningAlgorithm::AStar])
            .unwrap();
        planner.handle_worker_ready(worker_id.clone()).unwrap();

        // Create a path plan request
        let request = PathPlanRequest {
            request_id: "req-123".to_string(),
            agent_id: "agent-1".to_string(),
            start_position: Position2D { x: 10.0, y: 20.0 },
            destination_position: Position2D { x: 50.0, y: 80.0 },
            start_orientation: Orientation2D { angle: 0.0 },
            destination_orientation: Orientation2D { angle: 1.57 },
            created_at: Utc::now(),
        };

        planner.request_path_plan(request).unwrap();

        // Check that plan assignment was created
        assert_eq!(planner.plan_assignments.len(), 1);
        let assignment = &planner.plan_assignments[0];
        assert_eq!(assignment.worker_id, worker_id);

        // Check that plan status is assigned
        assert_eq!(planner.active_plans.len(), 1);
        assert_eq!(planner.active_plans[0].status, PlanStatus::Assigned);

        // Check that worker has the plan assigned
        let worker = &planner.registered_workers[0];
        assert!(worker.current_plan_id.is_some());
        assert_eq!(
            worker.current_plan_id.as_ref().unwrap(),
            &planner.active_plans[0].id
        );
    }

    #[test]
    fn test_plan_assignment_acceptance() {
        let mut planner = PathPlanner::new("planner-1".to_string(), PlanningAlgorithm::AStar);
        let worker_id = "worker-1".to_string();

        // Setup: register worker, create plan, and assign it
        planner
            .register_worker(worker_id.clone(), vec![PlanningAlgorithm::AStar])
            .unwrap();
        planner.handle_worker_ready(worker_id.clone()).unwrap();

        let request = PathPlanRequest {
            request_id: "req-123".to_string(),
            agent_id: "agent-1".to_string(),
            start_position: Position2D { x: 10.0, y: 20.0 },
            destination_position: Position2D { x: 50.0, y: 80.0 },
            start_orientation: Orientation2D { angle: 0.0 },
            destination_orientation: Orientation2D { angle: 1.57 },
            created_at: Utc::now(),
        };

        planner.request_path_plan(request).unwrap();
        let plan_id = planner.active_plans[0].id.clone();

        // Accept the assignment
        let result = planner.handle_plan_assignment_accepted(worker_id.clone(), plan_id.clone());
        assert!(result.is_ok());

        // Check plan status changed to InProgress
        assert_eq!(planner.active_plans[0].status, PlanStatus::InProgress);

        // Check worker status changed to Busy
        let worker = &planner.registered_workers[0];
        assert_eq!(worker.status, WorkerStatus::Busy);

        // Check event was emitted
        let events = planner.uncommitted_events();
        assert!(events
            .iter()
            .any(|e| matches!(e, PathPlanningEvent::PlanAssignmentAccepted { .. })));
    }

    #[test]
    fn test_plan_completion() {
        let mut planner = PathPlanner::new("planner-1".to_string(), PlanningAlgorithm::AStar);
        let worker_id = "worker-1".to_string();

        // Setup: register worker, create plan, assign it, and accept it
        planner
            .register_worker(worker_id.clone(), vec![PlanningAlgorithm::AStar])
            .unwrap();
        planner.handle_worker_ready(worker_id.clone()).unwrap();

        let request = PathPlanRequest {
            request_id: "req-123".to_string(),
            agent_id: "agent-1".to_string(),
            start_position: Position2D { x: 10.0, y: 20.0 },
            destination_position: Position2D { x: 50.0, y: 80.0 },
            start_orientation: Orientation2D { angle: 0.0 },
            destination_orientation: Orientation2D { angle: 1.57 },
            created_at: Utc::now(),
        };

        planner.request_path_plan(request).unwrap();
        let plan_id = planner.active_plans[0].id.clone();
        planner
            .handle_plan_assignment_accepted(worker_id.clone(), plan_id.clone())
            .unwrap();

        // Complete the plan
        let waypoints = vec![
            Position2D { x: 20.0, y: 30.0 },
            Position2D { x: 35.0, y: 55.0 },
            Position2D { x: 50.0, y: 80.0 },
        ];

        let result =
            planner.handle_plan_completed(worker_id.clone(), plan_id.clone(), waypoints.clone());
        assert!(result.is_ok());

        // Check plan status changed to Complete
        assert_eq!(planner.active_plans[0].status, PlanStatus::Complete);
        assert_eq!(planner.active_plans[0].waypoints, waypoints);

        // Check worker status changed back to Idle
        let worker = &planner.registered_workers[0];
        assert_eq!(worker.status, WorkerStatus::Idle);
        assert!(worker.current_plan_id.is_none());

        // Check assignment was removed
        assert_eq!(planner.plan_assignments.len(), 0);

        // Check events were emitted
        let events = planner.uncommitted_events();
        assert!(events
            .iter()
            .any(|e| matches!(e, PathPlanningEvent::PlanCompleted { .. })));
        assert!(events
            .iter()
            .any(|e| matches!(e, PathPlanningEvent::WorkerReady { .. })));
    }

    #[test]
    fn test_plan_failure() {
        let mut planner = PathPlanner::new("planner-1".to_string(), PlanningAlgorithm::AStar);
        let worker_id = "worker-1".to_string();

        // Setup similar to completion test
        planner
            .register_worker(worker_id.clone(), vec![PlanningAlgorithm::AStar])
            .unwrap();
        planner.handle_worker_ready(worker_id.clone()).unwrap();

        let request = PathPlanRequest {
            request_id: "req-123".to_string(),
            agent_id: "agent-1".to_string(),
            start_position: Position2D { x: 10.0, y: 20.0 },
            destination_position: Position2D { x: 50.0, y: 80.0 },
            start_orientation: Orientation2D { angle: 0.0 },
            destination_orientation: Orientation2D { angle: 1.57 },
            created_at: Utc::now(),
        };

        planner.request_path_plan(request).unwrap();
        let plan_id = planner.active_plans[0].id.clone();
        planner
            .handle_plan_assignment_accepted(worker_id.clone(), plan_id.clone())
            .unwrap();

        // Fail the plan
        let failure_reason = "No path found due to obstacles".to_string();
        let result =
            planner.handle_plan_failed(worker_id.clone(), plan_id.clone(), failure_reason.clone());
        assert!(result.is_ok());

        // Check plan status changed to Failed
        match &planner.active_plans[0].status {
            PlanStatus::Failed(reason) => {
                assert_eq!(*reason, "Planning failed");
            }
            _ => panic!("Expected plan status to be Failed"),
        }

        // Check worker status changed back to Idle
        let worker = &planner.registered_workers[0];
        assert_eq!(worker.status, WorkerStatus::Idle);
        assert!(worker.current_plan_id.is_none());

        // Check assignment was removed
        assert_eq!(planner.plan_assignments.len(), 0);

        // Check events were emitted
        let events = planner.uncommitted_events();
        assert!(events
            .iter()
            .any(|e| matches!(e, PathPlanningEvent::PlanFailed { .. })));
        assert!(events
            .iter()
            .any(|e| matches!(e, PathPlanningEvent::WorkerReady { .. })));
    }

    #[test]
    fn test_workspace_bounds_validation() {
        let planner = PathPlanner::new("planner-1".to_string(), PlanningAlgorithm::AStar);

        // Test positions within bounds
        assert!(planner.is_position_in_workspace(&Position2D { x: 0.0, y: 0.0 }));
        assert!(planner.is_position_in_workspace(&Position2D { x: 50.0, y: 50.0 }));
        assert!(planner.is_position_in_workspace(&Position2D {
            x: -100.0,
            y: -100.0
        }));
        assert!(planner.is_position_in_workspace(&Position2D { x: 100.0, y: 100.0 }));

        // Test positions outside bounds
        assert!(!planner.is_position_in_workspace(&Position2D { x: -101.0, y: 0.0 }));
        assert!(!planner.is_position_in_workspace(&Position2D { x: 101.0, y: 0.0 }));
        assert!(!planner.is_position_in_workspace(&Position2D { x: 0.0, y: -101.0 }));
        assert!(!planner.is_position_in_workspace(&Position2D { x: 0.0, y: 101.0 }));
    }

    #[test]
    fn test_multiple_workers_assignment_priority() {
        let mut planner = PathPlanner::new("planner-1".to_string(), PlanningAlgorithm::AStar);

        // Register multiple workers
        planner
            .register_worker("worker-1".to_string(), vec![PlanningAlgorithm::AStar])
            .unwrap();
        planner
            .register_worker("worker-2".to_string(), vec![PlanningAlgorithm::Dijkstra])
            .unwrap();
        planner
            .register_worker(
                "worker-3".to_string(),
                vec![PlanningAlgorithm::AStar, PlanningAlgorithm::Dijkstra],
            )
            .unwrap();

        // Make all workers ready
        planner.handle_worker_ready("worker-1".to_string()).unwrap();
        planner.handle_worker_ready("worker-2".to_string()).unwrap();
        planner.handle_worker_ready("worker-3".to_string()).unwrap();

        // Create a plan request
        let request = PathPlanRequest {
            request_id: "req-123".to_string(),
            agent_id: "agent-1".to_string(),
            start_position: Position2D { x: 10.0, y: 20.0 },
            destination_position: Position2D { x: 50.0, y: 80.0 },
            start_orientation: Orientation2D { angle: 0.0 },
            destination_orientation: Orientation2D { angle: 1.57 },
            created_at: Utc::now(),
        };

        planner.request_path_plan(request).unwrap();

        // Only one worker should be assigned (first idle worker found)
        assert_eq!(planner.plan_assignments.len(), 1);

        // Verify assignment is to the first worker
        let assignment = &planner.plan_assignments[0];
        assert_eq!(assignment.worker_id, "worker-1");

        // Other workers should still be idle
        let worker2 = planner
            .registered_workers
            .iter()
            .find(|w| w.worker_id == "worker-2")
            .unwrap();
        let worker3 = planner
            .registered_workers
            .iter()
            .find(|w| w.worker_id == "worker-3")
            .unwrap();
        assert!(worker2.current_plan_id.is_none());
        assert!(worker3.current_plan_id.is_none());
    }

    #[test]
    fn test_aggregate_root_trait_methods() {
        let mut planner = PathPlanner::new("planner-1".to_string(), PlanningAlgorithm::AStar);

        // Test aggregate_id
        assert_eq!(planner.aggregate_id(), "planner-1");

        // Test version (should start at 0)
        assert_eq!(planner.version(), 0);

        // Test uncommitted_events
        assert_eq!(planner.uncommitted_events().len(), 1);

        // Test mark_events_as_committed
        planner.mark_events_as_committed();
        assert_eq!(planner.uncommitted_events().len(), 0);

        // Test add_event
        let test_event = PathPlanningEvent::WorkerRegistered {
            planner_id: "planner-1".to_string(),
            worker_id: "test-worker".to_string(),
            capabilities: vec![PlanningAlgorithm::AStar],
            timestamp: Utc::now(),
        };

        planner.add_event(test_event.clone());
        assert_eq!(planner.uncommitted_events().len(), 1);

        // Test apply event
        let result = planner.apply(&test_event);
        assert!(result.is_ok());
        assert_eq!(planner.version(), 1);
    }
}

#[cfg(test)]
mod planning_algorithm_tests {
    use super::*;

    #[test]
    fn test_planning_algorithm_serialization() {
        // Test that all planning algorithms can be serialized/deserialized
        let algorithms = vec![
            PlanningAlgorithm::AStar,
            PlanningAlgorithm::Dijkstra,
            PlanningAlgorithm::RRT,
            PlanningAlgorithm::PRM,
            PlanningAlgorithm::DynamicWindow,
        ];

        for algorithm in algorithms {
            let serialized = serde_json::to_string(&algorithm).unwrap();
            let deserialized: PlanningAlgorithm = serde_json::from_str(&serialized).unwrap();
            assert_eq!(algorithm, deserialized);
        }
    }

    #[test]
    fn test_planning_algorithm_debug() {
        // Test Debug trait implementation
        let algorithm = PlanningAlgorithm::AStar;
        let debug_str = format!("{:?}", algorithm);
        assert!(!debug_str.is_empty());
    }
}

#[cfg(test)]
mod position_and_orientation_tests {
    use super::*;

    #[test]
    fn test_position2d_creation() {
        let pos = Position2D { x: 10.5, y: -20.3 };
        assert_eq!(pos.x, 10.5);
        assert_eq!(pos.y, -20.3);
    }

    #[test]
    fn test_position2d_serialization() {
        let pos = Position2D { x: 10.5, y: -20.3 };
        let serialized = serde_json::to_string(&pos).unwrap();
        let deserialized: Position2D = serde_json::from_str(&serialized).unwrap();
        assert_eq!(pos.x, deserialized.x);
        assert_eq!(pos.y, deserialized.y);
    }

    #[test]
    fn test_orientation2d_creation() {
        let orientation = Orientation2D { angle: 1.57 };
        assert_eq!(orientation.angle, 1.57);
    }

    #[test]
    fn test_orientation2d_serialization() {
        let orientation = Orientation2D {
            angle: std::f64::consts::PI,
        };
        let serialized = serde_json::to_string(&orientation).unwrap();
        let deserialized: Orientation2D = serde_json::from_str(&serialized).unwrap();
        assert_eq!(orientation.angle, deserialized.angle);
    }
}

#[cfg(test)]
mod worker_status_tests {
    use super::*;

    #[test]
    fn test_worker_status_variants() {
        let statuses = vec![
            WorkerStatus::Idle,
            WorkerStatus::Busy,
            WorkerStatus::Offline,
        ];

        for status in statuses {
            // Test that each status can be created and compared
            let status_copy = status.clone();
            assert_eq!(status, status_copy);

            // Test serialization
            let serialized = serde_json::to_string(&status).unwrap();
            let deserialized: WorkerStatus = serde_json::from_str(&serialized).unwrap();
            assert_eq!(status, deserialized);
        }
    }
}

#[cfg(test)]
mod plan_status_tests {
    use super::*;

    #[test]
    fn test_plan_status_variants() {
        let statuses = vec![
            PlanStatus::Planning,
            PlanStatus::Assigned,
            PlanStatus::InProgress,
            PlanStatus::Complete,
            PlanStatus::Failed("Test failure reason".to_string()),
        ];

        for status in statuses {
            // Test that each status can be created and compared
            let status_copy = status.clone();
            assert_eq!(status, status_copy);

            // Test serialization
            let serialized = serde_json::to_string(&status).unwrap();
            let deserialized: PlanStatus = serde_json::from_str(&serialized).unwrap();
            assert_eq!(status, deserialized);
        }
    }

    #[test]
    fn test_plan_status_failed_with_reason() {
        let status = PlanStatus::Failed("Custom error message".to_string());

        match status {
            PlanStatus::Failed(reason) => {
                assert_eq!(reason, "Custom error message");
            }
            _ => panic!("Expected Failed status"),
        }
    }
}

#[cfg(test)]
mod event_tests {
    use super::*;

    #[test]
    fn test_path_planning_events_serialization() {
        let events = vec![
            PathPlanningEvent::PlannerCreated {
                planner_id: "planner-1".to_string(),
                algorithm: PlanningAlgorithm::AStar,
                timestamp: Utc::now(),
            },
            PathPlanningEvent::PathPlanRequested {
                planner_id: "planner-1".to_string(),
                request_id: "req-123".to_string(),
                plan_id: "plan-456".to_string(),
                agent_id: "agent-1".to_string(),
                start_position: Position2D { x: 0.0, y: 0.0 },
                destination_position: Position2D { x: 10.0, y: 10.0 },
                start_orientation: Orientation2D { angle: 0.0 },
                destination_orientation: Orientation2D { angle: 1.57 },
                timestamp: Utc::now(),
            },
            PathPlanningEvent::WorkerRegistered {
                planner_id: "planner-1".to_string(),
                worker_id: "worker-1".to_string(),
                capabilities: vec![PlanningAlgorithm::AStar, PlanningAlgorithm::Dijkstra],
                timestamp: Utc::now(),
            },
            PathPlanningEvent::WorkerReady {
                planner_id: "planner-1".to_string(),
                worker_id: "worker-1".to_string(),
                timestamp: Utc::now(),
            },
            PathPlanningEvent::PlanCompleted {
                planner_id: "planner-1".to_string(),
                plan_id: "plan-456".to_string(),
                worker_id: Some("worker-1".to_string()),
                waypoints: vec![
                    Position2D { x: 5.0, y: 5.0 },
                    Position2D { x: 10.0, y: 10.0 },
                ],
                timestamp: Utc::now(),
            },
        ];

        for event in events {
            let serialized = serde_json::to_string(&event).unwrap();
            let deserialized: PathPlanningEvent = serde_json::from_str(&serialized).unwrap();

            // Compare discriminants (event types) since timestamps might differ slightly
            assert_eq!(
                std::mem::discriminant(&event),
                std::mem::discriminant(&deserialized)
            );
        }
    }

    #[test]
    fn test_domain_event_trait() {
        let event = PathPlanningEvent::PlannerCreated {
            planner_id: "planner-1".to_string(),
            algorithm: PlanningAlgorithm::AStar,
            timestamp: Utc::now(),
        };

        // Test that the event implements DomainEvent trait methods
        let event_type = event.event_type();
        assert!(!event_type.is_empty());
        assert_eq!(event_type, "PlannerCreated");
    }
}
