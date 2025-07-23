use serde::Deserialize;
use tracing::{info, instrument};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    print_state().await;
}

#[instrument]
async fn print_state() {
    let state = reqwest::get("https://api.sleeper.app/v1/state/nfl")
        .await
        .unwrap()
        .json::<NflState>()
        .await
        .unwrap();
    info!(?state, "Got nfl state");
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct NflState {
    week: u8,
    leg: u8,
    season: String,
    season_type: String,
    league_season: String,
    previous_season: String,
    season_start_date: String,
    display_week: u8,
    league_create_season: String,
    season_has_scores: bool,
}
