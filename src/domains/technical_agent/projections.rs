use super::aggregate::{AgentType, TechnicalStatus};
use super::events::TechnicalAgentEvent;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalAgentOverview {
    pub agent_id: String,
    pub name: String,
    pub agent_type: AgentType,
    pub status: TechnicalStatus,
    pub capabilities_count: usize,
    pub sensors_count: usize,
    pub actuators_count: usize,
    pub last_activity: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl TechnicalAgentOverview {
    pub fn new(
        agent_id: String,
        name: String,
        agent_type: AgentType,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            agent_id,
            name,
            agent_type,
            status: TechnicalStatus::Initializing,
            capabilities_count: 0,
            sensors_count: 0,
            actuators_count: 0,
            last_activity: created_at,
            created_at,
        }
    }

    pub fn apply_event(&mut self, event: &TechnicalAgentEvent) {
        match event {
            TechnicalAgentEvent::AgentCreated {
                name,
                agent_type,
                timestamp,
                ..
            } => {
                self.name = name.clone();
                self.agent_type = agent_type.clone();
                self.created_at = *timestamp;
                self.last_activity = *timestamp;
            }
            TechnicalAgentEvent::CapabilityAdded { timestamp, .. } => {
                self.capabilities_count += 1;
                self.last_activity = *timestamp;
            }
            TechnicalAgentEvent::StatusChanged {
                new_status,
                timestamp,
                ..
            } => {
                self.status = new_status.clone();
                self.last_activity = *timestamp;
            }
            TechnicalAgentEvent::SensorAdded { timestamp, .. } => {
                self.sensors_count += 1;
                self.last_activity = *timestamp;
            }
            TechnicalAgentEvent::ActuatorAdded { timestamp, .. } => {
                self.actuators_count += 1;
                self.last_activity = *timestamp;
            }
            _ => {}
        }
    }
}
