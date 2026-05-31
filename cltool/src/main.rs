mod app;
mod changelog;
mod config;
mod discord;
mod error;
mod render;

use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
#[command(author, version, about)]
pub struct Cli {
    #[arg(short, long, default_value = "cltool.toml")]
    pub config: PathBuf,

    #[arg(long)]
    pub webhook_url: Option<String>,

    #[arg(long)]
    pub dry_run: bool,

    #[arg(long)]
    pub stdout: bool,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    if let Err(error) = app::run(cli).await {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
