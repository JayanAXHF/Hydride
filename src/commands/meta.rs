use std::time::Instant;
use time::ext::InstantExt;

use anyhow::bail;

use crate::{
    app::START_TIMESTAMP,
    commands::{Context, Error},
};

#[poise::command(prefix_command, slash_command)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("pong").await?;
    Ok(())
}

#[poise::command(prefix_command, slash_command, track_edits)]
pub async fn help(ctx: Context<'_>, #[rest] command: Option<String>) -> Result<(), Error> {
    let config = poise::builtins::HelpConfiguration {
        show_subcommands: true,
        show_context_menu_commands: false,
        ..Default::default()
    };

    poise::builtins::help(ctx, command.as_deref(), config).await?;
    Ok(())
}

#[poise::command(prefix_command, slash_command)]
pub async fn uptime(ctx: Context<'_>) -> Result<(), Error> {
    let Some(time) = START_TIMESTAMP.get() else {
        bail!("Unable to get start timestamp");
    };
    let passed = Instant::now().signed_duration_since(*time);
    ctx.say(format!("Uptime: {}", passed)).await?;
    Ok(())
}
