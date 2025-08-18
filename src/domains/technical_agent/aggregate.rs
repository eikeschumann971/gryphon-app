use crate::common::{AggregateRoot, DomainResult, DomainError};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::events::TechnicalAgentEvent;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalAgent {
    pub id: String,
    pub name: String,
    pub agent_type: AgentType,
    pub capabilities: Vec<Capability>,
    pub hardware_specs: HardwareSpecs,
    pub software_modules: Vec<SoftwareModule>,
    pub status: TechnicalStatus,
    pub configuration: Configuration,
    pub version: u64,
    #[serde(skip)]
    uncommitted_events: Vec<TechnicalAgentEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentType {
    Drone,
    Robot,
    Vehicle,
    Sensor,
    Actuator,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub parameters: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareSpecs {
    pub processor: String,
    pub memory_gb: f64,
    pub storage_gb: f64,
    pub sensors: Vec<Sensor>,
    pub actuators: Vec<Actuator>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sensor {
    pub id: Uuid,
    pub sensor_type: String,
    pub model: String,
    pub accuracy: f64,
    pub range: f64,
    pub status: ComponentStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Actuator {
    pub id: Uuid,
    pub actuator_type: String,
    pub model: String,
    pub max_force: f64,
    pub precision: f64,
    pub status: ComponentStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentStatus {
    Online,
    Offline,
    Error(String),
    Maintenance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwareModule {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub enabled: bool,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TechnicalStatus {
    Initializing,
    Ready,
    Active,
    Idle,
    Maintenance,
    Error(String),
    Shutdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Configuration {
    pub parameters: std::collections::HashMap<String, String>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl TechnicalAgent {
    pub fn new(id: String, name: String, agent_type: AgentType) -> Self {
        let mut agent = Self {
            id: id.clone(),
            name: name.clone(),
            agent_type: agent_type.clone(),
            capabilities: Vec::new(),
            hardware_specs: HardwareSpecs {
                processor: "Unknown".to_string(),
                memory_gb: 0.0,
                storage_gb: 0.0,
                sensors: Vec::new(),
                actuators: Vec::new(),
            },
            software_modules: Vec::new(),
            status: TechnicalStatus::Initializing,
            configuration: Configuration {
                parameters: std::collections::HashMap::new(),
                last_updated: chrono::Utc::now(),
            },
            version: 0,
            uncommitted_events: Vec::new(),
        };

        let event = TechnicalAgentEvent::AgentCreated {
            agent_id: id,
            name,
            agent_type,
            timestamp: chrono::Utc::now(),
        };
        
        agent.add_event(event);
        agent
    }

    pub fn add_capability(&mut self, name: String, description: String) -> DomainResult<Uuid> {
        let capability_id = Uuid::new_v4();
        let capability = Capability {
            id: capability_id,
            name: name.clone(),
            description: description.clone(),
            enabled: true,
            parameters: std::collections::HashMap::new(),
        };

        self.capabilities.push(capability);

        let event = TechnicalAgentEvent::CapabilityAdded {
            agent_id: self.id.clone(),
            capability_id,
            name,
            description,
            timestamp: chrono::Utc::now(),
        };

        self.add_event(event);
        Ok(capability_id)
    }

    pub fn update_status(&mut self, new_status: TechnicalStatus) -> DomainResult<()> {
        self.status = new_status.clone();

        let event = TechnicalAgentEvent::StatusChanged {
            agent_id: self.id.clone(),
            new_status,
            timestamp: chrono::Utc::now(),
        };

        self.add_event(event);
        Ok(())
    }
}

impl AggregateRoot for TechnicalAgent {
    type Event = TechnicalAgentEvent;

    fn aggregate_id(&self) -> &str {
        &self.id
    }

    fn version(&self) -> u64 {
        self.version
    }

    fn apply(&mut self, event: &Self::Event) -> DomainResult<()> {
        match event {
            TechnicalAgentEvent::AgentCreated { agent_id, name, agent_type, .. } => {
                self.id = agent_id.clone();
                self.name = name.clone();
                self.agent_type = agent_type.clone();
                self.status = TechnicalStatus::Initializing;
            }
            TechnicalAgentEvent::CapabilityAdded { capability_id, name, description, .. } => {
                let capability = Capability {
                    id: *capability_id,
                    name: name.clone(),
                    description: description.clone(),
                    enabled: true,
                    parameters: std::collections::HashMap::new(),
                };
                self.capabilities.push(capability);
            }
            TechnicalAgentEvent::StatusChanged { new_status, .. } => {
                self.status = new_status.clone();
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
