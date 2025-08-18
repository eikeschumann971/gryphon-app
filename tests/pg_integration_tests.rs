#[cfg(feature = "pg_integration")]
use gryphon_app::adapters::outbound::postgres_graph_store::PostgresGraphStore;
#[cfg(feature = "pg_integration")]
use gryphon_app::adapters::outbound::path_planning_data::FilesystemDataSource;
#[cfg(feature = "pg_integration")]
use deadpool_postgres::Config as DeadPoolConfig;
#[cfg(feature = "pg_integration")]
use tokio_postgres::NoTls;
#[cfg(feature = "use_testcontainers")]
use testcontainers::clients::Cli;
#[cfg(feature = "use_testcontainers")]
use testcontainers_modules::postgres::Postgres;

#[cfg(feature = "pg_integration")]
#[tokio::test]
async fn test_postgres_graph_store_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
    // Determine Postgres connection parameters. By default the test expects an external
    // Postgres instance (host/port) to be provided via environment variables. If the
    // optional dev feature `use_testcontainers` is enabled the test will start a
    // container in-process using the `testcontainers` crate.

    let (host, port, user, password, db) = {
        #[cfg(feature = "use_testcontainers")]
        {
            // Start a Postgres container using testcontainers (self-contained)
            let docker = Cli::default();
            let node = docker.run(Postgres::default());
            let port = node.get_host_port_ipv4(5432);
            ("127.0.0.1".to_string(), port, "postgres".to_string(), "postgres".to_string(), "postgres".to_string())
        }
        #[cfg(not(feature = "use_testcontainers"))]
        {
            // Use environment variables set by the helper script or CI. Defaults to localhost:5432
            let host = std::env::var("PG_TEST_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
            let port = std::env::var("PG_TEST_PORT").ok().and_then(|s| s.parse::<u16>().ok()).unwrap_or(5432);
            let user = std::env::var("PG_TEST_USER").unwrap_or_else(|_| "postgres".to_string());
            let password = std::env::var("PG_TEST_PASSWORD").unwrap_or_else(|_| "postgres".to_string());
            let db = std::env::var("PG_TEST_DB").unwrap_or_else(|_| "postgres".to_string());
            (host, port, user, password, db)
        }
    };

    let conn_str = format!("host={} port={} user={} password={} dbname={}", host, port, user, password, db);

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
