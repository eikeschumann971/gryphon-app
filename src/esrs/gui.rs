use esrs::Aggregate;
use serde::{Deserialize, Serialize};
use crate::domains::gui::events::GUIEvent;
use chrono::Utc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GUIState {
    pub id: String,
    pub name: String,
}

impl Default for GUIState {
    fn default() -> Self {
        Self { id: String::new(), name: String::new() }
    }
}

#[derive(Debug, Clone)]
pub enum GUICommand {
    CreateApplication { app_id: String, name: String },
    CreateWindow { app_id: String, window_id: uuid::Uuid, title: String },
}

#[derive(Debug, thiserror::Error)]
pub enum GUIError {
    #[error("invalid command: {0}")]
    InvalidCommand(String),
}

pub struct GUIAggregate;

impl Aggregate for GUIAggregate {
    const NAME: &'static str = "gui";
    type State = GUIState;
    type Command = GUICommand;
    type Event = GUIEvent;
    type Error = GUIError;

    fn handle_command(_state: &Self::State, command: Self::Command) -> Result<Vec<Self::Event>, Self::Error> {
        match command {
            GUICommand::CreateApplication { app_id, name } => Ok(vec![GUIEvent::ApplicationCreated { app_id, name, timestamp: Utc::now() }]),
            GUICommand::CreateWindow { app_id, window_id, title } => Ok(vec![GUIEvent::WindowCreated { app_id, window_id, title, window_type: crate::domains::gui::aggregate::WindowType::Main, timestamp: Utc::now() }]),
        }
    }

    fn apply_event(_state: Self::State, event: Self::Event) -> Self::State {
        match event {
            GUIEvent::ApplicationCreated { app_id, name, .. } => GUIState { id: app_id, name },
            GUIEvent::WindowCreated { app_id, .. } => GUIState { id: app_id, name: String::new() },
            _ => GUIState::default(),
        }
    }
}
