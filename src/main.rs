mod client;

use crate::client::Client;
use serde::{Deserialize, Serialize};
use std::io::ErrorKind;
use tracing::{instrument, level_filters::LevelFilter};
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

    let mut metadata = load_metadata().await?.unwrap_or_default();
    let mut is_modified = false;

    let client = Client::new();

    if let Some(state) = client.get_state(&metadata).await? {
        metadata.state = Some(state);
        is_modified = true;
    }

    if let Some(players) = client.get_players(&metadata).await? {
        metadata.players = Some(players);
        is_modified = true;
    }

    save_metadata(&metadata, is_modified).await?;

    Ok(())
}

#[instrument]
async fn load_metadata() -> anyhow::Result<Option<Metadata>> {
    tracing::info!("loading metadata");
    let res = tokio::fs::read("metadata.json").await;
    match res {
        Ok(data) => {
            let metadata = serde_json::from_slice(data.as_slice())?;
            Ok(metadata)
        }
        Err(err) => {
            if err.kind() == ErrorKind::NotFound {
                tracing::warn!(?err, "file not found");
                Ok(None)
            } else {
                Err(err.into())
            }
        }
    }
}

#[instrument(skip_all)]
async fn save_metadata(metadata: &Metadata, is_modified: bool) -> anyhow::Result<()> {
    if is_modified {
        let raw = serde_json::to_string_pretty(metadata)?;
        tokio::fs::write("metadata.json", raw).await?;
        tracing::info!("saved metadata");
    } else {
        tracing::info!("no modifications needed");
    }
    Ok(())
}

#[derive(Clone, Deserialize, Serialize)]
struct MetadataEntry {
    etag: Option<String>,
    content: String,
}

#[derive(Clone, Deserialize, Serialize, Default)]
struct Metadata {
    state: Option<MetadataEntry>,
    players: Option<MetadataEntry>,
}
