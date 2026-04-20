use std::{env, path::PathBuf, sync::Arc};

use anyhow::Context;
use tracing_subscriber::{EnvFilter, fmt};

use crate::{bot, config::BootstrapConfig, db::Database, state::AppState};

pub async fn run() -> anyhow::Result<()> {
    let config_path = env::var("MODBOT_CONFIG")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("config.toml"));
    let config = Arc::new(BootstrapConfig::load(&config_path)?);

    init_tracing(&config.logging.filter)?;

    let database = Database::connect(&config.database.url)
        .await
        .context("failed to initialize database")?;
    database
        .migrate()
        .await
        .context("failed to run database migrations")?;

    let state = AppState::new(config, database);
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
