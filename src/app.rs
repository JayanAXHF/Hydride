use std::{
    env,
    path::PathBuf,
    sync::{Arc, OnceLock},
    time::Instant,
};

use anyhow::{Context, anyhow};
use tracing_subscriber::{EnvFilter, fmt};

use crate::{bot, config::BootstrapConfig, db::Database, state::AppState};

pub static START_TIMESTAMP: OnceLock<Instant> = OnceLock::new();

pub async fn run() -> anyhow::Result<()> {
    START_TIMESTAMP
        .set(Instant::now())
        .map_err(|_| anyhow!("Unable to set start time. This should not be possible"))?;
    let config_path = env::var("MODBOT_CONFIG")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("config.toml"));
    let banlist_path = env::var("MODBOT_UUID_BANLIST")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("banlist.toml"));
    let config = Arc::new(BootstrapConfig::load(&config_path, &banlist_path)?);

    init_tracing(&config.logging.filter)?;

    let database = Database::connect(&config.database.url)
        .await
        .context("failed to initialize database")?;
    database
        .migrate()
        .await
        .context("failed to run database migrations")?;

    let highlights_database =
        crate::db::HighlightsDatabase::connect(&config.database.highlights_url)
            .await
            .context("failed to initialize highlights database")?;
    highlights_database
        .migrate()
        .await
        .context("failed to run highlights database migrations")?;

    let highlight_cache = crate::domain::highlights::build_initial_cache(&highlights_database)
        .await
        .context("failed to build initial highlight cache")?;

    let state = AppState::new(config, database, highlights_database, highlight_cache);
    bot::framework::run(state).await
}

fn init_tracing(filter: &str) -> anyhow::Result<()> {
    let env_filter = EnvFilter::try_new(filter).or_else(|_| EnvFilter::try_new("info"))?;

    fmt()
        .with_env_filter(env_filter)
        .with_target(true)
        .compact()
        .try_init()
        .map_err(|error| anyhow::anyhow!("failed to initialize tracing subscriber: {error}"))?;

    Ok(())
}
