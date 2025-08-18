use crate::domains::path_planning::events::PathPlanningEvent;
use crate::common::aggregate::AggregateRoot;
use super::super::plan::PlanStatus;
use super::super::worker::{WorkerStatus};
use super::{PathPlanner};
use crate::common::DomainResult;
use chrono::Utc;

impl PathPlanner {
	pub(crate) fn try_assign_plan(&mut self, plan_id: &str) -> DomainResult<()> {
		if let Some(worker) = self.registered_workers.iter().find(|w| w.status == WorkerStatus::Idle && w.current_plan_id.is_none()) {
			let worker_id = worker.worker_id.clone();
			let timeout_seconds = 300;
			let event = PathPlanningEvent::PlanAssigned {
				planner_id: self.id.clone(),
				plan_id: plan_id.to_string(),
				worker_id,
				timeout_seconds,
				timestamp: Utc::now(),
			};
			self.add_event(event.clone());
			self.apply(&event)?;
		}
		Ok(())
	}

	pub(crate) fn try_assign_work_to_worker(&mut self, worker_id: &str) -> DomainResult<()> {
		if let Some(plan) = self.active_plans.iter().find(|p| p.status == PlanStatus::Planning) {
			let plan_id = plan.id.clone();
			let timeout_seconds = 300;
			let event = PathPlanningEvent::PlanAssigned {
				planner_id: self.id.clone(),
				plan_id,
				worker_id: worker_id.to_string(),
				timeout_seconds,
				timestamp: Utc::now(),
			};
			self.add_event(event.clone());
			self.apply(&event)?;
		}
		Ok(())
	}
}
