use crate::{
    app::START_TIMESTAMP,
    commands::{Context, Error},
};
use anyhow::bail;
use futures::{self, StreamExt, stream};
use poise::CreateReply;
use serenity::all::{CreateEmbed, CreateEmbedFooter, User, UserId};
use std::fmt::Write;
use time::{UtcDateTime, ext::InstantExt, format_description::well_known::Rfc2822};

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
    let passed = UtcDateTime::now() - *time;
    ctx.say(format!("Uptime: {}", passed)).await?;
    Ok(())
}
#[poise::command(prefix_command, slash_command)]
pub async fn last_updated(ctx: Context<'_>) -> Result<(), Error> {
    let Some(time) = START_TIMESTAMP.get() else {
        bail!("Unable to get last update timestamp");
    };
    let formatted = time.format(&Rfc2822)?;
    ctx.say(format!("Last Updated at {formatted}")).await?;
    Ok(())
}

#[poise::command(prefix_command, slash_command)]
pub async fn pinglist(ctx: Context<'_>) -> Result<(), Error> {
    let pinglist_ids = ctx.data().config().join_pinglist.members.clone();
    let users = u64_to_user(ctx, pinglist_ids);
    let mut embed_desc = String::new();
    for (i, user) in users.await.iter().enumerate() {
        write!(embed_desc, "{}. ", i + 1)?;
        fmt_user(user, &mut embed_desc)?;
    }
    let now = time::UtcDateTime::now();
    let embed = CreateEmbed::new()
        .title("Pinglist")
        .description(embed_desc)
        .footer(CreateEmbedFooter::new(format!(
            "Last updated at {}",
            now.format(&Rfc2822)?
        )));
    ctx.send(CreateReply::default().embed(embed)).await?;
    Ok(())
}
#[poise::command(prefix_command, slash_command)]
pub async fn banlist(ctx: Context<'_>) -> Result<(), Error> {
    let banlist = ctx.data().config().banlist.ids.clone();
    let users = u64_to_user(ctx, banlist);
    let mut embed_desc = String::new();
    for (i, user) in users.await.iter().enumerate() {
        write!(embed_desc, "{}. ", i + 1)?;
        fmt_user(user, &mut embed_desc)?;
    }
    let now = time::UtcDateTime::now();
    let embed = CreateEmbed::new()
        .title("Banlist")
        .description(embed_desc)
        .footer(CreateEmbedFooter::new(format!(
            "Last updated at {}",
            now.format(&Rfc2822)?
        )));
    ctx.send(CreateReply::default().embed(embed)).await?;
    Ok(())
}

async fn u64_to_user(ctx: Context<'_>, ids: Vec<u64>) -> Vec<User> {
    let users: Vec<_> = stream::iter(ids)
        .filter_map(async |f| UserId::from(f).to_user(&ctx).await.ok())
        .collect()
        .await;
    users
}

fn fmt_user(
    User { id, name, .. }: &User,
    f: &mut impl std::fmt::Write,
) -> Result<(), std::fmt::Error> {
    write!(f, "{name} - {id}")
}
