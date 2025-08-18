use crate::common::DomainEvent;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use super::aggregate::{PlanningAlgorithm, Position2D, Orientation2D};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PathPlanningEvent {
    PlannerCreated {
        planner_id: String,
        algorithm: PlanningAlgorithm,
        timestamp: DateTime<Utc>,
    },
    RouteRequested {
        planner_id: String,
        request_id: String,
        plan_id: String,
        agent_id: String,
        start_position: Position2D,
        destination_position: Position2D,
        start_orientation: Orientation2D,
        destination_orientation: Orientation2D,
        timestamp: DateTime<Utc>,
    },
    
    // Worker lifecycle events
    WorkerRegistered {
        planner_id: String,
        worker_id: String,
        algorithm_capabilities: Vec<PlanningAlgorithm>,
        timestamp: DateTime<Utc>,
    },
    WorkerReady {
        planner_id: String,
        worker_id: String,
        timestamp: DateTime<Utc>,
    },
    WorkerBusy {
        planner_id: String,
        worker_id: String,
        plan_id: String,
        timestamp: DateTime<Utc>,
    },
    WorkerOffline {
        planner_id: String,
        worker_id: String,
        reason: String,
        timestamp: DateTime<Utc>,
    },
    
    // Work assignment events
    PlanAssigned {
        planner_id: String,
        plan_id: String,
        worker_id: String,
        timeout_seconds: u64,
        timestamp: DateTime<Utc>,
    },
    PlanAssignmentAccepted {
        planner_id: String,
        plan_id: String,
        worker_id: String,
        timestamp: DateTime<Utc>,
    },
    PlanAssignmentRejected {
        planner_id: String,
        plan_id: String,
        worker_id: String,
        reason: String,
        timestamp: DateTime<Utc>,
    },
    PlanAssignmentTimedOut {
        planner_id: String,
        plan_id: String,
        worker_id: String,
        timestamp: DateTime<Utc>,
    },
    
    // Legacy events
    PlanRequested {
        planner_id: String,
        plan_id: String,
        agent_id: String,
        start: Position2D,
        goal: Position2D,
        timestamp: DateTime<Utc>,
    },
    PlanCompleted {
        planner_id: String,
        plan_id: String,
        worker_id: Option<String>, // Add worker_id to track who completed it
        waypoints: Vec<Position2D>,
        timestamp: DateTime<Utc>,
    },
    PlanFailed {
        planner_id: String,
        plan_id: String,
        worker_id: Option<String>, // Add worker_id to track who failed it
        reason: String,
        timestamp: DateTime<Utc>,
    },
}

impl DomainEvent for PathPlanningEvent {
    fn event_type(&self) -> &'static str {
        match self {
            PathPlanningEvent::PlannerCreated { .. } => "PlannerCreated",
            PathPlanningEvent::RouteRequested { .. } => "RouteRequested",
            PathPlanningEvent::WorkerRegistered { .. } => "WorkerRegistered",
            PathPlanningEvent::WorkerReady { .. } => "WorkerReady",
            PathPlanningEvent::WorkerBusy { .. } => "WorkerBusy",
            PathPlanningEvent::WorkerOffline { .. } => "WorkerOffline",
            PathPlanningEvent::PlanAssigned { .. } => "PlanAssigned",
            PathPlanningEvent::PlanAssignmentAccepted { .. } => "PlanAssignmentAccepted",
            PathPlanningEvent::PlanAssignmentRejected { .. } => "PlanAssignmentRejected",
            PathPlanningEvent::PlanAssignmentTimedOut { .. } => "PlanAssignmentTimedOut",
            PathPlanningEvent::PlanRequested { .. } => "PlanRequested",
            PathPlanningEvent::PlanCompleted { .. } => "PlanCompleted",
            PathPlanningEvent::PlanFailed { .. } => "PlanFailed",
        }
    }

    fn aggregate_id(&self) -> &str {
        match self {
            PathPlanningEvent::PlannerCreated { planner_id, .. } => planner_id,
            PathPlanningEvent::RouteRequested { planner_id, .. } => planner_id,
            PathPlanningEvent::WorkerRegistered { planner_id, .. } => planner_id,
            PathPlanningEvent::WorkerReady { planner_id, .. } => planner_id,
            PathPlanningEvent::WorkerBusy { planner_id, .. } => planner_id,
            PathPlanningEvent::WorkerOffline { planner_id, .. } => planner_id,
            PathPlanningEvent::PlanAssigned { planner_id, .. } => planner_id,
            PathPlanningEvent::PlanAssignmentAccepted { planner_id, .. } => planner_id,
            PathPlanningEvent::PlanAssignmentRejected { planner_id, .. } => planner_id,
            PathPlanningEvent::PlanAssignmentTimedOut { planner_id, .. } => planner_id,
            PathPlanningEvent::PlanRequested { planner_id, .. } => planner_id,
            PathPlanningEvent::PlanCompleted { planner_id, .. } => planner_id,
            PathPlanningEvent::PlanFailed { planner_id, .. } => planner_id,
        }
    }

    fn event_version(&self) -> u64 { 1 }

    fn occurred_at(&self) -> DateTime<Utc> {
        match self {
            PathPlanningEvent::PlannerCreated { timestamp, .. } => *timestamp,
            PathPlanningEvent::RouteRequested { timestamp, .. } => *timestamp,
            PathPlanningEvent::WorkerRegistered { timestamp, .. } => *timestamp,
            PathPlanningEvent::WorkerReady { timestamp, .. } => *timestamp,
            PathPlanningEvent::WorkerBusy { timestamp, .. } => *timestamp,
            PathPlanningEvent::WorkerOffline { timestamp, .. } => *timestamp,
            PathPlanningEvent::PlanAssigned { timestamp, .. } => *timestamp,
            PathPlanningEvent::PlanAssignmentAccepted { timestamp, .. } => *timestamp,
            PathPlanningEvent::PlanAssignmentRejected { timestamp, .. } => *timestamp,
            PathPlanningEvent::PlanAssignmentTimedOut { timestamp, .. } => *timestamp,
            PathPlanningEvent::PlanRequested { timestamp, .. } => *timestamp,
            PathPlanningEvent::PlanCompleted { timestamp, .. } => *timestamp,
            PathPlanningEvent::PlanFailed { timestamp, .. } => *timestamp,
        }
    }
}
