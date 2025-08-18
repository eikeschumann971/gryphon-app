use std::path::{PathBuf, Path};
use std::env;
use std::fs;
use std::io::{Read, Write};
use crate::domains::path_planning::ports::PathPlanningDataSource;
use crate::common::DomainResult;
use crate::common::DomainError;

pub struct FilesystemDataSource {
    base: PathBuf,
}

impl FilesystemDataSource {
    pub fn new(base: Option<PathBuf>) -> Self {
        let base = base.unwrap_or_else(|| {
            if let Ok(v) = env::var("PATH_PLANNING_DATA_DIR") {
                PathBuf::from(v)
            } else {
                let cwd_default = Path::new("resources/path_planning");
                if cwd_default.exists() { cwd_default.to_path_buf() } else { PathBuf::from("/usr/share/gryphon-app/path_planning") }
            }
        });
        Self { base }
    }
}

impl PathPlanningDataSource for FilesystemDataSource {
    fn load_geojson(&self, name: &str) -> DomainResult<String> {
        let mut p = self.base.clone();
        p.push("geojson");
        p.push(name);
        let mut s = String::new();
        let mut f = fs::File::open(&p).map_err(|e| DomainError::InfrastructureError(format!("{}", e)))?;
        f.read_to_string(&mut s).map_err(|e| DomainError::InfrastructureError(format!("{}", e)))?;
        Ok(s)
    }

    fn load_graph_bytes(&self, name: &str) -> DomainResult<Vec<u8>> {
        let mut p = self.base.clone();
        p.push("graphs");
        p.push(name);
        let mut buf = Vec::new();
        let mut f = fs::File::open(&p).map_err(|e| DomainError::InfrastructureError(format!("{}", e)))?;
        f.read_to_end(&mut buf).map_err(|e| DomainError::InfrastructureError(format!("{}", e)))?;
        Ok(buf)
    }
}

impl FilesystemDataSource {
    /// Save graph bytes to the graphs folder (creates directories as needed)
    pub fn save_graph_bytes(&self, name: &str, bytes: &[u8]) -> DomainResult<()> {
        let mut p = self.base.clone();
        p.push("graphs");
        if !p.exists() { fs::create_dir_all(&p).map_err(|e| DomainError::InfrastructureError(format!("{}", e)))?; }
        p.push(name);
        let mut f = fs::File::create(&p).map_err(|e| DomainError::InfrastructureError(format!("{}", e)))?;
        f.write_all(bytes).map_err(|e| DomainError::InfrastructureError(format!("{}", e)))?;
        Ok(())
    }

    /// Build a minimal placeholder graph from a GeoJSON string and return serialized bytes.
    /// In a real implementation this would construct a Petgraph and serialize with bincode.
    pub fn build_graph_from_geojson(&self, geojson: &str) -> DomainResult<Vec<u8>> {
        // placeholder: return the bytes of the geojson as a stand-in for a serialized graph
        Ok(geojson.as_bytes().to_vec())
    }
}
