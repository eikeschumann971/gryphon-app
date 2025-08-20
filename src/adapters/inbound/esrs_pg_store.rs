#![cfg(feature = "esrs_migration")]

use esrs::store::postgres::PgStoreBuilder;
use esrs::store::postgres::PgStore;
use esrs::bus::EventBus;
use sqlx::postgres::PgPoolOptions;

/// Build a PgStore for a specific event type T and attach an EventBus.
/// The function runs migrations by default via PgStoreBuilder::try_build().
pub async fn build_pg_store_with_bus<A, B>(database_url: &str, bus: B) -> anyhow::Result<PgStore<A>>
where
    A: esrs::Aggregate,
    A::Event: serde::Serialize + for<'de> serde::de::Deserialize<'de> + Send + Sync + 'static + Clone,
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
/// This avoids generating new random UUIDs when the application uses string ids
/// while keeping esrs' UUID-based identity consistent across runs.
pub fn uuid_for_aggregate_id(id: &str) -> uuid::Uuid {
    // Implement a stable name-based UUID (v3) using MD5 so we avoid depending on
    // a specific uuid crate feature. This mirrors the semantics of v3/v5 but is
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
