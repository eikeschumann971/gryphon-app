use gryphon_app::adapters::outbound::path_planning_data::{Coord, FilesystemDataSource};
use gryphon_app::PathPlanningDataSource;
use std::fs;
use std::io::Write;
use tempfile::tempdir;

#[test]
fn test_build_graph_from_point_feature() {
    let dir = tempdir().unwrap();
    let base = dir.path().to_path_buf();
    let ds = FilesystemDataSource::new(Some(base.clone()));

    // create geojson folder and file
    let mut geo_dir = base.clone();
    geo_dir.push("geojson");
    fs::create_dir_all(&geo_dir).unwrap();
    let mut geo_file = geo_dir.clone();
    geo_file.push("points.geojson");
    fs::write(&geo_file, r#"{
  "type": "FeatureCollection",
  "features": [
    { "type": "Feature", "properties": {"name": "A"}, "geometry": { "type": "Point", "coordinates": [0.0, 0.0] } },
    { "type": "Feature", "properties": {"name": "B"}, "geometry": { "type": "Point", "coordinates": [10.0, 0.0] } }
  ]
}"#).unwrap();

    let geo = ds.load_geojson("points.geojson").unwrap();
    let graph = ds.build_graph_struct(&geo).unwrap();
    assert!(graph.node_count() > 0);
}

#[test]
fn test_build_graph_from_linestring() {
    let dir = tempdir().unwrap();
    let base = dir.path().to_path_buf();
    let ds = FilesystemDataSource::new(Some(base.clone()));

    // create geojson folder and file
    let mut geo_dir = base.clone();
    geo_dir.push("geojson");
    fs::create_dir_all(&geo_dir).unwrap();
    let mut geo_file = geo_dir.clone();
    geo_file.push("lines.geojson");
    fs::write(&geo_file, r#"{
  "type": "FeatureCollection",
  "features": [
    { "type": "Feature", "properties": {"name": "L"}, "geometry": { "type": "LineString", "coordinates": [[0.0,0.0],[5.0,0.0],[10.0,0.0]] } }
  ]
}"#).unwrap();

    let geo = ds.load_geojson("lines.geojson").unwrap();
    let graph = ds.build_graph_struct(&geo).unwrap();
    assert!(graph.node_count() > 0);
}

#[test]
fn test_missing_geojson_file_returns_error() {
    let dir = tempdir().unwrap();
    let base = dir.path().to_path_buf();
    let ds = FilesystemDataSource::new(Some(base.clone()));

    let res = ds.load_geojson("does_not_exist.geojson");
    assert!(res.is_err());
}

#[test]
fn test_save_and_load_graph_roundtrip() {
    let dir = tempdir().unwrap();
    let base = dir.path().to_path_buf();
    let ds = FilesystemDataSource::new(Some(base.clone()));

    // create geojson folder and file
    let mut geo_dir = base.clone();
    geo_dir.push("geojson");
    fs::create_dir_all(&geo_dir).unwrap();
    let mut geo_file = geo_dir.clone();
    geo_file.push("lines.geojson");
    fs::write(&geo_file, r#"{
  "type": "FeatureCollection",
  "features": [
    { "type": "Feature", "properties": {"name": "L"}, "geometry": { "type": "LineString", "coordinates": [[0.0,0.0],[5.0,0.0],[10.0,0.0]] } }
  ]
}"#).unwrap();

    let geo = ds.load_geojson("lines.geojson").unwrap();
    let graph = ds.build_graph_struct(&geo).unwrap();
    ds.save_graph("roundtrip.graph", &graph).unwrap();
    let loaded = ds.load_graph("roundtrip.graph").unwrap();
    assert_eq!(graph.node_count(), loaded.node_count());
    assert_eq!(graph.edge_count(), loaded.edge_count());
}

#[test]
fn test_load_legacy_v0_graph_file_migrates() {
    let dir = tempdir().unwrap();
    let base = dir.path().to_path_buf();
    let ds = FilesystemDataSource::new(Some(base.clone()));

    // create a small graph payload and write a v0 file manually
    use petgraph::graph::Graph;
    use petgraph::Undirected;
    let mut graph: Graph<Coord, (), Undirected> = Graph::new_undirected();
    let _n1 = graph.add_node(Coord(0.0, 0.0));
    let _n2 = graph.add_node(Coord(1.0, 0.0));
    let payload = bincode::serialize(&graph).unwrap();

    // write file: magic + version(0) + header_len(0) + payload
    let mut p = base.clone();
    p.push("graphs");
    fs::create_dir_all(&p).unwrap();
    p.push("legacy_v0.graph");
    let mut f = fs::File::create(&p).unwrap();
    f.write_all(b"PGPH").unwrap();
    f.write_all(&[0u8]).unwrap();
    f.write_all(&0u32.to_le_bytes()).unwrap();
    f.write_all(&payload).unwrap();
    drop(f);

    let loaded = ds.load_graph("legacy_v0.graph").unwrap();
    assert_eq!(loaded.node_count(), 2);
}
