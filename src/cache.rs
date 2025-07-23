use crate::{Metadata, MetadataEntry, client::Client};
use std::io::ErrorKind;
use tracing::instrument;

const CACHE_PATH: &str = "data/metadata.json";

pub struct Cache {
    client: Client,
    old_metadata: Metadata,
    metadata: Metadata,
}

impl Cache {
    pub fn new() -> Self {
        let client = Client::new();
        let old_metadata = Metadata::default();
        let metadata = Metadata::default();

        Self {
            client,
            old_metadata,
            metadata,
        }
    }

    #[instrument(skip_all)]
    pub async fn load_metadata(&mut self) -> anyhow::Result<()> {
        tracing::info!("loading metadata");

        let res = tokio::fs::read(CACHE_PATH).await;
        let data = res.map_or_else(
            |err| match err.kind() {
                ErrorKind::NotFound => {
                    tracing::warn!(?err, "file not found");
                    Ok(None)
                }
                _ => Err(err),
            },
            |data| Ok(Some(data)),
        )?;

        if let Some(data) = data {
            let metadata: Metadata = serde_json::from_slice(data.as_slice())?;
            self.old_metadata = metadata.clone();
            self.metadata = metadata;
        }

        Ok(())
    }

    #[instrument(skip_all)]
    pub async fn save_metadata(&mut self) -> anyhow::Result<()> {
        if self.metadata != self.old_metadata {
            let raw = serde_json::to_string_pretty(&self.metadata)?;
            tokio::fs::write(CACHE_PATH, raw).await?;
            tracing::info!("saved metadata");
        } else {
            tracing::info!("no modifications needed");
        }
        Ok(())
    }

    #[instrument(skip_all)]
    pub async fn update_state(&mut self) -> anyhow::Result<()> {
        let Some(state) = self.client.get_state(&self.metadata).await? else {
            return Ok(());
        };

        tokio::fs::write("data/state.json", state.content).await?;

        self.metadata
            .state
            .replace(MetadataEntry { etag: state.etag });

        Ok(())
    }

    #[instrument(skip_all)]
    pub async fn update_players(&mut self) -> anyhow::Result<()> {
        let Some(players) = self.client.get_players(&self.metadata).await? else {
            return Ok(());
        };

        tokio::fs::write("data/players.json", players.content).await?;

        self.metadata
            .players
            .replace(MetadataEntry { etag: players.etag });
        Ok(())
    }
}
