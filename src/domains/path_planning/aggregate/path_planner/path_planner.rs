use crate::common::aggregate::AggregateRoot;
use crate::common::{DomainError, DomainResult};
use crate::domains::path_planning::events::PathPlanningEvent;
use crate::domains::path_planning::plan::PathPlan;
use crate::domains::path_planning::types::{PathPlanRequest, PlanningAlgorithm, Position2D};
use crate::domains::path_planning::worker::{PathPlanWorker, PlanAssignment};
use crate::domains::path_planning::workspace::{Workspace, WorkspaceBounds};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathPlanner {
    pub id: String,
    pub algorithm: PlanningAlgorithm,
    pub workspace: Workspace,
    pub active_plans: Vec<PathPlan>,
    pub registered_workers: Vec<PathPlanWorker>,
    pub plan_assignments: Vec<PlanAssignment>,
    pub version: u64,
    #[serde(skip)]
    uncommitted_events: Vec<PathPlanningEvent>,
}

impl PathPlanner {
    // Public methods for AggregateRoot trait
    pub fn uncommitted_events(&self) -> &[PathPlanningEvent] {
        &self.uncommitted_events
    }
    pub fn mark_events_as_committed(&mut self) {
        self.uncommitted_events.clear();
    }
    pub fn add_event(&mut self, event: PathPlanningEvent) {
        self.uncommitted_events.push(event);
    }
}

impl PathPlanner {
    pub fn new(id: String, algorithm: PlanningAlgorithm) -> Self {
        let mut planner = Self {
            id: id.clone(),
            algorithm: algorithm.clone(),
            workspace: Workspace {
                bounds: WorkspaceBounds {
                    min_x: -100.0,
                    max_x: 100.0,
                    min_y: -100.0,
                    max_y: 100.0,
                },
                obstacles: Vec::new(),
            },
            active_plans: Vec::new(),
            registered_workers: Vec::new(),
            plan_assignments: Vec::new(),
            version: 0,
            uncommitted_events: Vec::new(),
        };

        let event = PathPlanningEvent::PlannerCreated {
            planner_id: id,
            algorithm,
            timestamp: chrono::Utc::now(),
        };

        planner.add_event(event);
        planner
    }

    pub fn request_path_plan(&mut self, path_plan_request: PathPlanRequest) -> DomainResult<()> {
        if !self.is_position_in_workspace(&path_plan_request.start_position) {
            return Err(DomainError::InvalidCommand {
                reason: "Start position is outside workspace bounds".to_string(),
            });
        }
        if !self.is_position_in_workspace(&path_plan_request.destination_position) {
            return Err(DomainError::InvalidCommand {
                reason: "Destination position is outside workspace bounds".to_string(),
            });
        }
        let plan_id = Uuid::new_v4().to_string();
        let event = PathPlanningEvent::PathPlanRequested {
            planner_id: self.id.clone(),
            request_id: path_plan_request.request_id,
            plan_id: plan_id.clone(),
            agent_id: path_plan_request.agent_id,
            start_position: path_plan_request.start_position,
            destination_position: path_plan_request.destination_position,
            start_orientation: path_plan_request.start_orientation,
            destination_orientation: path_plan_request.destination_orientation,
            timestamp: Utc::now(),
        };
        self.add_event(event.clone());
        self.apply(&event)?;
        self.try_assign_plan(&plan_id)?;
        Ok(())
    }

    pub fn register_worker(
        &mut self,
        worker_id: String,
        algorithm_capabilities: Vec<PlanningAlgorithm>,
    ) -> DomainResult<()> {
        if self
            .registered_workers
            .iter()
            .any(|w| w.worker_id == worker_id)
        {
            return Err(DomainError::InvalidCommand {
                reason: format!("Worker {} is already registered", worker_id),
            });
        }
        let event = PathPlanningEvent::WorkerRegistered {
            planner_id: self.id.clone(),
            worker_id,
            capabilities: algorithm_capabilities,
            timestamp: Utc::now(),
        };
        self.add_event(event.clone());
        self.apply(&event)?;
        Ok(())
    }

    pub fn handle_worker_ready(&mut self, worker_id: String) -> DomainResult<()> {
        if !self
            .registered_workers
            .iter()
            .any(|w| w.worker_id == worker_id)
        {
            return Err(DomainError::InvalidCommand {
                reason: format!("Worker {} is not registered", worker_id),
            });
        }
        let event = PathPlanningEvent::WorkerReady {
            planner_id: self.id.clone(),
            worker_id: worker_id.clone(),
            timestamp: Utc::now(),
        };
        self.add_event(event.clone());
        self.apply(&event)?;
        self.try_assign_work_to_worker(&worker_id)?;
        Ok(())
    }

    pub fn handle_plan_assignment_accepted(
        &mut self,
        worker_id: String,
        plan_id: String,
    ) -> DomainResult<()> {
        let event = PathPlanningEvent::PlanAssignmentAccepted {
            planner_id: self.id.clone(),
            plan_id,
            worker_id,
            timestamp: Utc::now(),
        };
        self.add_event(event.clone());
        self.apply(&event)?;
        Ok(())
    }

    pub fn handle_plan_completed(
        &mut self,
        worker_id: String,
        plan_id: String,
        waypoints: Vec<Position2D>,
    ) -> DomainResult<()> {
        let event = PathPlanningEvent::PlanCompleted {
            planner_id: self.id.clone(),
            plan_id,
            worker_id: Some(worker_id.clone()),
            waypoints,
            timestamp: Utc::now(),
        };
        self.add_event(event.clone());
        self.apply(&event)?;
        let ready_event = PathPlanningEvent::WorkerReady {
            planner_id: self.id.clone(),
            worker_id: worker_id.clone(),
            timestamp: Utc::now(),
        };
        self.add_event(ready_event.clone());
        self.apply(&ready_event)?;
        self.try_assign_work_to_worker(&worker_id)?;
        Ok(())
    }

    pub fn handle_plan_failed(
        &mut self,
        worker_id: String,
        plan_id: String,
        reason: String,
    ) -> DomainResult<()> {
        let event = PathPlanningEvent::PlanFailed {
            planner_id: self.id.clone(),
            plan_id,
            worker_id: Some(worker_id.clone()),
            reason,
            timestamp: Utc::now(),
        };
        self.add_event(event.clone());
        self.apply(&event)?;
        let ready_event = PathPlanningEvent::WorkerReady {
            planner_id: self.id.clone(),
            worker_id: worker_id.clone(),
            timestamp: Utc::now(),
        };
        self.add_event(ready_event.clone());
        self.apply(&ready_event)?;
        self.try_assign_work_to_worker(&worker_id)?;
        Ok(())
    }

    pub fn is_position_in_workspace(&self, position: &Position2D) -> bool {
        let bounds = &self.workspace.bounds;
        position.x >= bounds.min_x
            && position.x <= bounds.max_x
            && position.y >= bounds.min_y
            && position.y <= bounds.max_y
    }
}
