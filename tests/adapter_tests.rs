use gryphon_app::adapters::outbound::path_planning_data::FilesystemDataSource;
use gryphon_app::PathPlanningDataSource;
use tempfile::tempdir;
use std::fs;

#[test]
fn test_filesystem_datasource_save_and_load() {
    let dir = tempdir().unwrap();
    let base = dir.path().to_path_buf();
    let ds = FilesystemDataSource::new(Some(base.clone()));

    // create geojson folder and file
    let mut geo_dir = base.clone();
    geo_dir.push("geojson");
    fs::create_dir_all(&geo_dir).unwrap();
    let mut geo_file = geo_dir.clone();
    geo_file.push("sample.geojson");
        fs::write(&geo_file, r#"{
    "type": "FeatureCollection",
    "features": [
        { "type": "Feature", "properties": {"name": "A"}, "geometry": { "type": "Point", "coordinates": [0.0, 0.0] } },
        { "type": "Feature", "properties": {"name": "B"}, "geometry": { "type": "Point", "coordinates": [10.0, 0.0] } }
    ]
}"#).unwrap();

    let geo = ds.load_geojson("sample.geojson").unwrap();
    assert!(geo.contains("FeatureCollection"));

    // build graph struct and save using new graph API
    let graph = ds.build_graph_struct(&geo).unwrap();
    ds.save_graph("sample.graph.bin", &graph).unwrap();

    // load back
    let loaded = ds.load_graph("sample.graph.bin").unwrap();
    assert_eq!(graph.node_count(), loaded.node_count());
    assert_eq!(graph.edge_count(), loaded.edge_count());
}
