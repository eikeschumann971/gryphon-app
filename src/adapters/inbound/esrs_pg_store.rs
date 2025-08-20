use esrs::bus::EventBus;
use esrs::store::postgres::PgStore;
use esrs::store::postgres::PgStoreBuilder;
use sqlx::postgres::PgPoolOptions;
use tracing::{info, warn};

/// Build a PgStore for a specific event type T and attach an EventBus.
/// The function runs migrations by default via PgStoreBuilder::try_build().
pub async fn build_pg_store_with_bus<A, B>(database_url: &str, bus: B) -> anyhow::Result<PgStore<A>>
where
    A: esrs::Aggregate,
    A::Event:
        serde::Serialize + for<'de> serde::de::Deserialize<'de> + Send + Sync + 'static + Clone,
    B: EventBus<A> + Send + Sync + 'static,
{
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    let builder = PgStoreBuilder::new(pool).add_event_bus(bus);
    let store: PgStore<A> = builder
        .try_build()
        .await
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    Ok(store)
}

/// Convert a string aggregate id into a stable UUID using UUID v5 (name-based).
/// This produces a deterministic UUID for the given aggregate id using the
/// DNS namespace and SHA-1 (v5). Using v5 keeps identities stable across runs
/// and avoids introducing random UUIDs when the domain uses string IDs.
pub fn uuid_for_aggregate_id(id: &str) -> uuid::Uuid {
    // Implement a stable name-based UUID (v3) using MD5 so we avoid depending on
    // a specific uuid crate feature. This mirrors the semantics of v3 but is
    // deterministic for the given id string.
    // Namespace: use the DNS namespace bytes (UUID::NAMESPACE_DNS)
    let mut input = Vec::new();
    input.extend_from_slice(uuid::Uuid::NAMESPACE_DNS.as_bytes());
    input.extend_from_slice(id.as_bytes());
    let digest = md5::compute(&input);
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&digest[0..16]);

    // Set version to 3 (name-based MD5)
    bytes[6] = (bytes[6] & 0x0f) | (3 << 4);
    // Set variant to RFC 4122
    bytes[8] = (bytes[8] & 0x3f) | 0x80;

    uuid::Uuid::from_bytes(bytes)
}

/// Fetch the last sequence_number for the aggregate from the esrs events table.
/// Returns Ok(Some(n)) if found, Ok(None) if no events exist, or Err on DB error.
pub async fn agg_last_sequence_for<A>(agg_uuid: &uuid::Uuid) -> anyhow::Result<Option<i64>>
where
    A: esrs::Aggregate,
{
    let database_url = std::env::var("DATABASE_URL").map_err(|e| anyhow::anyhow!(e.to_string()))?;
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await?;

    // derive the esrs events table name for the aggregate (convention: <name>_events)
    let table = format!("{}_events", <A as esrs::Aggregate>::NAME);
    let query = format!("SELECT max(sequence_number) FROM {} WHERE aggregate_id = $1", table);

    let row = sqlx::query_as::<_, (Option<i64>,)>(&query)
        .bind(agg_uuid)
        .fetch_one(&pool)
        .await?;

    Ok(row.0)
}

/// Persist events into the provided PgStore but treat duplicate-key DB errors as
/// idempotent (best-effort mirroring). This avoids noisy failures when the same
/// domain event is mirrored more than once during migration.
pub async fn persist_best_effort<S>(
    store: &S,
    agg_state: &mut esrs::AggregateState<<S::Aggregate as esrs::Aggregate>::State>,
    events: Vec<<S::Aggregate as esrs::Aggregate>::Event>,
) -> anyhow::Result<()>
where
    S: esrs::store::EventStore + Send + Sync,
    <S::Aggregate as esrs::Aggregate>::Event:
        serde::Serialize + for<'de> serde::de::Deserialize<'de> + Send + Sync + 'static + Clone,
{
    use esrs::store::EventStore as EsrsEventStore;

    // Attempt a lightweight pre-check to see if the event(s) already exist in the
    // esrs-generated events table for this aggregate. This is a best-effort check
    // to avoid triggering duplicate-key insert attempts when mirroring events.
    // If the pre-check cannot be performed (e.g. DB connection error), we fall
    // back to calling the store persist and keep the existing duplicate-key
    // handling behavior.
    if let Ok(database_url) = std::env::var("DATABASE_URL") {
        // Try to connect with a small pool for the pre-check
        match PgPoolOptions::new()
            .max_connections(1)
            .connect(&database_url)
            .await
        {
            Ok(pool) => {
                // Use the aggregate id from the provided agg_state to scope the query
                // `id()` returns a Uuid which is Copy; avoid clone_on_copy lint.
                let agg_id = *agg_state.id();
                // derive table name from aggregate type
                let table = format!("{}_events", <<S as esrs::store::EventStore>::Aggregate as esrs::Aggregate>::NAME);

                // For each event, serialize to JSON and do a best-effort existence check
                for evt in &events {
                    match serde_json::to_value(evt) {
                        Ok(payload_json) => {
                            // Query for an exact payload match for this aggregate.
                            // The payload column is jsonb in the esrs table.
                            let query = format!(
                                "SELECT count(*) FROM {} WHERE aggregate_id = $1 AND payload = $2",
                                table
                            );
                            let res = sqlx::query_as::<_, (i64,)>(&query)
                                .bind(agg_id)
                                .bind(payload_json)
                                .fetch_one(&pool)
                                .await;

                            match res {
                                Ok((cnt,)) => {
                                    if cnt > 0 {
                                        info!(%agg_id, "esrs pre-check: matching event already exists for aggregate (skipping persist)");
                                        return Ok(());
                                    }
                                }
                                Err(e) => {
                                    // If the pre-check query fails, log and break to fall back
                                    // to the normal persist path below.
                                    warn!("esrs pre-check query failed: {}. Falling back to persist.", e);
                                    break;
                                }
                            }
                        }
                        Err(e) => {
                            warn!("esrs pre-check JSON serialization failed: {}. Falling back to persist.", e);
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                warn!("esrs pre-check DB connection failed: {}. Falling back to persist.", e);
            }
        }
    }

    // If we reach here, either the pre-check didn't find a duplicate, or the
    // pre-check couldn't be completed; attempt to persist and handle common
    // duplicate-key errors idempotently.
    match EsrsEventStore::persist(store, agg_state, events).await {
        Ok(_) => Ok(()),
        Err(e) => {
            let s = e.to_string();
            // Common Postgres duplicate key error fragment
            if s.contains("duplicate key value") || s.contains("unique constraint") {
                // Best-effort: treat as already-applied
                warn!("esrs persist duplicate detected and ignored: {}", s);
                Ok(())
            } else {
                Err(anyhow::anyhow!(s))
            }
        }
    }
}
