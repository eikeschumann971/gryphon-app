use crate::common::{SnapshotStore, Snapshot};
use crate::config::PostgresConfig;
use async_trait::async_trait;
use deadpool_postgres::{Config, Pool, Runtime};
use tokio_postgres::NoTls;
use uuid::Uuid;

pub struct PostgresSnapshotStore {
    pool: Pool,
}

impl PostgresSnapshotStore {
    pub async fn new(config: PostgresConfig) -> Result<Self, String> {
        let mut pg_config = Config::new();
        pg_config.host = Some(config.host);
        pg_config.port = Some(config.port);
        pg_config.dbname = Some(config.database);
        pg_config.user = Some(config.username);
        pg_config.password = Some(config.password);
        
        let pool = pg_config
            .create_pool(Some(Runtime::Tokio1), NoTls)
            .map_err(|e| format!("Failed to create PostgreSQL pool: {}", e))?;

        let store = Self { pool };
        
        // Initialize database schema
        store.initialize_schema().await?;
        
        Ok(store)
    }

    async fn initialize_schema(&self) -> Result<(), String> {
        let client = self.pool.get().await
            .map_err(|e| format!("Failed to get database connection: {}", e))?;

        let schema = r#"
            CREATE TABLE IF NOT EXISTS snapshots (
                snapshot_id UUID PRIMARY KEY,
                aggregate_id VARCHAR(255) NOT NULL,
                aggregate_type VARCHAR(100) NOT NULL,
                aggregate_version BIGINT NOT NULL,
                snapshot_data JSONB NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                UNIQUE(aggregate_id, aggregate_version)
            );

            CREATE INDEX IF NOT EXISTS idx_snapshots_aggregate_id 
            ON snapshots(aggregate_id);
            
            CREATE INDEX IF NOT EXISTS idx_snapshots_aggregate_type 
            ON snapshots(aggregate_type);
            
            CREATE INDEX IF NOT EXISTS idx_snapshots_version 
            ON snapshots(aggregate_id, aggregate_version DESC);
        "#;

        client.batch_execute(schema).await
            .map_err(|e| format!("Failed to initialize database schema: {}", e))?;

        Ok(())
    }
}

#[async_trait]
impl SnapshotStore for PostgresSnapshotStore {
    async fn save_snapshot(&self, snapshot: Snapshot) -> Result<(), String> {
        let client = self.pool.get().await
            .map_err(|e| format!("Failed to get database connection: {}", e))?;

        let stmt = client.prepare(
            "INSERT INTO snapshots (snapshot_id, aggregate_id, aggregate_type, aggregate_version, snapshot_data, created_at) 
             VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT (aggregate_id, aggregate_version) DO UPDATE SET
             snapshot_data = EXCLUDED.snapshot_data,
             created_at = EXCLUDED.created_at"
        ).await
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

        client.execute(
            &stmt,
            &[
                &snapshot.snapshot_id,
                &snapshot.aggregate_id,
                &snapshot.aggregate_type,
                &(snapshot.aggregate_version as i64),
                &snapshot.snapshot_data,
                &snapshot.created_at,
            ],
        ).await
        .map_err(|e| format!("Failed to save snapshot: {}", e))?;

        Ok(())
    }

    async fn load_snapshot(
        &self,
        aggregate_id: &str,
        max_version: Option<u64>,
    ) -> Result<Option<Snapshot>, String> {
        let client = self.pool.get().await
            .map_err(|e| format!("Failed to get database connection: {}", e))?;

        let (query, params): (&str, Vec<&(dyn tokio_postgres::types::ToSql + Sync)>) = 
            if let Some(max_ver) = max_version {
                (
                    "SELECT snapshot_id, aggregate_id, aggregate_type, aggregate_version, snapshot_data, created_at 
                     FROM snapshots 
                     WHERE aggregate_id = $1 AND aggregate_version <= $2 
                     ORDER BY aggregate_version DESC 
                     LIMIT 1",
                    vec![&aggregate_id, &(max_ver as i64)],
                )
            } else {
                (
                    "SELECT snapshot_id, aggregate_id, aggregate_type, aggregate_version, snapshot_data, created_at 
                     FROM snapshots 
                     WHERE aggregate_id = $1 
                     ORDER BY aggregate_version DESC 
                     LIMIT 1",
                    vec![&aggregate_id],
                )
            };

        let row = client.query_opt(query, &params).await
            .map_err(|e| format!("Failed to load snapshot: {}", e))?;

        if let Some(row) = row {
            Ok(Some(Snapshot {
                snapshot_id: row.get(0),
                aggregate_id: row.get(1),
                aggregate_type: row.get(2),
                aggregate_version: row.get::<_, i64>(3) as u64,
                snapshot_data: row.get(4),
                created_at: row.get(5),
            }))
        } else {
            Ok(None)
        }
    }

    async fn delete_snapshots_before(
        &self,
        aggregate_id: &str,
        version: u64,
    ) -> Result<(), String> {
        let client = self.pool.get().await
            .map_err(|e| format!("Failed to get database connection: {}", e))?;

        client.execute(
            "DELETE FROM snapshots WHERE aggregate_id = $1 AND aggregate_version < $2",
            &[&aggregate_id, &(version as i64)],
        ).await
        .map_err(|e| format!("Failed to delete old snapshots: {}", e))?;

        Ok(())
    }
}
