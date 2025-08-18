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
        // Parse the GeoJSON and build a simple undirected graph.
        // For this implementation we support FeatureCollections with LineString or MultiLineString features.
    let gj = GeoJson::from_str(geojson).map_err(|e| DomainError::InfrastructureError(format!("geojson parse error: {}", e)))?;

    // We'll store node coordinates as (f64, f64)
    #[derive(Serialize, Deserialize, Clone)]
    struct Coord(f64, f64);

        #[derive(Serialize, Deserialize)]
        struct SerializableGraph {
            nodes: Vec<Coord>,
            edges: Vec<(usize, usize)>,
            directed: bool,
        }

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
                                if !coord_index_map.contains_key(&key) { let ni = graph.add_node(Coord(x,y)); coord_index_map.insert(key, ni); }
                            }
                        }
                        _ => { /* skip unsupported geometries for now */ }
                    }
                }
            }
        } else {
            return Err(DomainError::InfrastructureError("expected FeatureCollection GeoJSON".to_string()));
        }

        // Convert graph into serializable structure
        let mut nodes: Vec<Coord> = Vec::new();
        let mut index_to_pos: std::collections::HashMap<NodeIndex, usize> = std::collections::HashMap::new();
        for (i, n) in graph.node_indices().enumerate() {
            let coord = graph[n].clone();
            index_to_pos.insert(n, i);
            nodes.push(coord);
        }
        let mut edges: Vec<(usize,usize)> = Vec::new();
        for e in graph.edge_indices() {
            let (a,b) = graph.edge_endpoints(e).unwrap();
            let pa = index_to_pos[&a];
            let pb = index_to_pos[&b];
            edges.push((pa,pb));
        }

        let sgraph = SerializableGraph { nodes, edges, directed: false };
        let bytes = bincode::serialize(&sgraph).map_err(|e| DomainError::InfrastructureError(format!("bincode serialize error: {}", e)))?;
        Ok(bytes)
    }
}
