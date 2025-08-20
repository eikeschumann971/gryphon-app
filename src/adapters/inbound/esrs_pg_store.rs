#![cfg(feature = "esrs_migration")]

use esrs::pg::PgStoreBuilder;
use esrs::store::PgStore;
use sqlx::postgres::PgPoolOptions;
use sqlx::Pool;
use sqlx::Postgres;
use std::sync::Arc;

use crate::domains::path_planning::path_planning::PathPlanningEvent; // placeholder

pub async fn build_pg_store(database_url: &str) -> anyhow::Result<PgStore<PathPlanningEvent>> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    // Build the store with default migrations
    let store: PgStore<PathPlanningEvent> = PgStoreBuilder::new(pool)
        .try_build()
        .await
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    Ok(store)
}
