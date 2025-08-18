use crate::common::{AggregateRoot, DomainResult, DomainError};
use serde::{Deserialize, Serialize};
use super::events::KinematicAgentEvent;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KinematicAgent {
    pub id: String,
    pub position: Position3D,
    pub velocity: Velocity3D,
    pub acceleration: Acceleration3D,
    pub orientation: Orientation,
    pub angular_velocity: AngularVelocity,
    pub kinematics_model: KinematicsModel,
    pub constraints: MovementConstraints,
    pub version: u64,
    #[serde(skip)]
    uncommitted_events: Vec<KinematicAgentEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Velocity3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Acceleration3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Orientation {
    pub roll: f64,
    pub pitch: f64,
    pub yaw: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AngularVelocity {
    pub roll_rate: f64,
    pub pitch_rate: f64,
    pub yaw_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KinematicsModel {
    PointMass,
    RigidBody,
    Articulated { joint_count: usize },
    Differential,
    Holonomic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovementConstraints {
    pub max_velocity: f64,
    pub max_acceleration: f64,
    pub max_angular_velocity: f64,
    pub min_turning_radius: f64,
    pub workspace_bounds: WorkspaceBounds,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceBounds {
    pub min_x: f64,
    pub max_x: f64,
    pub min_y: f64,
    pub max_y: f64,
    pub min_z: f64,
    pub max_z: f64,
}

impl KinematicAgent {
    pub fn new(id: String, initial_position: Position3D, kinematics_model: KinematicsModel) -> Self {
        let mut agent = Self {
            id: id.clone(),
            position: initial_position.clone(),
            velocity: Velocity3D { x: 0.0, y: 0.0, z: 0.0 },
            acceleration: Acceleration3D { x: 0.0, y: 0.0, z: 0.0 },
            orientation: Orientation { roll: 0.0, pitch: 0.0, yaw: 0.0 },
            angular_velocity: AngularVelocity { roll_rate: 0.0, pitch_rate: 0.0, yaw_rate: 0.0 },
            kinematics_model: kinematics_model.clone(),
            constraints: MovementConstraints {
                max_velocity: 10.0,
                max_acceleration: 5.0,
                max_angular_velocity: 3.14,
                min_turning_radius: 1.0,
                workspace_bounds: WorkspaceBounds {
                    min_x: -100.0, max_x: 100.0,
                    min_y: -100.0, max_y: 100.0,
                    min_z: 0.0, max_z: 50.0,
                },
            },
            version: 0,
            uncommitted_events: Vec::new(),
        };

        let event = KinematicAgentEvent::AgentCreated {
            agent_id: id,
            initial_position,
            kinematics_model,
            timestamp: chrono::Utc::now(),
        };
        
        agent.add_event(event);
        agent
    }

    pub fn update_position(&mut self, new_position: Position3D) -> DomainResult<()> {
        if !self.is_position_valid(&new_position) {
            return Err(DomainError::InvalidCommand {
                reason: "Position violates workspace constraints".to_string(),
            });
        }

        self.position = new_position.clone();

        let event = KinematicAgentEvent::PositionUpdated {
            agent_id: self.id.clone(),
            new_position,
            timestamp: chrono::Utc::now(),
        };

        self.add_event(event);
        Ok(())
    }

    fn is_position_valid(&self, position: &Position3D) -> bool {
        let bounds = &self.constraints.workspace_bounds;
        position.x >= bounds.min_x && position.x <= bounds.max_x &&
        position.y >= bounds.min_y && position.y <= bounds.max_y &&
        position.z >= bounds.min_z && position.z <= bounds.max_z
    }
}

impl AggregateRoot for KinematicAgent {
    type Event = KinematicAgentEvent;

    fn aggregate_id(&self) -> &str {
        &self.id
    }

    fn version(&self) -> u64 {
        self.version
    }

    fn apply(&mut self, event: &Self::Event) -> DomainResult<()> {
        match event {
            KinematicAgentEvent::AgentCreated { agent_id, initial_position, kinematics_model, .. } => {
                self.id = agent_id.clone();
                self.position = initial_position.clone();
                self.kinematics_model = kinematics_model.clone();
            }
            KinematicAgentEvent::PositionUpdated { new_position, .. } => {
                self.position = new_position.clone();
            }
            KinematicAgentEvent::VelocityUpdated { new_velocity, .. } => {
                self.velocity = new_velocity.clone();
            }
            _ => {}
        }
        self.version += 1;
        Ok(())
    }

    fn uncommitted_events(&self) -> &[Self::Event] {
        &self.uncommitted_events
    }

    fn mark_events_as_committed(&mut self) {
        self.uncommitted_events.clear();
    }

    fn add_event(&mut self, event: Self::Event) {
        self.uncommitted_events.push(event);
    }
}
