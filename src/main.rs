mod app;
mod bot;
mod commands;
mod config;
mod db;
mod domain;
mod error;
mod state;
mod util;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    app::run().await
}
