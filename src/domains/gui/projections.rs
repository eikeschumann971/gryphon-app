// GUI projections - simplified implementation
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GUIProjection {
    pub app_id: String,
    pub active_windows_count: usize,
    pub active_sessions_count: usize,
    pub total_interactions: usize,
}
