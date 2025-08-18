#[cfg(feature = "pg_integration")]
use gryphon_app::adapters::outbound::postgres_graph_store::PostgresGraphStore;
#[cfg(feature = "pg_integration")]
use gryphon_app::adapters::outbound::path_planning_data::FilesystemDataSource;
#[cfg(feature = "pg_integration")]
use deadpool_postgres::Config as DeadPoolConfig;
#[cfg(feature = "pg_integration")]
use tokio_postgres::NoTls;

#[cfg(feature = "pg_integration")]
#[tokio::test]
async fn test_postgres_graph_store_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
    // This test expects a local Postgres instance reachable at 127.0.0.1:<port>.
    // You can start one with Docker before running the test:
    // docker run --rm -e POSTGRES_PASSWORD=postgres -p 5433:5432 -d postgres:15
    let port = std::env::var("PG_TEST_PORT").ok().and_then(|s| s.parse::<u16>().ok()).unwrap_or(5433u16);
    let host = "127.0.0.1";
    let user = "postgres";
    let password = "postgres";
    let db = "postgres";

    let conn_str = format!("host={} port={} user={} password={} dbname={}", host, port, user, password, db);

    // build a deadpool pool and pass it to the PostgresGraphStore
    let mut dp_cfg = DeadPoolConfig::new();
    dp_cfg.host = Some(host.to_string());
    dp_cfg.port = Some(port);
    dp_cfg.user = Some(user.to_string());
    dp_cfg.password = Some(password.to_string());
    dp_cfg.dbname = Some(db.to_string());

    let pool = dp_cfg.create_pool(None, NoTls).expect("failed to create test pg pool");
    let pg = PostgresGraphStore::new(pool);

    // Wait for Postgres to accept connections (simple retry loop)
    let mut connected = false;
    for _ in 0..10 {
        match tokio_postgres::connect(&conn_str, tokio_postgres::NoTls).await {
            Ok((client, connection)) => {
                tokio::spawn(async move { let _ = connection.await; });
                let _ = client.simple_query("SELECT 1").await;
                connected = true;
                break;
            }
            Err(_) => {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            }
        }
    }
    if !connected { return Err("Postgres did not become ready".into()); }

    // Build a tiny graph using FilesystemDataSource builder
    let fs = FilesystemDataSource::new(None);
    let gj = r#"{"type":"FeatureCollection","features":[{"type":"Feature","geometry":{"type":"Point","coordinates":[1.0,2.0]}}]}"#;
    let graph = fs.build_graph_struct(gj).map_err(|e| format!("build graph err: {:?}", e))?;

    // Serialize into bytes using the filesystem save format (with header)
    let mut bytes = Vec::new();
    // emulate FilesystemDataSource::save_graph to produce header+payload
    let payload = bincode::serialize(&graph)?;
    let header = serde_json::to_vec(&serde_json::json!({"format":"petgraph-bincode","version":1}))?;
    bytes.extend_from_slice(b"PGPH");
    bytes.push(1u8);
    let hl = (header.len() as u32).to_le_bytes();
    bytes.extend_from_slice(&hl);
    bytes.extend_from_slice(&header);
    bytes.extend_from_slice(&payload);

    // bring trait into scope so methods are available
    use gryphon_app::domains::path_planning::ports::GraphStoreAsync as _;

    // save via PostgresGraphStore
    pg.save_graph_bytes("test_graph.bin", &bytes).await.map_err(|e| format!("pg save err: {:?}", e))?;

    // load and compare
    let loaded = pg.load_graph_bytes("test_graph.bin").await.map_err(|e| format!("pg load err: {:?}", e))?;
    assert_eq!(loaded, bytes);

    Ok(())
}
