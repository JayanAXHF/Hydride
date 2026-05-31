use crate::Cli;
use crate::changelog::generate as generate_changelog;
use crate::config::AppConfig;
use crate::discord::DiscordWebhookClient;
use crate::error::AppError;
use crate::render::{chunk_for_discord, with_heading};

pub async fn run(cli: Cli) -> Result<(), AppError> {
    let (config, config_path) = AppConfig::load(&cli.config)?;
    let resolved = config.resolve(&config_path, cli.webhook_url.as_deref())?;
    let changelog = generate_changelog(
        &resolved.repo_path,
        &resolved.cliff_config_path,
        resolved.range.as_deref(),
    )?;
    let rendered = with_heading(&changelog, resolved.output.heading.as_deref());
    let chunks = chunk_for_discord(&rendered, resolved.output.max_content_chars);

    if cli.stdout {
        print!("{rendered}");
        return Ok(());
    }

    if cli.dry_run {
        print_dry_run(&chunks);
        return Ok(());
    }

    let client = DiscordWebhookClient::new(resolved.discord)?;
    let sent_ids = client.sync_messages(&chunks).await?;
    tracing::info!("updated {} Discord message(s)", sent_ids.len());
    Ok(())
}

fn print_dry_run(chunks: &[String]) {
    println!("dry run: {} chunk(s)", chunks.len());
    for (index, chunk) in chunks.iter().enumerate() {
        println!("\n--- chunk {} ---\n{}", index + 1, chunk);
    }
}
