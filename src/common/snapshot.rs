use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub snapshot_id: Uuid,
    pub aggregate_id: String,
    pub aggregate_type: String,
    pub aggregate_version: u64,
    pub snapshot_data: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

impl Snapshot {
    pub fn new<T: Serialize>(
        aggregate_id: &str,
        aggregate_type: &str,
        aggregate_version: u64,
        aggregate_data: &T,
    ) -> Result<Self, serde_json::Error> {
        Ok(Self {
            snapshot_id: Uuid::new_v4(),
            aggregate_id: aggregate_id.to_string(),
            aggregate_type: aggregate_type.to_string(),
            aggregate_version,
            snapshot_data: serde_json::to_value(aggregate_data)?,
            created_at: Utc::now(),
        })
    }
}

#[async_trait::async_trait]
pub trait SnapshotStore {
    async fn save_snapshot(&self, snapshot: Snapshot) -> Result<(), String>;
    
    async fn load_snapshot(
        &self,
        aggregate_id: &str,
        max_version: Option<u64>,
    ) -> Result<Option<Snapshot>, String>;
    
    async fn delete_snapshots_before(
        &self,
        aggregate_id: &str,
        version: u64,
    ) -> Result<(), String>;
}
