use crate::common::DomainResult;

/// Port trait that the path_planning domain depends on for loading map data.
/// Implementations (adapters) will provide filesystem or network-backed sources.
pub trait PathPlanningDataSource: Send + Sync {
    fn load_geojson(&self, name: &str) -> DomainResult<String>;
    fn load_graph_bytes(&self, name: &str) -> DomainResult<Vec<u8>>;
}
