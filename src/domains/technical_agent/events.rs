use super::aggregate::{AgentType, ComponentStatus, TechnicalStatus};
use crate::common::DomainEvent;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TechnicalAgentEvent {
    AgentCreated {
        agent_id: String,
        name: String,
        agent_type: AgentType,
        timestamp: DateTime<Utc>,
    },
    CapabilityAdded {
        agent_id: String,
        capability_id: Uuid,
        name: String,
        description: String,
        timestamp: DateTime<Utc>,
    },
    CapabilityEnabled {
        agent_id: String,
        capability_id: Uuid,
        timestamp: DateTime<Utc>,
    },
    CapabilityDisabled {
        agent_id: String,
        capability_id: Uuid,
        timestamp: DateTime<Utc>,
    },
    StatusChanged {
        agent_id: String,
        new_status: TechnicalStatus,
        timestamp: DateTime<Utc>,
    },
    SensorAdded {
        agent_id: String,
        sensor_id: Uuid,
        sensor_type: String,
        model: String,
        timestamp: DateTime<Utc>,
    },
    SensorStatusChanged {
        agent_id: String,
        sensor_id: Uuid,
        new_status: ComponentStatus,
        timestamp: DateTime<Utc>,
    },
    ActuatorAdded {
        agent_id: String,
        actuator_id: Uuid,
        actuator_type: String,
        model: String,
        timestamp: DateTime<Utc>,
    },
    ActuatorStatusChanged {
        agent_id: String,
        actuator_id: Uuid,
        new_status: ComponentStatus,
        timestamp: DateTime<Utc>,
    },
    SoftwareModuleInstalled {
        agent_id: String,
        module_id: Uuid,
        name: String,
        version: String,
        timestamp: DateTime<Utc>,
    },
    ConfigurationUpdated {
        agent_id: String,
        parameter: String,
        old_value: String,
        new_value: String,
        timestamp: DateTime<Utc>,
    },
}

impl DomainEvent for TechnicalAgentEvent {
    fn event_type(&self) -> &'static str {
        match self {
            TechnicalAgentEvent::AgentCreated { .. } => "TechnicalAgentCreated",
            TechnicalAgentEvent::CapabilityAdded { .. } => "CapabilityAdded",
            TechnicalAgentEvent::CapabilityEnabled { .. } => "CapabilityEnabled",
            TechnicalAgentEvent::CapabilityDisabled { .. } => "CapabilityDisabled",
            TechnicalAgentEvent::StatusChanged { .. } => "StatusChanged",
            TechnicalAgentEvent::SensorAdded { .. } => "SensorAdded",
            TechnicalAgentEvent::SensorStatusChanged { .. } => "SensorStatusChanged",
            TechnicalAgentEvent::ActuatorAdded { .. } => "ActuatorAdded",
            TechnicalAgentEvent::ActuatorStatusChanged { .. } => "ActuatorStatusChanged",
            TechnicalAgentEvent::SoftwareModuleInstalled { .. } => "SoftwareModuleInstalled",
            TechnicalAgentEvent::ConfigurationUpdated { .. } => "ConfigurationUpdated",
        }
    }

    fn aggregate_id(&self) -> &str {
        match self {
            TechnicalAgentEvent::AgentCreated { agent_id, .. } => agent_id,
            TechnicalAgentEvent::CapabilityAdded { agent_id, .. } => agent_id,
            TechnicalAgentEvent::CapabilityEnabled { agent_id, .. } => agent_id,
            TechnicalAgentEvent::CapabilityDisabled { agent_id, .. } => agent_id,
            TechnicalAgentEvent::StatusChanged { agent_id, .. } => agent_id,
            TechnicalAgentEvent::SensorAdded { agent_id, .. } => agent_id,
            TechnicalAgentEvent::SensorStatusChanged { agent_id, .. } => agent_id,
            TechnicalAgentEvent::ActuatorAdded { agent_id, .. } => agent_id,
            TechnicalAgentEvent::ActuatorStatusChanged { agent_id, .. } => agent_id,
            TechnicalAgentEvent::SoftwareModuleInstalled { agent_id, .. } => agent_id,
            TechnicalAgentEvent::ConfigurationUpdated { agent_id, .. } => agent_id,
        }
    }

    fn event_version(&self) -> u64 {
        1
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        match self {
            TechnicalAgentEvent::AgentCreated { timestamp, .. } => *timestamp,
            TechnicalAgentEvent::CapabilityAdded { timestamp, .. } => *timestamp,
            TechnicalAgentEvent::CapabilityEnabled { timestamp, .. } => *timestamp,
            TechnicalAgentEvent::CapabilityDisabled { timestamp, .. } => *timestamp,
            TechnicalAgentEvent::StatusChanged { timestamp, .. } => *timestamp,
            TechnicalAgentEvent::SensorAdded { timestamp, .. } => *timestamp,
            TechnicalAgentEvent::SensorStatusChanged { timestamp, .. } => *timestamp,
            TechnicalAgentEvent::ActuatorAdded { timestamp, .. } => *timestamp,
            TechnicalAgentEvent::ActuatorStatusChanged { timestamp, .. } => *timestamp,
            TechnicalAgentEvent::SoftwareModuleInstalled { timestamp, .. } => *timestamp,
            TechnicalAgentEvent::ConfigurationUpdated { timestamp, .. } => *timestamp,
        }
    }
}
