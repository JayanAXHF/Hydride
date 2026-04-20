use crate::commands::{Context, Error};

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
