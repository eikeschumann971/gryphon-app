use super::events::GUIEvent;
use crate::common::{AggregateRoot, DomainResult};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GUIApplication {
    pub id: String,
    pub name: String,
    pub windows: Vec<Window>,
    pub active_sessions: Vec<UserSession>,
    pub version: u64,
    #[serde(skip)]
    uncommitted_events: Vec<GUIEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Window {
    pub id: Uuid,
    pub title: String,
    pub window_type: WindowType,
    pub position: (i32, i32),
    pub size: (u32, u32),
    pub visible: bool,
    pub components: Vec<UIComponent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WindowType {
    Main,
    Dialog,
    Toolbar,
    Visualization,
    Console,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIComponent {
    pub id: Uuid,
    pub component_type: ComponentType,
    pub properties: std::collections::HashMap<String, String>,
    pub event_handlers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentType {
    Button,
    TextInput,
    Label,
    Canvas,
    Chart,
    TreeView,
    DataGrid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    pub session_id: Uuid,
    pub user_id: String,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub last_activity: chrono::DateTime<chrono::Utc>,
    pub active_windows: Vec<Uuid>,
}

impl GUIApplication {
    pub fn new(id: String, name: String) -> Self {
        let mut app = Self {
            id: id.clone(),
            name: name.clone(),
            windows: Vec::new(),
            active_sessions: Vec::new(),
            version: 0,
            uncommitted_events: Vec::new(),
        };

        let event = GUIEvent::ApplicationCreated {
            app_id: id,
            name,
            timestamp: chrono::Utc::now(),
        };

        app.add_event(event);
        app
    }
}

impl AggregateRoot for GUIApplication {
    type Event = GUIEvent;
    fn aggregate_id(&self) -> &str {
        &self.id
    }
    fn version(&self) -> u64 {
        self.version
    }
    fn apply(&mut self, _event: &Self::Event) -> DomainResult<()> {
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
