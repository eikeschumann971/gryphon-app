use crate::common::DomainResult;
use crate::common::DomainError;
use crate::domains::path_planning::ports::{GraphStore, GraphStoreAsync};
use async_trait::async_trait;
use std::sync::Arc;
use tokio_postgres::{NoTls, Error as PgError};

pub struct PostgresGraphStore {
    conn_str: String,
}

impl PostgresGraphStore {
    pub fn new(conn_str: String) -> Self {
        Self { conn_str }
    }

    async fn get_client(&self) -> Result<tokio_postgres::Client, PgError> {
        let (client, connection) = tokio_postgres::connect(&self.conn_str, NoTls).await?;
        // spawn connection to run in background
        tokio::spawn(async move {
            if let Err(e) = connection.await { eprintln!("Postgres connection error: {}", e); }
        });
        Ok(client)
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
        Err(DomainError::InfrastructureError("PostgresGraphStore.delete_graph is not implemented synchronously.".to_string()))
    }
}

#[async_trait]
impl GraphStoreAsync for PostgresGraphStore {
    async fn save_graph_bytes(&self, name: &str, bytes: &[u8]) -> DomainResult<()> {
        let client = self.get_client().await.map_err(|e| DomainError::InfrastructureError(format!("pg connect: {}", e)))?;
        // ensure table exists
        client.execute("CREATE TABLE IF NOT EXISTS graphs (name TEXT PRIMARY KEY, data BYTEA)", &[]).await.map_err(|e| DomainError::InfrastructureError(format!("pg create table: {}", e)))?;
        // upsert
        client.execute("INSERT INTO graphs (name, data) VALUES ($1, $2) ON CONFLICT (name) DO UPDATE SET data = EXCLUDED.data", &[&name, &bytes]).await.map_err(|e| DomainError::InfrastructureError(format!("pg insert: {}", e)))?;
        Ok(())
    }

    async fn load_graph_bytes(&self, name: &str) -> DomainResult<Vec<u8>> {
        let client = self.get_client().await.map_err(|e| DomainError::InfrastructureError(format!("pg connect: {}", e)))?;
        let row = client.query_one("SELECT data FROM graphs WHERE name = $1", &[&name]).await.map_err(|e| DomainError::InfrastructureError(format!("pg query: {}", e)))?;
        let data: Vec<u8> = row.get(0);
        Ok(data)
    }

    async fn delete_graph(&self, name: &str) -> DomainResult<()> {
        let client = self.get_client().await.map_err(|e| DomainError::InfrastructureError(format!("pg connect: {}", e)))?;
        client.execute("DELETE FROM graphs WHERE name = $1", &[&name]).await.map_err(|e| DomainError::InfrastructureError(format!("pg delete: {}", e)))?;
        Ok(())
    }
}
