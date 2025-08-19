use super::aggregate::{Acceleration3D, KinematicsModel, Orientation, Position3D, Velocity3D};
use crate::common::DomainEvent;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KinematicAgentEvent {
    AgentCreated {
        agent_id: String,
        initial_position: Position3D,
        kinematics_model: KinematicsModel,
        timestamp: DateTime<Utc>,
    },
    PositionUpdated {
        agent_id: String,
        new_position: Position3D,
        timestamp: DateTime<Utc>,
    },
    VelocityUpdated {
        agent_id: String,
        new_velocity: Velocity3D,
        timestamp: DateTime<Utc>,
    },
    AccelerationUpdated {
        agent_id: String,
        new_acceleration: Acceleration3D,
        timestamp: DateTime<Utc>,
    },
    OrientationUpdated {
        agent_id: String,
        new_orientation: Orientation,
        timestamp: DateTime<Utc>,
    },
    TrajectoryStarted {
        agent_id: String,
        trajectory_id: String,
        start_position: Position3D,
        target_position: Position3D,
        timestamp: DateTime<Utc>,
    },
    TrajectoryCompleted {
        agent_id: String,
        trajectory_id: String,
        final_position: Position3D,
        timestamp: DateTime<Utc>,
    },
    CollisionDetected {
        agent_id: String,
        collision_point: Position3D,
        other_agent_id: Option<String>,
        timestamp: DateTime<Utc>,
    },
}

impl DomainEvent for KinematicAgentEvent {
    fn event_type(&self) -> &'static str {
        match self {
            KinematicAgentEvent::AgentCreated { .. } => "KinematicAgentCreated",
            KinematicAgentEvent::PositionUpdated { .. } => "PositionUpdated",
            KinematicAgentEvent::VelocityUpdated { .. } => "VelocityUpdated",
            KinematicAgentEvent::AccelerationUpdated { .. } => "AccelerationUpdated",
            KinematicAgentEvent::OrientationUpdated { .. } => "OrientationUpdated",
            KinematicAgentEvent::TrajectoryStarted { .. } => "TrajectoryStarted",
            KinematicAgentEvent::TrajectoryCompleted { .. } => "TrajectoryCompleted",
            KinematicAgentEvent::CollisionDetected { .. } => "CollisionDetected",
        }
    }

    fn aggregate_id(&self) -> &str {
        match self {
            KinematicAgentEvent::AgentCreated { agent_id, .. } => agent_id,
            KinematicAgentEvent::PositionUpdated { agent_id, .. } => agent_id,
            KinematicAgentEvent::VelocityUpdated { agent_id, .. } => agent_id,
            KinematicAgentEvent::AccelerationUpdated { agent_id, .. } => agent_id,
            KinematicAgentEvent::OrientationUpdated { agent_id, .. } => agent_id,
            KinematicAgentEvent::TrajectoryStarted { agent_id, .. } => agent_id,
            KinematicAgentEvent::TrajectoryCompleted { agent_id, .. } => agent_id,
            KinematicAgentEvent::CollisionDetected { agent_id, .. } => agent_id,
        }
    }

    fn event_version(&self) -> u64 {
        1
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        match self {
            KinematicAgentEvent::AgentCreated { timestamp, .. } => *timestamp,
            KinematicAgentEvent::PositionUpdated { timestamp, .. } => *timestamp,
            KinematicAgentEvent::VelocityUpdated { timestamp, .. } => *timestamp,
            KinematicAgentEvent::AccelerationUpdated { timestamp, .. } => *timestamp,
            KinematicAgentEvent::OrientationUpdated { timestamp, .. } => *timestamp,
            KinematicAgentEvent::TrajectoryStarted { timestamp, .. } => *timestamp,
            KinematicAgentEvent::TrajectoryCompleted { timestamp, .. } => *timestamp,
            KinematicAgentEvent::CollisionDetected { timestamp, .. } => *timestamp,
        }
    }
}
