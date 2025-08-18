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
    fs::write(&geo_file, "{ \"type\": \"FeatureCollection\" }").unwrap();

    let geo = ds.load_geojson("sample.geojson").unwrap();
    assert!(geo.contains("FeatureCollection"));

    // build graph bytes and save
    let graph_bytes = ds.build_graph_from_geojson(&geo).unwrap();
    ds.save_graph_bytes("sample.graph.bin", &graph_bytes).unwrap();

    // load back
    let loaded = ds.load_graph_bytes("sample.graph.bin").unwrap();
    assert_eq!(loaded, graph_bytes);
}
