use super::aggregate::WindowType;
use crate::common::DomainEvent;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GUIEvent {
    ApplicationCreated {
        app_id: String,
        name: String,
        timestamp: DateTime<Utc>,
    },
    WindowCreated {
        app_id: String,
        window_id: Uuid,
        title: String,
        window_type: WindowType,
        timestamp: DateTime<Utc>,
    },
    WindowClosed {
        app_id: String,
        window_id: Uuid,
        timestamp: DateTime<Utc>,
    },
    UserSessionStarted {
        app_id: String,
        session_id: Uuid,
        user_id: String,
        timestamp: DateTime<Utc>,
    },
    UserSessionEnded {
        app_id: String,
        session_id: Uuid,
        timestamp: DateTime<Utc>,
    },
    UserInteraction {
        app_id: String,
        session_id: Uuid,
        window_id: Uuid,
        component_id: Uuid,
        interaction_type: String,
        timestamp: DateTime<Utc>,
    },
}

impl DomainEvent for GUIEvent {
    fn event_type(&self) -> &'static str {
        match self {
            GUIEvent::ApplicationCreated { .. } => "ApplicationCreated",
            GUIEvent::WindowCreated { .. } => "WindowCreated",
            GUIEvent::WindowClosed { .. } => "WindowClosed",
            GUIEvent::UserSessionStarted { .. } => "UserSessionStarted",
            GUIEvent::UserSessionEnded { .. } => "UserSessionEnded",
            GUIEvent::UserInteraction { .. } => "UserInteraction",
        }
    }

    fn aggregate_id(&self) -> &str {
        match self {
            GUIEvent::ApplicationCreated { app_id, .. } => app_id,
            GUIEvent::WindowCreated { app_id, .. } => app_id,
            GUIEvent::WindowClosed { app_id, .. } => app_id,
            GUIEvent::UserSessionStarted { app_id, .. } => app_id,
            GUIEvent::UserSessionEnded { app_id, .. } => app_id,
            GUIEvent::UserInteraction { app_id, .. } => app_id,
        }
    }

    fn event_version(&self) -> u64 {
        1
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        match self {
            GUIEvent::ApplicationCreated { timestamp, .. } => *timestamp,
            GUIEvent::WindowCreated { timestamp, .. } => *timestamp,
            GUIEvent::WindowClosed { timestamp, .. } => *timestamp,
            GUIEvent::UserSessionStarted { timestamp, .. } => *timestamp,
            GUIEvent::UserSessionEnded { timestamp, .. } => *timestamp,
            GUIEvent::UserInteraction { timestamp, .. } => *timestamp,
        }
    }
}
