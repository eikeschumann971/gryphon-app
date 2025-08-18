use crate::common::DomainResult;
use async_trait::async_trait;

/// Port trait that the path_planning domain depends on for loading map data.
/// Implementations (adapters) will provide filesystem or network-backed sources.
pub trait PathPlanningDataSource: Send + Sync {
    fn load_geojson(&self, name: &str) -> DomainResult<String>;
    fn load_graph_bytes(&self, name: &str) -> DomainResult<Vec<u8>>;
}

/// Port for storing and retrieving graphs in various backends (filesystem, postgres, s3, ...)
pub trait GraphStore: Send + Sync {
    /// Save raw graph bytes under the given name
    fn save_graph_bytes(&self, name: &str, bytes: &[u8]) -> DomainResult<()>;
    /// Load raw graph bytes previously saved
    fn load_graph_bytes(&self, name: &str) -> DomainResult<Vec<u8>>;
    /// Delete a stored graph
    fn delete_graph(&self, name: &str) -> DomainResult<()>;
}

/// Async variant of GraphStore for adapters that perform async I/O (Postgres, S3, etc.)
#[async_trait]
pub trait GraphStoreAsync: Send + Sync {
    async fn save_graph_bytes(&self, name: &str, bytes: &[u8]) -> DomainResult<()>;
    async fn load_graph_bytes(&self, name: &str) -> DomainResult<Vec<u8>>;
    async fn delete_graph(&self, name: &str) -> DomainResult<()>;
}
