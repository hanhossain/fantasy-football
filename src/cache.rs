use crate::{Metadata, MetadataEntry, client::Client};
use serde::Deserialize;
use std::io::ErrorKind;
use tracing::instrument;

const RAW_CACHE_DIR: &str = "data/raw";

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

        let res = tokio::fs::read(format!("{RAW_CACHE_DIR}/metadata.json")).await;
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
            tokio::fs::write(format!("{RAW_CACHE_DIR}/metadata.json"), raw).await?;
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

        tokio::fs::write(format!("{RAW_CACHE_DIR}/state.json"), state.content).await?;

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

        tokio::fs::write(format!("{RAW_CACHE_DIR}/players.json"), players.content).await?;

        self.metadata
            .players
            .replace(MetadataEntry { etag: players.etag });
        Ok(())
    }

    #[instrument(skip_all)]
    pub async fn update_schedules(&mut self) -> anyhow::Result<()> {
        // load state
        let content = tokio::fs::read(format!("{RAW_CACHE_DIR}/state.json")).await?;
        let state: SleeperState = serde_json::from_slice(content.as_slice())?;

        let previous_season = self
            .client
            .get_schedule(&self.metadata, "regular", &state.previous_season)
            .await?;
        if let Some(previous_season) = previous_season {
            tokio::fs::write(
                format!(
                    "{RAW_CACHE_DIR}/schedules/regular/{}.json",
                    state.previous_season
                ),
                previous_season.content,
            )
            .await?;

            let schedules = self.metadata.schedules.get_or_insert_default();
            schedules.insert(
                state.previous_season.clone(),
                MetadataEntry {
                    etag: previous_season.etag,
                },
            );
        }

        let current_season = self
            .client
            .get_schedule(&self.metadata, "regular", &state.season)
            .await?;
        if let Some(current_season) = current_season {
            tokio::fs::write(
                format!("{RAW_CACHE_DIR}/schedules/regular/{}.json", state.season),
                current_season.content,
            )
            .await?;

            let schedules = self.metadata.schedules.get_or_insert_default();
            schedules.insert(
                state.season.clone(),
                MetadataEntry {
                    etag: current_season.etag,
                },
            );
        }

        Ok(())
    }
}

#[derive(Deserialize)]
struct SleeperState {
    previous_season: String,
    season: String,
}
