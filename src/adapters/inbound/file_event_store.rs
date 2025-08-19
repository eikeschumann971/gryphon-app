use crate::common::{EventEnvelope, EventStore};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json;
use std::path::PathBuf;
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

/// File-based EventStore implementation for testing and development
///
/// This implementation stores events in JSON Lines format (one JSON object per line)
/// Events are appended to files named by aggregate_id
/// This allows multiple processes to share the same event store through the file system
pub struct FileEventStore {
    base_path: PathBuf,
}

impl FileEventStore {
    pub fn new<P: Into<PathBuf>>(base_path: P) -> Self {
        let base_path = base_path.into();
        Self { base_path }
    }

    /// Get the file path for a specific aggregate
    fn get_file_path(&self, aggregate_id: &str) -> PathBuf {
        self.base_path.join(format!("{}.jsonl", aggregate_id))
    }

    /// Ensure the base directory exists
    async fn ensure_base_dir(&self) -> Result<(), String> {
        if let Some(parent) = self.base_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| format!("Failed to create base directory: {}", e))?;
        }
        tokio::fs::create_dir_all(&self.base_path)
            .await
            .map_err(|e| format!("Failed to create event store directory: {}", e))?;
        Ok(())
    }
}

#[async_trait]
impl EventStore for FileEventStore {
    async fn append_events(
        &self,
        aggregate_id: &str,
        _expected_version: u64,
        events: Vec<EventEnvelope>,
    ) -> Result<(), String> {
        self.ensure_base_dir().await?;

        let file_path = self.get_file_path(aggregate_id);

        // For simplicity in this demo, we'll ignore version checking
        // In a production system, you'd want to implement proper optimistic concurrency control

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)
            .await
            .map_err(|e| format!("Failed to open event file {}: {}", file_path.display(), e))?;

        for event in events {
            let json_line = serde_json::to_string(&event)
                .map_err(|e| format!("Failed to serialize event: {}", e))?;

            file.write_all(json_line.as_bytes())
                .await
                .map_err(|e| format!("Failed to write event: {}", e))?;
            file.write_all(b"\n")
                .await
                .map_err(|e| format!("Failed to write newline: {}", e))?;
        }

        file.flush()
            .await
            .map_err(|e| format!("Failed to flush file: {}", e))?;

        Ok(())
    }

    async fn load_events(
        &self,
        aggregate_id: &str,
        from_version: u64,
    ) -> Result<Vec<EventEnvelope>, String> {
        let file_path = self.get_file_path(aggregate_id);

        if !file_path.exists() {
            return Ok(vec![]);
        }

        let file = File::open(&file_path)
            .await
            .map_err(|e| format!("Failed to open event file {}: {}", file_path.display(), e))?;

        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        let mut events = Vec::new();
        let mut line_number = 0u64;

        while let Some(line) = lines
            .next_line()
            .await
            .map_err(|e| format!("Failed to read line: {}", e))?
        {
            if line_number >= from_version {
                let event: EventEnvelope = serde_json::from_str(&line).map_err(|e| {
                    format!(
                        "Failed to deserialize event at line {}: {}",
                        line_number + 1,
                        e
                    )
                })?;
                events.push(event);
            }
            line_number += 1;
        }

        Ok(events)
    }

    async fn load_events_by_type(
        &self,
        event_type: &str,
        from_timestamp: Option<DateTime<Utc>>,
    ) -> Result<Vec<EventEnvelope>, String> {
        self.ensure_base_dir().await?;

        let mut all_events = Vec::new();

        // Read all .jsonl files in the directory
        let mut dir = tokio::fs::read_dir(&self.base_path)
            .await
            .map_err(|e| format!("Failed to read directory: {}", e))?;

        while let Some(entry) = dir
            .next_entry()
            .await
            .map_err(|e| format!("Failed to read directory entry: {}", e))?
        {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
                let file = File::open(&path)
                    .await
                    .map_err(|e| format!("Failed to open file {}: {}", path.display(), e))?;

                let reader = BufReader::new(file);
                let mut lines = reader.lines();

                while let Some(line) = lines
                    .next_line()
                    .await
                    .map_err(|e| format!("Failed to read line: {}", e))?
                {
                    let event: EventEnvelope = serde_json::from_str(&line)
                        .map_err(|e| format!("Failed to deserialize event: {}", e))?;

                    // Filter by event type
                    if event.event_type == event_type {
                        // Filter by timestamp if provided
                        if let Some(from_ts) = from_timestamp {
                            if event.occurred_at >= from_ts {
                                all_events.push(event);
                            }
                        } else {
                            all_events.push(event);
                        }
                    }
                }
            }
        }

        // Sort by timestamp
        all_events.sort_by(|a, b| a.occurred_at.cmp(&b.occurred_at));

        Ok(all_events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::EventMetadata;
    use tempfile::TempDir;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_file_event_store() {
        let temp_dir = TempDir::new().unwrap();
        let store = FileEventStore::new(temp_dir.path().join("events"));

        let aggregate_id = "test-aggregate";
        let event = EventEnvelope {
            event_id: Uuid::new_v4(),
            aggregate_id: aggregate_id.to_string(),
            aggregate_type: "TestAggregate".to_string(),
            event_type: "TestEvent".to_string(),
            event_version: 1,
            event_data: serde_json::json!({"test": "data"}),
            metadata: EventMetadata {
                correlation_id: Some(Uuid::new_v4()),
                causation_id: None,
                user_id: Some("test-user".to_string()),
                source: "test".to_string(),
            },
            occurred_at: Utc::now(),
        };

        // Append event
        store
            .append_events(aggregate_id, 0, vec![event.clone()])
            .await
            .unwrap();

        // Load events
        let loaded_events = store.load_events(aggregate_id, 0).await.unwrap();
        assert_eq!(loaded_events.len(), 1);
        assert_eq!(loaded_events[0].event_id, event.event_id);

        // Load by type
        let events_by_type = store.load_events_by_type("TestEvent", None).await.unwrap();
        assert_eq!(events_by_type.len(), 1);
        assert_eq!(events_by_type[0].event_id, event.event_id);
    }
}
