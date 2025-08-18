use std::path::{PathBuf, Path};
use std::env;
use std::fs;
use std::io::{Read, Write};
use crate::domains::path_planning::ports::PathPlanningDataSource;
use crate::common::DomainResult;
use crate::common::DomainError;

use petgraph::graph::{Graph, NodeIndex};
use petgraph::Undirected;
use geojson::GeoJson;
use geojson::Value;
use serde::{Serialize, Deserialize};
use bincode;
use ordered_float::NotNan;
use std::str::FromStr;
use serde_json::json;

// Node coordinate type used in graphs
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Coord(pub f64, pub f64);


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

    /// Build a petgraph structure from GeoJSON and return it.
    pub fn build_graph_struct(&self, geojson: &str) -> DomainResult<Graph<Coord, (), Undirected>> {
        // reuse the parsing logic in build_graph_from_geojson
        self.build_graph_from_geojson(geojson)
    }

    /// Save a petgraph to disk with a small header (magic + version + metadata length + metadata JSON + payload).
    pub fn save_graph(&self, name: &str, graph: &Graph<Coord, (), Undirected>) -> DomainResult<()> {
        // serialize graph with bincode
        let payload = bincode::serialize(graph).map_err(|e| DomainError::InfrastructureError(format!("bincode serialize error: {}", e)))?;
        // header
        let header = json!({"format":"petgraph-bincode","version":1});
        let header_bytes = serde_json::to_vec(&header).map_err(|e| DomainError::InfrastructureError(format!("header json error: {}", e)))?;

        let mut p = self.base.clone();
        p.push("graphs");
        if !p.exists() { fs::create_dir_all(&p).map_err(|e| DomainError::InfrastructureError(format!("{}", e)))?; }
        p.push(name);

        let mut f = fs::File::create(&p).map_err(|e| DomainError::InfrastructureError(format!("{}", e)))?;
        // write magic
        f.write_all(b"PGPH").map_err(|e| DomainError::InfrastructureError(format!("{}", e)))?;
        // write version (u8)
        f.write_all(&[1u8]).map_err(|e| DomainError::InfrastructureError(format!("{}", e)))?;
        // write header length (u32 LE)
        let hl = (header_bytes.len() as u32).to_le_bytes();
        f.write_all(&hl).map_err(|e| DomainError::InfrastructureError(format!("{}", e)))?;
        // write header
        f.write_all(&header_bytes).map_err(|e| DomainError::InfrastructureError(format!("{}", e)))?;
        // write payload
        f.write_all(&payload).map_err(|e| DomainError::InfrastructureError(format!("{}", e)))?;
        Ok(())
    }

    /// Load a petgraph previously saved with `save_graph`.
    pub fn load_graph(&self, name: &str) -> DomainResult<Graph<Coord, (), Undirected>> {
        let mut p = self.base.clone();
        p.push("graphs");
        p.push(name);
        let mut f = fs::File::open(&p).map_err(|e| DomainError::InfrastructureError(format!("{}", e)))?;
        // read magic
        let mut magic = [0u8;4];
        f.read_exact(&mut magic).map_err(|e| DomainError::InfrastructureError(format!("{}", e)))?;
        if &magic != b"PGPH" { return Err(DomainError::InfrastructureError("invalid graph file magic".to_string())); }
        // version
        let mut ver = [0u8;1];
        f.read_exact(&mut ver).map_err(|e| DomainError::InfrastructureError(format!("{}", e)))?;
        let mut hl = [0u8;4];
        f.read_exact(&mut hl).map_err(|e| DomainError::InfrastructureError(format!("{}", e)))?;
        let header_len = u32::from_le_bytes(hl) as usize;
        let mut header_bytes = vec![0u8; header_len];
        f.read_exact(&mut header_bytes).map_err(|e| DomainError::InfrastructureError(format!("{}", e)))?;
        // rest is payload
        let mut payload = Vec::new();
        f.read_to_end(&mut payload).map_err(|e| DomainError::InfrastructureError(format!("{}", e)))?;

        match ver[0] {
            0 => {
                // legacy format: payload is direct bincode of Graph<Coord,..>
                let graph: Graph<Coord, (), Undirected> = bincode::deserialize(&payload).map_err(|e| DomainError::InfrastructureError(format!("bincode deserialize error (v0): {}", e)))?;
                Ok(graph)
            }
            1 => {
                // current format: payload is bincode of Graph as well
                let graph: Graph<Coord, (), Undirected> = bincode::deserialize(&payload).map_err(|e| DomainError::InfrastructureError(format!("bincode deserialize error (v1): {}", e)))?;
                Ok(graph)
            }
            other => Err(DomainError::InfrastructureError(format!("unsupported graph file version: {}", other))),
        }
    }

    /// Build a minimal placeholder graph from a GeoJSON string and return serialized bytes.
    /// In a real implementation this would construct a Petgraph and serialize with bincode.
    pub fn build_graph_from_geojson(&self, geojson: &str) -> DomainResult<Graph<Coord, (), Undirected>> {
        // Parse the GeoJSON and build a simple undirected graph.
        // For this implementation we support FeatureCollections with LineString or MultiLineString features.
        let gj = GeoJson::from_str(geojson).map_err(|e| DomainError::InfrastructureError(format!("geojson parse error: {}", e)))?;

        // We'll construct a petgraph::Graph<Coord, ()>

        let mut graph: Graph<Coord, (), Undirected> = Graph::new_undirected();
        let mut coord_index_map: std::collections::HashMap<(NotNan<f64>,NotNan<f64>), NodeIndex> = std::collections::HashMap::new();

        if let GeoJson::FeatureCollection(fc) = gj {
            for feature in fc.features.iter() {
                if let Some(geom) = &feature.geometry {
                    match &geom.value {
                        Value::LineString(lines) => {
                            let mut prev: Option<NodeIndex> = None;
                            for c in lines.iter() {
                                if c.len() < 2 { continue; }
                                let x = c[0]; let y = c[1];
                                let nx = NotNan::new(x).map_err(|e| DomainError::InfrastructureError(format!("non-finite coord: {}", e)))?;
                                let ny = NotNan::new(y).map_err(|e| DomainError::InfrastructureError(format!("non-finite coord: {}", e)))?;
                                let key = (nx,ny);
                                let idx = if let Some(&i) = coord_index_map.get(&key) {
                                    i
                                } else {
                                    let ni = graph.add_node(Coord(x,y));
                                    coord_index_map.insert(key, ni);
                                    ni
                                };
                                if let Some(p) = prev { if p != idx { graph.update_edge(p, idx, ()); } }
                                prev = Some(idx);
                            }
                        },
                        Value::MultiLineString(lines_multi) => {
                            for lines in lines_multi.iter() {
                                let mut prev: Option<NodeIndex> = None;
                                for c in lines.iter() {
                                    if c.len() < 2 { continue; }
                                    let x = c[0]; let y = c[1];
                                    let nx = NotNan::new(x).map_err(|e| DomainError::InfrastructureError(format!("non-finite coord: {}", e)))?;
                                    let ny = NotNan::new(y).map_err(|e| DomainError::InfrastructureError(format!("non-finite coord: {}", e)))?;
                                    let key = (nx,ny);
                                    let idx = if let Some(&i) = coord_index_map.get(&key) {
                                        i
                                    } else {
                                        let ni = graph.add_node(Coord(x,y));
                                        coord_index_map.insert(key, ni);
                                        ni
                                    };
                                    if let Some(p) = prev { if p != idx { graph.update_edge(p, idx, ()); } }
                                    prev = Some(idx);
                                }
                            }
                        },
                        // For points we add isolated nodes
                        Value::Point(p) => {
                            if p.len() >= 2 {
                                let x = p[0]; let y = p[1];
                                let nx = NotNan::new(x).map_err(|e| DomainError::InfrastructureError(format!("non-finite coord: {}", e)))?;
                                let ny = NotNan::new(y).map_err(|e| DomainError::InfrastructureError(format!("non-finite coord: {}", e)))?;
                                let key = (nx,ny);
                                // use entry API to avoid contains_key followed by insert
                                let _ = *coord_index_map.entry(key).or_insert_with(|| graph.add_node(Coord(x,y)));
                            }
                        }
                        _ => { /* skip unsupported geometries for now */ }
                    }
                }
            }
        } else {
            return Err(DomainError::InfrastructureError("expected FeatureCollection GeoJSON".to_string()));
        }

    Ok(graph)
    }
}
