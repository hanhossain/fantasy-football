use reqwest::header::ETAG;
use serde::{Deserialize, Serialize};
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

    let state = get_state().await?;
    let metadata = Metadata { state };

    let raw = serde_json::to_string_pretty(&metadata)?;

    tokio::fs::write("metadata.json", raw).await?;
    Ok(())
}

#[instrument]
async fn get_state() -> anyhow::Result<MetadataEntry> {
    let response = reqwest::get("https://api.sleeper.app/v1/state/nfl")
        .await?
        .error_for_status()?;
    let etag = response
        .headers()
        .get(ETAG)
        .map(|e| e.to_str().map(|x| x.to_owned()))
        .transpose()?;

    if etag.is_none() {
        tracing::warn!("missing etag in state");
    }

    match &etag {
        Some(etag) => tracing::info!(etag),
        None => tracing::warn!("missing etag"),
    }

    let content = response.text().await?;
    Ok(MetadataEntry { etag, content })
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct MetadataEntry {
    etag: Option<String>,
    content: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Metadata {
    state: MetadataEntry,
}
