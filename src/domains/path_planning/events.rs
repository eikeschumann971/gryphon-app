use crate::common::DomainEvent;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use super::aggregate::PlanningAlgorithm;
use crate::domains::kinematic_agent::Position3D;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PathPlanningEvent {
    PlannerCreated {
        planner_id: String,
        algorithm: PlanningAlgorithm,
        timestamp: DateTime<Utc>,
    },
    PlanRequested {
        planner_id: String,
        plan_id: String,
        agent_id: String,
        start: Position3D,
        goal: Position3D,
        timestamp: DateTime<Utc>,
    },
    PlanCompleted {
        planner_id: String,
        plan_id: String,
        waypoints: Vec<Position3D>,
        timestamp: DateTime<Utc>,
    },
    PlanFailed {
        planner_id: String,
        plan_id: String,
        reason: String,
        timestamp: DateTime<Utc>,
    },
}

impl DomainEvent for PathPlanningEvent {
    fn event_type(&self) -> &'static str {
        match self {
            PathPlanningEvent::PlannerCreated { .. } => "PlannerCreated",
            PathPlanningEvent::PlanRequested { .. } => "PlanRequested",
            PathPlanningEvent::PlanCompleted { .. } => "PlanCompleted",
            PathPlanningEvent::PlanFailed { .. } => "PlanFailed",
        }
    }

    fn aggregate_id(&self) -> &str {
        match self {
            PathPlanningEvent::PlannerCreated { planner_id, .. } => planner_id,
            PathPlanningEvent::PlanRequested { planner_id, .. } => planner_id,
            PathPlanningEvent::PlanCompleted { planner_id, .. } => planner_id,
            PathPlanningEvent::PlanFailed { planner_id, .. } => planner_id,
        }
    }

    fn event_version(&self) -> u64 { 1 }

    fn occurred_at(&self) -> DateTime<Utc> {
        match self {
            PathPlanningEvent::PlannerCreated { timestamp, .. } => *timestamp,
            PathPlanningEvent::PlanRequested { timestamp, .. } => *timestamp,
            PathPlanningEvent::PlanCompleted { timestamp, .. } => *timestamp,
            PathPlanningEvent::PlanFailed { timestamp, .. } => *timestamp,
        }
    }
}
