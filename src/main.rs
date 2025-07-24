mod cache;
mod client;

use crate::cache::Cache;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    let mut cache = Cache::new();
    cache.load_metadata().await?;

    cache.update_state().await?;
    cache.update_players().await?;
    cache.update_schedules().await?;

    cache.save_metadata().await?;

    Ok(())
}

#[derive(Clone, Deserialize, Serialize, PartialEq)]
struct MetadataEntry {
    etag: Option<String>,
}

#[derive(Clone, Deserialize, Serialize, Default, PartialEq)]
struct Metadata {
    state: Option<MetadataEntry>,
    players: Option<MetadataEntry>,
    schedules: Option<HashMap<String, MetadataEntry>>,
}
