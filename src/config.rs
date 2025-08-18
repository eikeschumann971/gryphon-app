use serde::{Deserialize, Serialize};
use std::path::Path;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub kafka: KafkaConfig,
    pub postgres: PostgresConfig,
    pub event_store: EventStoreConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KafkaConfig {
    pub brokers: Vec<String>,
    pub client_id: String,
    pub group_id: String,
    pub topics: KafkaTopics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KafkaTopics {
    pub logical_agent_events: String,
    pub technical_agent_events: String,
    pub kinematic_agent_events: String,
    pub path_planning_events: String,
    pub dynamics_events: String,
    pub gui_events: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostgresConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventStoreConfig {
    pub snapshot_frequency: u64,
    pub batch_size: usize,
}

impl Config {
    pub async fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = tokio::fs::read_to_string(path).await?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            kafka: KafkaConfig {
                brokers: vec!["localhost:9092".to_string()],
                client_id: "gryphon-app".to_string(),
                group_id: "gryphon-app-group".to_string(),
                topics: KafkaTopics {
                    logical_agent_events: "logical-agent-events".to_string(),
                    technical_agent_events: "technical-agent-events".to_string(),
                    kinematic_agent_events: "kinematic-agent-events".to_string(),
                    path_planning_events: "path-planning-events".to_string(),
                    dynamics_events: "dynamics-events".to_string(),
                    gui_events: "gui-events".to_string(),
                },
            },
            postgres: PostgresConfig {
                host: "localhost".to_string(),
                port: 5432,
                database: "gryphon_app".to_string(),
                username: "postgres".to_string(),
                password: "password".to_string(),
                max_connections: 10,
            },
            event_store: EventStoreConfig {
                snapshot_frequency: 100,
                batch_size: 50,
            },
        }
    }
}
