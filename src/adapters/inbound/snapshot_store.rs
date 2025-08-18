use crate::common::{SnapshotStore, Snapshot};
use async_trait::async_trait;
use std::collections::HashMap;
use tokio::sync::RwLock;

/// In-memory snapshot store implementation for testing and development
#[derive(Debug, Default)]
pub struct InMemorySnapshotStore {
    snapshots: RwLock<HashMap<String, Vec<Snapshot>>>,
}

impl InMemorySnapshotStore {
    pub fn new() -> Self {
        Self {
            snapshots: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl SnapshotStore for InMemorySnapshotStore {
    async fn save_snapshot(&self, snapshot: Snapshot) -> Result<(), String> {
        let mut store = self.snapshots.write().await;
        
        let aggregate_snapshots = store
            .entry(snapshot.aggregate_id.clone())
            .or_insert_with(Vec::new);
        
        aggregate_snapshots.push(snapshot);
        
        // Sort by version (latest first)
        aggregate_snapshots.sort_by(|a, b| b.aggregate_version.cmp(&a.aggregate_version));
        
        Ok(())
    }

    async fn load_snapshot(
        &self,
        aggregate_id: &str,
        max_version: Option<u64>,
    ) -> Result<Option<Snapshot>, String> {
        let store = self.snapshots.read().await;
        
        if let Some(snapshots) = store.get(aggregate_id) {
            for snapshot in snapshots {
                if let Some(max_ver) = max_version {
                    if snapshot.aggregate_version <= max_ver {
                        return Ok(Some(snapshot.clone()));
                    }
                } else {
                    return Ok(Some(snapshot.clone()));
                }
            }
        }
        
        Ok(None)
    }

    async fn delete_snapshots_before(
        &self,
        aggregate_id: &str,
        version: u64,
    ) -> Result<(), String> {
        let mut store = self.snapshots.write().await;
        
        if let Some(snapshots) = store.get_mut(aggregate_id) {
            snapshots.retain(|snapshot| snapshot.aggregate_version >= version);
        }
        
        Ok(())
    }
}
