use crate::common::{AggregateRoot, DomainResult};
use serde::{Deserialize, Serialize};
use super::events::DynamicsEvent;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicsSimulator {
    pub id: String,
    pub physics_model: PhysicsModel,
    pub simulation_state: SimulationState,
    pub entities: Vec<DynamicEntity>,
    pub version: u64,
    #[serde(skip)]
    uncommitted_events: Vec<DynamicsEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PhysicsModel {
    Newtonian,
    Relativistic,
    Quantum,
    Simplified,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SimulationState {
    Stopped,
    Running,
    Paused,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicEntity {
    pub id: String,
    pub mass: f64,
    pub inertia: InertiaMatrix,
    pub forces: Vec<Force>,
    pub torques: Vec<Torque>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InertiaMatrix {
    pub ixx: f64, pub ixy: f64, pub ixz: f64,
    pub iyx: f64, pub iyy: f64, pub iyz: f64,
    pub izx: f64, pub izy: f64, pub izz: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Force {
    pub x: f64, pub y: f64, pub z: f64,
    pub application_point: (f64, f64, f64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Torque {
    pub x: f64, pub y: f64, pub z: f64,
}

impl DynamicsSimulator {
    pub fn new(id: String, physics_model: PhysicsModel) -> Self {
        let mut simulator = Self {
            id: id.clone(),
            physics_model: physics_model.clone(),
            simulation_state: SimulationState::Stopped,
            entities: Vec::new(),
            version: 0,
            uncommitted_events: Vec::new(),
        };

        let event = DynamicsEvent::SimulatorCreated {
            simulator_id: id,
            physics_model,
            timestamp: chrono::Utc::now(),
        };
        
        simulator.add_event(event);
        simulator
    }
}

impl AggregateRoot for DynamicsSimulator {
    type Event = DynamicsEvent;
    fn aggregate_id(&self) -> &str { &self.id }
    fn version(&self) -> u64 { self.version }
    fn apply(&mut self, _event: &Self::Event) -> DomainResult<()> { 
        self.version += 1; 
        Ok(()) 
    }
    fn uncommitted_events(&self) -> &[Self::Event] { &self.uncommitted_events }
    fn mark_events_as_committed(&mut self) { self.uncommitted_events.clear(); }
    fn add_event(&mut self, event: Self::Event) { self.uncommitted_events.push(event); }
}
