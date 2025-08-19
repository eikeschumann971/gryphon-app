use crate::common::DomainError;
use crate::common::DomainResult;
use crate::domains::path_planning::ports::{GraphStore, GraphStoreAsync};
use async_trait::async_trait;
use deadpool_postgres::{Client, Pool};
use serde_json::Value as JsonValue;
use tokio_postgres::types::ToSql;

pub struct PostgresGraphStore {
    pool: Pool,
}

impl PostgresGraphStore {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    async fn get_client(&self) -> Result<Client, DomainError> {
        self.pool
            .get()
            .await
            .map_err(|e| DomainError::InfrastructureError(format!("deadpool get client: {}", e)))
    }
}

impl GraphStore for PostgresGraphStore {
    fn save_graph_bytes(&self, _name: &str, _bytes: &[u8]) -> DomainResult<()> {
        // For now, implement sync stub that returns Unsupported until async wiring is added.
        Err(DomainError::InfrastructureError("PostgresGraphStore.save_graph_bytes is not implemented synchronously. Use the async client API.".to_string()))
    }

    fn load_graph_bytes(&self, _name: &str) -> DomainResult<Vec<u8>> {
        Err(DomainError::InfrastructureError("PostgresGraphStore.load_graph_bytes is not implemented synchronously. Use the async client API.".to_string()))
    }

    fn delete_graph(&self, _name: &str) -> DomainResult<()> {
        Err(DomainError::InfrastructureError(
            "PostgresGraphStore.delete_graph is not implemented synchronously.".to_string(),
        ))
    }
}

#[async_trait]
impl GraphStoreAsync for PostgresGraphStore {
    async fn save_graph_bytes(&self, name: &str, bytes: &[u8]) -> DomainResult<()> {
        let client = self.get_client().await?;
        // ensure table exists with metadata columns (version, header jsonb, timestamps)
        client
            .execute(
                "CREATE TABLE IF NOT EXISTS graphs (
                name TEXT PRIMARY KEY,
                data BYTEA NOT NULL,
                version INT,
                header JSONB,
                created_at TIMESTAMPTZ DEFAULT now(),
                updated_at TIMESTAMPTZ DEFAULT now()
            )",
                &[],
            )
            .await
            .map_err(|e| DomainError::InfrastructureError(format!("pg create table: {}", e)))?;

        // attempt to parse header from the provided bytes (filesystem format: PGPH + ver + header_len + header_json + payload)
        let mut version_val: Option<i32> = None;
        let mut header_json: Option<JsonValue> = None;
        if bytes.len() >= 9 && &bytes[0..4] == b"PGPH" {
            let ver = bytes[4];
            let hl = u32::from_le_bytes([bytes[5], bytes[6], bytes[7], bytes[8]]) as usize;
            if bytes.len() >= 9 + hl {
                let header_bytes = &bytes[9..9 + hl];
                if let Ok(hv) = serde_json::from_slice::<JsonValue>(header_bytes) {
                    version_val = Some(ver as i32);
                    header_json = Some(hv);
                }
            }
        }

        // Upsert with metadata
        let version_i32: i32 = version_val.unwrap_or(0);
        if let Some(hj) = header_json {
            let params: &[&(dyn ToSql + Sync)] = &[&name, &bytes, &version_i32, &hj];
            client.execute(
                "INSERT INTO graphs (name, data, version, header) VALUES ($1, $2, $3, $4)
                 ON CONFLICT (name) DO UPDATE SET data = EXCLUDED.data, version = EXCLUDED.version, header = EXCLUDED.header, updated_at = now()",
                params,
            ).await.map_err(|e| DomainError::InfrastructureError(format!("pg insert: {}", e)))?;
        } else {
            let params: &[&(dyn ToSql + Sync)] = &[&name, &bytes];
            client
                .execute(
                    "INSERT INTO graphs (name, data) VALUES ($1, $2)
                 ON CONFLICT (name) DO UPDATE SET data = EXCLUDED.data, updated_at = now()",
                    params,
                )
                .await
                .map_err(|e| DomainError::InfrastructureError(format!("pg insert: {}", e)))?;
        }
        Ok(())
    }

    async fn load_graph_bytes(&self, name: &str) -> DomainResult<Vec<u8>> {
        let client = self.get_client().await?;
        let row = client
            .query_one("SELECT data FROM graphs WHERE name = $1", &[&name])
            .await
            .map_err(|e| DomainError::InfrastructureError(format!("pg query: {}", e)))?;
        let data: Vec<u8> = row.get(0);
        Ok(data)
    }

    async fn delete_graph(&self, name: &str) -> DomainResult<()> {
        let client = self.get_client().await?;
        client
            .execute("DELETE FROM graphs WHERE name = $1", &[&name])
            .await
            .map_err(|e| DomainError::InfrastructureError(format!("pg delete: {}", e)))?;
        Ok(())
    }
}
