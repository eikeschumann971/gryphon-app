use ::esrs::Aggregate;
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::domains::path_planning::aggregate::plan::PathPlan;
use crate::domains::path_planning::aggregate::types::PlanningAlgorithm;
use crate::domains::path_planning::aggregate::worker::{PathPlanWorker, PlanAssignment};
use crate::domains::path_planning::aggregate::workspace::Workspace;
use crate::domains::path_planning::events::PathPlanningEvent;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathPlannerState {
    pub id: String,
    pub algorithm: PlanningAlgorithm,
    pub workspace: Workspace,
    pub active_plans: Vec<PathPlan>,
    pub registered_workers: Vec<PathPlanWorker>,
    pub plan_assignments: Vec<PlanAssignment>,
    pub version: u64,
}

impl Default for PathPlannerState {
    fn default() -> Self {
        PathPlannerState {
            id: String::new(),
            algorithm: PlanningAlgorithm::AStar,
            workspace: Workspace {
                bounds: crate::domains::path_planning::aggregate::workspace::WorkspaceBounds {
                    min_x: 0.0,
                    max_x: 0.0,
                    min_y: 0.0,
                    max_y: 0.0,
                },
                obstacles: Vec::new(),
            },
            active_plans: Vec::new(),
            registered_workers: Vec::new(),
            plan_assignments: Vec::new(),
            version: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PathPlannerCommand {
    CreatePlanner {
        planner_id: String,
        algorithm: PlanningAlgorithm,
    },
    RequestPathPlan {
        request_id: String,
        agent_id: String,
        start_position: crate::domains::path_planning::aggregate::types::Position2D,
        destination_position: crate::domains::path_planning::aggregate::types::Position2D,
        start_orientation: crate::domains::path_planning::aggregate::types::Orientation2D,
        destination_orientation: crate::domains::path_planning::aggregate::types::Orientation2D,
    },
    RegisterWorker {
        worker_id: String,
        capabilities: Vec<PlanningAlgorithm>,
    },
    WorkerReady {
        worker_id: String,
    },
    PlanAssignmentAccepted {
        worker_id: String,
        plan_id: String,
    },
    PlanCompleted {
        worker_id: String,
        plan_id: String,
        waypoints: Vec<crate::domains::path_planning::aggregate::types::Position2D>,
    },
    PlanFailed {
        worker_id: String,
        plan_id: String,
        reason: String,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum PathPlannerError {
    #[error("invalid command: {0}")]
    InvalidCommand(String),
}

pub struct PathPlanner;

impl Aggregate for PathPlanner {
    const NAME: &'static str = "path_planner";

    type State = PathPlannerState;
    type Command = PathPlannerCommand;
    type Event = PathPlanningEvent;
    type Error = PathPlannerError;

    fn handle_command(
        state: &Self::State,
        command: Self::Command,
    ) -> Result<Vec<Self::Event>, Self::Error> {
        match command {
            PathPlannerCommand::CreatePlanner {
                planner_id,
                algorithm,
            } => Ok(vec![PathPlanningEvent::PlannerCreated {
                planner_id,
                algorithm,
                timestamp: Utc::now(),
            }]),
            PathPlannerCommand::RequestPathPlan {
                request_id,
                agent_id,
                start_position,
                destination_position,
                start_orientation,
                destination_orientation,
            } => {
                // validate positions within workspace
                if start_position.x < state.workspace.bounds.min_x
                    || start_position.x > state.workspace.bounds.max_x
                    || start_position.y < state.workspace.bounds.min_y
                    || start_position.y > state.workspace.bounds.max_y
                {
                    return Err(PathPlannerError::InvalidCommand(
                        "Start position outside workspace bounds".to_string(),
                    ));
                }
                if destination_position.x < state.workspace.bounds.min_x
                    || destination_position.x > state.workspace.bounds.max_x
                    || destination_position.y < state.workspace.bounds.min_y
                    || destination_position.y > state.workspace.bounds.max_y
                {
                    return Err(PathPlannerError::InvalidCommand(
                        "Destination position outside workspace bounds".to_string(),
                    ));
                }
                let plan_id = uuid::Uuid::new_v4().to_string();
                Ok(vec![PathPlanningEvent::PathPlanRequested {
                    planner_id: state.id.clone(),
                    request_id,
                    plan_id,
                    agent_id,
                    start_position,
                    destination_position,
                    start_orientation,
                    destination_orientation,
                    timestamp: Utc::now(),
                }])
            }
            PathPlannerCommand::RegisterWorker {
                worker_id,
                capabilities,
            } => {
                if state
                    .registered_workers
                    .iter()
                    .any(|w| w.worker_id == worker_id)
                {
                    return Err(PathPlannerError::InvalidCommand(format!(
                        "Worker {} is already registered",
                        worker_id
                    )));
                }
                Ok(vec![PathPlanningEvent::WorkerRegistered {
                    planner_id: state.id.clone(),
                    worker_id,
                    capabilities,
                    timestamp: Utc::now(),
                }])
            }
            PathPlannerCommand::WorkerReady { worker_id } => {
                if !state
                    .registered_workers
                    .iter()
                    .any(|w| w.worker_id == worker_id)
                {
                    return Err(PathPlannerError::InvalidCommand(format!(
                        "Worker {} is not registered",
                        worker_id
                    )));
                }
                Ok(vec![PathPlanningEvent::WorkerReady {
                    planner_id: state.id.clone(),
                    worker_id,
                    timestamp: Utc::now(),
                }])
            }
            PathPlannerCommand::PlanAssignmentAccepted { worker_id, plan_id } => {
                Ok(vec![PathPlanningEvent::PlanAssignmentAccepted {
                    planner_id: state.id.clone(),
                    plan_id,
                    worker_id,
                    timestamp: Utc::now(),
                }])
            }
            PathPlannerCommand::PlanCompleted {
                worker_id,
                plan_id,
                waypoints,
            } => Ok(vec![PathPlanningEvent::PlanCompleted {
                planner_id: state.id.clone(),
                plan_id,
                worker_id: Some(worker_id),
                waypoints,
                timestamp: Utc::now(),
            }]),
            PathPlannerCommand::PlanFailed {
                worker_id,
                plan_id,
                reason,
            } => Ok(vec![PathPlanningEvent::PlanFailed {
                planner_id: state.id.clone(),
                plan_id,
                worker_id: Some(worker_id),
                reason,
                timestamp: Utc::now(),
            }]),
        }
    }

    fn apply_event(mut state: Self::State, event: Self::Event) -> Self::State {
        match event {
            PathPlanningEvent::PlannerCreated {
                planner_id,
                algorithm,
                ..
            } => {
                state.id = planner_id;
                state.algorithm = algorithm;
            }
            PathPlanningEvent::PathPlanRequested {
                plan_id,
                agent_id,
                start_position,
                destination_position,
                start_orientation,
                destination_orientation,
                timestamp,
                ..
            } => {
                let path_plan = PathPlan {
                    id: plan_id.clone(),
                    agent_id: agent_id.clone(),
                    start: start_position.clone(),
                    goal: destination_position.clone(),
                    start_orientation: start_orientation.clone(),
                    destination_orientation: destination_orientation.clone(),
                    waypoints: Vec::new(),
                    status: crate::domains::path_planning::aggregate::plan::PlanStatus::Planning,
                    created_at: timestamp,
                };
                state.active_plans.push(path_plan);
            }
            PathPlanningEvent::WorkerRegistered {
                worker_id,
                capabilities,
                timestamp,
                ..
            } => {
                let worker = PathPlanWorker {
                    worker_id: worker_id.clone(),
                    status: crate::domains::path_planning::aggregate::worker::WorkerStatus::Idle,
                    algorithm_capabilities: capabilities.clone(),
                    last_heartbeat: timestamp,
                    current_plan_id: None,
                };
                state.registered_workers.push(worker);
            }
            PathPlanningEvent::WorkerReady {
                worker_id,
                timestamp,
                ..
            } => {
                if let Some(worker) = state
                    .registered_workers
                    .iter_mut()
                    .find(|w| w.worker_id == worker_id)
                {
                    worker.status =
                        crate::domains::path_planning::aggregate::worker::WorkerStatus::Idle;
                    worker.last_heartbeat = timestamp;
                    worker.current_plan_id = None;
                }
            }
            PathPlanningEvent::WorkerBusy {
                worker_id,
                plan_id,
                timestamp,
                ..
            } => {
                if let Some(worker) = state
                    .registered_workers
                    .iter_mut()
                    .find(|w| w.worker_id == worker_id)
                {
                    worker.status =
                        crate::domains::path_planning::aggregate::worker::WorkerStatus::Busy;
                    worker.last_heartbeat = timestamp;
                    worker.current_plan_id = Some(plan_id.clone());
                }
            }
            PathPlanningEvent::WorkerProcessing {
                worker_id,
                plan_id,
                timestamp,
                ..
            } => {
                if let Some(worker) = state
                    .registered_workers
                    .iter_mut()
                    .find(|w| w.worker_id == worker_id)
                {
                    worker.status =
                        crate::domains::path_planning::aggregate::worker::WorkerStatus::Busy;
                    worker.last_heartbeat = timestamp;
                    worker.current_plan_id = Some(plan_id.clone());
                }
            }
            PathPlanningEvent::WorkerOffline { worker_id, .. } => {
                if let Some(worker) = state
                    .registered_workers
                    .iter_mut()
                    .find(|w| w.worker_id == worker_id)
                {
                    worker.status =
                        crate::domains::path_planning::aggregate::worker::WorkerStatus::Offline;
                    worker.current_plan_id = None;
                }
                state.plan_assignments.retain(|a| a.worker_id != worker_id);
            }
            PathPlanningEvent::WorkerHeartbeat {
                worker_id,
                timestamp,
                ..
            } => {
                if let Some(worker) = state
                    .registered_workers
                    .iter_mut()
                    .find(|w| w.worker_id == worker_id)
                {
                    worker.last_heartbeat = timestamp;
                }
            }
            PathPlanningEvent::PlanAssigned {
                plan_id,
                worker_id,
                timeout_seconds,
                timestamp,
                ..
            } => {
                let assignment = PlanAssignment {
                    plan_id: plan_id.clone(),
                    worker_id: worker_id.clone(),
                    assigned_at: timestamp,
                    timeout_at: timestamp + chrono::Duration::seconds(timeout_seconds as i64),
                };
                state.plan_assignments.push(assignment);
                if let Some(plan) = state.active_plans.iter_mut().find(|p| p.id == plan_id) {
                    plan.status =
                        crate::domains::path_planning::aggregate::plan::PlanStatus::Assigned;
                }
                if let Some(worker) = state
                    .registered_workers
                    .iter_mut()
                    .find(|w| w.worker_id == worker_id)
                {
                    worker.current_plan_id = Some(plan_id.clone());
                }
            }
            PathPlanningEvent::PlanAssignmentAccepted {
                plan_id, worker_id, ..
            } => {
                if let Some(plan) = state.active_plans.iter_mut().find(|p| p.id == plan_id) {
                    plan.status =
                        crate::domains::path_planning::aggregate::plan::PlanStatus::InProgress;
                }
                if let Some(worker) = state
                    .registered_workers
                    .iter_mut()
                    .find(|w| w.worker_id == worker_id)
                {
                    worker.status =
                        crate::domains::path_planning::aggregate::worker::WorkerStatus::Busy;
                }
            }
            PathPlanningEvent::PlanAssignmentRejected {
                plan_id, worker_id, ..
            } => {
                state
                    .plan_assignments
                    .retain(|a| !(a.plan_id == plan_id && a.worker_id == worker_id));
                if let Some(plan) = state.active_plans.iter_mut().find(|p| p.id == plan_id) {
                    plan.status =
                        crate::domains::path_planning::aggregate::plan::PlanStatus::Planning;
                }
                if let Some(worker) = state
                    .registered_workers
                    .iter_mut()
                    .find(|w| w.worker_id == worker_id)
                {
                    worker.current_plan_id = None;
                    worker.status =
                        crate::domains::path_planning::aggregate::worker::WorkerStatus::Idle;
                }
            }
            PathPlanningEvent::PlanAssignmentTimedOut {
                plan_id, worker_id, ..
            } => {
                state
                    .plan_assignments
                    .retain(|a| !(a.plan_id == plan_id && a.worker_id == worker_id));
                if let Some(plan) = state.active_plans.iter_mut().find(|p| p.id == plan_id) {
                    plan.status =
                        crate::domains::path_planning::aggregate::plan::PlanStatus::Planning;
                }
                if let Some(worker) = state
                    .registered_workers
                    .iter_mut()
                    .find(|w| w.worker_id == worker_id)
                {
                    worker.status =
                        crate::domains::path_planning::aggregate::worker::WorkerStatus::Offline;
                    worker.current_plan_id = None;
                }
            }
            PathPlanningEvent::PlanRequested { .. } => {}
            PathPlanningEvent::PlanCompleted {
                plan_id,
                waypoints,
                worker_id,
                ..
            } => {
                if let Some(plan) = state.active_plans.iter_mut().find(|p| p.id == plan_id) {
                    plan.status =
                        crate::domains::path_planning::aggregate::plan::PlanStatus::Complete;
                    plan.waypoints = waypoints.clone();
                }
                state.plan_assignments.retain(|a| a.plan_id != plan_id);
                if let Some(wid) = worker_id {
                    if let Some(worker) = state
                        .registered_workers
                        .iter_mut()
                        .find(|w| w.worker_id == wid)
                    {
                        worker.status =
                            crate::domains::path_planning::aggregate::worker::WorkerStatus::Idle;
                        worker.current_plan_id = None;
                    }
                }
            }
            PathPlanningEvent::PlanFailed {
                plan_id, worker_id, ..
            } => {
                if let Some(plan) = state.active_plans.iter_mut().find(|p| p.id == plan_id) {
                    plan.status =
                        crate::domains::path_planning::aggregate::plan::PlanStatus::Failed(
                            "Planning failed".to_string(),
                        );
                }
                state.plan_assignments.retain(|a| a.plan_id != plan_id);
                if let Some(wid) = worker_id {
                    if let Some(worker) = state
                        .registered_workers
                        .iter_mut()
                        .find(|w| w.worker_id == wid)
                    {
                        worker.status =
                            crate::domains::path_planning::aggregate::worker::WorkerStatus::Idle;
                        worker.current_plan_id = None;
                    }
                }
            }
        }
        state.version += 1;
        state
    }
}
