use std::path::{PathBuf, Path};
use std::env;
use std::fs;
use serde::{Serialize, Deserialize};
use std::io::Read;

use crate::domains::path_planning::types::Position2D;
use crate::domains::path_planning::events::PathPlanningEvent;

#[derive(Debug)]
pub enum LoaderError {
    Io(std::io::Error),
    Parse(String),
}

impl From<std::io::Error> for LoaderError {
    fn from(e: std::io::Error) -> Self { LoaderError::Io(e) }
}

/// Resolve the path_planning data directory.
/// Precedence: PATH_PLANNING_DATA_DIR env var -> ./resources/path_planning -> /usr/share/gryphon-app/path_planning
pub fn resolve_data_dir() -> PathBuf {
    if let Ok(v) = env::var("PATH_PLANNING_DATA_DIR") {
        return PathBuf::from(v);
    }
    let cwd_default = Path::new("resources/path_planning");
    if cwd_default.exists() { return cwd_default.to_path_buf(); }
    PathBuf::from("/usr/share/gryphon-app/path_planning")
}

/// Load a GeoJSON file by filename (relative to resolved data dir). Returns file contents as string.
pub fn load_geojson(name: &str) -> Result<String, LoaderError> {
    let mut p = resolve_data_dir();
    p.push("geojson");
    p.push(name);
    let mut s = String::new();
    let mut f = fs::File::open(&p)?;
    f.read_to_string(&mut s)?;
    Ok(s)
}

// ...future: load serialized graphs, cache logic, builder
