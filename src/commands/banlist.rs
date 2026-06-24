use crate::commands::{Context, Error, guild_settings, meta::fmt_user, normalized_reason};
use poise::CreateReply;
use serenity::all::{CreateEmbed, CreateEmbedFooter, User, UserId};
use std::fmt::Write;
use time::{UtcDateTime, format_description::well_known::Rfc2822};

/// Manage the guild banlist
#[poise::command(
    prefix_command,
    slash_command,
    required_permissions = "MODERATE_MEMBERS",
    guild_only,
    subcommands("list", "add", "remove")
)]
pub async fn banlist(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// List all banlist entries
#[poise::command(
    prefix_command,
    slash_command,
    required_permissions = "MODERATE_MEMBERS",
    guild_only
)]
pub async fn list(ctx: Context<'_>) -> Result<(), Error> {
    let db = ctx.data().blacklist_database();
    let banlist = db
        .blacklist_for_guild(ctx.guild_id().unwrap().into())
        .await?;

    let mut embed_desc = String::new();

    for (i, blacklist_record) in banlist.iter().enumerate() {
        let user = UserId::new(blacklist_record.user_id as u64)
            .to_user(&ctx)
            .await?;

        write!(embed_desc, "{}. ", i + 1)?;

        let time =
            UtcDateTime::from_unix_timestamp(blacklist_record.created_at)?.format(&Rfc2822)?;

        fmt_user(&user, &mut embed_desc)?;
        write!(embed_desc, " #{} at {}", blacklist_record.id, time)?;
        if let Some(ref reason) = blacklist_record.reason {
            write!(embed_desc, " - {reason}")?;
        }
        writeln!(embed_desc)?;
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

/// Add a user to the banlist
#[poise::command(
    prefix_command,
    slash_command,
    required_permissions = "MODERATE_MEMBERS",
    guild_only
)]
pub async fn add(
    ctx: Context<'_>,
    #[description = "Target user"] user: User,
    #[description = "Reason for adding to the banlist"]
    #[rest]
    reason: Option<String>,
) -> Result<(), Error> {
    let (guild_id, settings) = guild_settings(ctx).await?;
    let reason = normalized_reason(&settings, reason)?;

    let db = ctx.data().blacklist_database();

    let record = db
        .add_blacklist(guild_id.get() as i64, user.id.get() as i64, reason)
        .await?;

    if let Some(record) = record {
        ctx.say(format!("Added banlist entry with id #{}", record.id))
            .await?;
    }

    Ok(())
}

/// Remove a banlist entry
#[poise::command(
    prefix_command,
    slash_command,
    required_permissions = "MODERATE_MEMBERS",
    guild_only
)]
pub async fn remove(ctx: Context<'_>, id: i64) -> Result<(), Error> {
    let db = ctx.data().blacklist_database();

    let removed = db
        .remove_blacklist(ctx.guild_id().unwrap().into(), id)
        .await?;

    if removed {
        ctx.say(format!("Removed banlist entry #{}", id)).await?;
    }

    Ok(())
}
