use crate::{Metadata, MetadataEntry};
use reqwest::{
    StatusCode, Url,
    header::{ETAG, HeaderMap, IF_NONE_MATCH},
};
use tracing::instrument;

pub struct Client {
    client: reqwest::Client,
    base_url: Url,
}

impl Client {
    pub fn new() -> Self {
        let base_url = Url::parse("https://api.sleeper.com/").unwrap();
        let client = reqwest::Client::new();
        Self { client, base_url }
    }

    #[instrument(skip(self))]
    pub async fn get(
        &self,
        path: &str,
        prev_etag: &Option<&String>,
    ) -> anyhow::Result<Option<SleeperEntry>> {
        tracing::debug!("starting request");

        let url = self.base_url.join(path)?;

        let mut headers = HeaderMap::new();
        if let Some(prev_etag) = prev_etag {
            headers.insert(IF_NONE_MATCH, prev_etag.parse()?);
        }

        let response = self
            .client
            .get(url)
            .headers(headers)
            .send()
            .await?
            .error_for_status()?;

        if response.status() == StatusCode::NOT_MODIFIED {
            tracing::debug!("not modified");
            return Ok(None);
        }

        let etag = response
            .headers()
            .get(ETAG)
            .map(|etag| etag.to_str().map(|s| s.to_owned()))
            .transpose()?;

        match &etag {
            Some(etag) => tracing::info!(etag),
            None => tracing::warn!("missing etag"),
        }

        let content = response.text().await?;
        tracing::debug!("finished request");

        Ok(Some(SleeperEntry { etag, content }))
    }

    #[instrument(skip_all)]
    pub async fn get_state(&self, metadata: &Metadata) -> anyhow::Result<Option<SleeperEntry>> {
        let etag = match &metadata.state {
            Some(MetadataEntry { etag, .. }) => etag.as_ref(),
            None => None,
        };
        self.get("/v1/state/nfl", &etag).await
    }

    #[instrument(skip_all)]
    pub async fn get_players(&self, metadata: &Metadata) -> anyhow::Result<Option<SleeperEntry>> {
        let etag = match &metadata.players {
            Some(MetadataEntry { etag, .. }) => etag.as_ref(),
            None => None,
        };
        self.get("/v1/players/nfl", &etag).await
    }
}

#[derive(Debug)]
pub struct SleeperEntry {
    pub etag: Option<String>,
    pub content: String,
}
