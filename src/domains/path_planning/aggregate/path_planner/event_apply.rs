use crate::domains::path_planning::events::PathPlanningEvent;
use super::super::plan::{PathPlan, PlanStatus};
use super::super::worker::{PathPlanWorker, WorkerStatus, PlanAssignment};
use super::PathPlanner;
use crate::common::{AggregateRoot, DomainResult};

impl AggregateRoot for PathPlanner {
	type Event = PathPlanningEvent;

	fn aggregate_id(&self) -> &str { &self.id }
	fn version(&self) -> u64 { self.version }
	fn apply(&mut self, event: &Self::Event) -> DomainResult<()> {
		match event {
			PathPlanningEvent::PlannerCreated { .. } => {}
			PathPlanningEvent::PathPlanRequested { plan_id, agent_id, start_position, destination_position, start_orientation, destination_orientation, timestamp, .. } => {
				let path_plan = PathPlan {
					id: plan_id.clone(),
					agent_id: agent_id.clone(),
					start: start_position.clone(),
					goal: destination_position.clone(),
					start_orientation: start_orientation.clone(),
					destination_orientation: destination_orientation.clone(),
					waypoints: Vec::new(),
					status: PlanStatus::Planning,
					created_at: *timestamp,
				};
				self.active_plans.push(path_plan);
			}
			PathPlanningEvent::WorkerRegistered { worker_id, capabilities, timestamp, .. } => {
				let worker = PathPlanWorker {
					worker_id: worker_id.clone(),
					status: WorkerStatus::Idle,
					algorithm_capabilities: capabilities.clone(),
					last_heartbeat: *timestamp,
					current_plan_id: None,
				};
				self.registered_workers.push(worker);
			}
			PathPlanningEvent::WorkerReady { worker_id, timestamp, .. } => {
				if let Some(worker) = self.registered_workers.iter_mut().find(|w| w.worker_id == *worker_id) {
					worker.status = WorkerStatus::Idle;
					worker.last_heartbeat = *timestamp;
					worker.current_plan_id = None;
				}
			}
			PathPlanningEvent::WorkerBusy { worker_id, plan_id, timestamp, .. } => {
				if let Some(worker) = self.registered_workers.iter_mut().find(|w| w.worker_id == *worker_id) {
					worker.status = WorkerStatus::Busy;
					worker.last_heartbeat = *timestamp;
					worker.current_plan_id = Some(plan_id.clone());
				}
			}
			PathPlanningEvent::WorkerProcessing { worker_id, plan_id, timestamp, .. } => {
				// Similar to WorkerBusy - indicates worker is processing a plan
				if let Some(worker) = self.registered_workers.iter_mut().find(|w| w.worker_id == *worker_id) {
					worker.status = WorkerStatus::Busy;
					worker.last_heartbeat = *timestamp;
					worker.current_plan_id = Some(plan_id.clone());
				}
			}
			PathPlanningEvent::WorkerOffline { worker_id, .. } => {
				if let Some(worker) = self.registered_workers.iter_mut().find(|w| w.worker_id == *worker_id) {
					worker.status = WorkerStatus::Offline;
					worker.current_plan_id = None;
				}
				self.plan_assignments.retain(|a| a.worker_id != *worker_id);
			}
			PathPlanningEvent::PlanAssigned { plan_id, worker_id, timeout_seconds, timestamp, .. } => {
				let assignment = PlanAssignment {
					plan_id: plan_id.clone(),
					worker_id: worker_id.clone(),
					assigned_at: *timestamp,
					timeout_at: *timestamp + chrono::Duration::seconds(*timeout_seconds as i64),
				};
				self.plan_assignments.push(assignment);
				if let Some(plan) = self.active_plans.iter_mut().find(|p| p.id == *plan_id) {
					plan.status = PlanStatus::Assigned;
				}
				if let Some(worker) = self.registered_workers.iter_mut().find(|w| w.worker_id == *worker_id) {
					worker.current_plan_id = Some(plan_id.clone());
				}
			}
			PathPlanningEvent::PlanAssignmentAccepted { plan_id, worker_id, .. } => {
				if let Some(plan) = self.active_plans.iter_mut().find(|p| p.id == *plan_id) {
					plan.status = PlanStatus::InProgress;
				}
				if let Some(worker) = self.registered_workers.iter_mut().find(|w| w.worker_id == *worker_id) {
					worker.status = WorkerStatus::Busy;
				}
			}
			PathPlanningEvent::PlanAssignmentRejected { plan_id, worker_id, .. } => {
				self.plan_assignments.retain(|a| !(a.plan_id == *plan_id && a.worker_id == *worker_id));
				if let Some(plan) = self.active_plans.iter_mut().find(|p| p.id == *plan_id) {
					plan.status = PlanStatus::Planning;
				}
				if let Some(worker) = self.registered_workers.iter_mut().find(|w| w.worker_id == *worker_id) {
					worker.current_plan_id = None;
					worker.status = WorkerStatus::Idle;
				}
			}
			PathPlanningEvent::PlanAssignmentTimedOut { plan_id, worker_id, .. } => {
				self.plan_assignments.retain(|a| !(a.plan_id == *plan_id && a.worker_id == *worker_id));
				if let Some(plan) = self.active_plans.iter_mut().find(|p| p.id == *plan_id) {
					plan.status = PlanStatus::Planning;
				}
				if let Some(worker) = self.registered_workers.iter_mut().find(|w| w.worker_id == *worker_id) {
					worker.status = WorkerStatus::Offline;
					worker.current_plan_id = None;
				}
			}
			PathPlanningEvent::PlanRequested { .. } => {}
			PathPlanningEvent::PlanCompleted { plan_id, waypoints, worker_id, .. } => {
				if let Some(plan) = self.active_plans.iter_mut().find(|p| p.id == *plan_id) {
					plan.status = PlanStatus::Complete;
					plan.waypoints = waypoints.clone();
				}
				self.plan_assignments.retain(|a| a.plan_id != *plan_id);
				if let Some(wid) = worker_id {
					if let Some(worker) = self.registered_workers.iter_mut().find(|w| w.worker_id == *wid) {
						worker.status = WorkerStatus::Idle;
						worker.current_plan_id = None;
					}
				}
			}
			PathPlanningEvent::PlanFailed { plan_id, worker_id, .. } => {
				if let Some(plan) = self.active_plans.iter_mut().find(|p| p.id == *plan_id) {
					plan.status = PlanStatus::Failed("Planning failed".to_string());
				}
				self.plan_assignments.retain(|a| a.plan_id != *plan_id);
				if let Some(wid) = worker_id {
					if let Some(worker) = self.registered_workers.iter_mut().find(|w| w.worker_id == *wid) {
						worker.status = WorkerStatus::Idle;
						worker.current_plan_id = None;
					}
				}
			}
		}
		self.version += 1;
		Ok(())
	}
	fn uncommitted_events(&self) -> &[Self::Event] {
		PathPlanner::uncommitted_events(self)
	}
	fn mark_events_as_committed(&mut self) {
		PathPlanner::mark_events_as_committed(self)
	}
	fn add_event(&mut self, event: Self::Event) {
		PathPlanner::add_event(self, event.clone())
	}
}
