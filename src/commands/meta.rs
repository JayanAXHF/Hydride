use crate::{
    app::START_TIMESTAMP,
    commands::{Context, Error},
};
use anyhow::bail;
use futures::{self, StreamExt, stream};
use poise::CreateReply;
use serenity::all::{
    CreateAllowedMentions, CreateAttachment, CreateEmbed, CreateEmbedFooter, EditRole, Mentionable,
    Role, RoleId, User, UserId,
};
use std::fmt::Write;
use time::{UtcDateTime, format_description::well_known::Rfc2822};

const SOURCE: &str = "https://github.com/plushys-playground-discord/Hydride";

#[poise::command(prefix_command, slash_command, category = "Meta")]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("pong").await?;
    Ok(())
}

/// Print help message.
#[poise::command(prefix_command, slash_command, track_edits, category = "Meta")]
pub async fn help(ctx: Context<'_>, #[rest] command: Option<String>) -> Result<(), Error> {
    let config = poise::builtins::HelpConfiguration {
        ..Default::default()
    };

    poise::builtins::help(ctx, command.as_deref(), config).await?;
    Ok(())
}

/// How long has the bot been running.
#[poise::command(prefix_command, slash_command, category = "Meta")]
pub async fn uptime(ctx: Context<'_>) -> Result<(), Error> {
    let Some(time) = START_TIMESTAMP.get() else {
        bail!("Unable to get start timestamp");
    };
    let passed = UtcDateTime::now() - *time;
    ctx.say(format!("Uptime: {}", passed)).await?;
    Ok(())
}
/// Displays when the bot was last started.
#[poise::command(prefix_command, slash_command, category = "Meta")]
pub async fn last_updated(ctx: Context<'_>) -> Result<(), Error> {
    let Some(time) = START_TIMESTAMP.get() else {
        bail!("Unable to get last update timestamp");
    };
    let formatted = time.format(&Rfc2822)?;
    ctx.say(format!("Last Updated at {formatted}")).await?;
    Ok(())
}
#[poise::command(prefix_command, slash_command, category = "Meta")]
pub async fn source(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say(SOURCE).await?;
    Ok(())
}

/// List users on the join-pinglist.
#[poise::command(
    prefix_command,
    slash_command,
    required_permissions = "MODERATE_MEMBERS"
)]
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

/// Change the icon of a role by its ID
#[poise::command(slash_command, guild_only, required_permissions = "MANAGE_ROLES")]
pub async fn set_role_icon(
    ctx: Context<'_>,
    #[description = "The role to update"] role: Role,
    #[description = "Attachment to use as the role icon"] icon: serenity::all::Attachment,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap(); // can't be run outside a guild as set above

    // Validate it's an image
    let content_type = icon.content_type.as_deref().unwrap_or("");
    if !content_type.starts_with("image/") {
        ctx.say("Attachment must be an image.").await?;
        return Ok(());
    }

    let bytes = icon.download().await?;

    // CreateAttachment::bytes(data, filename) — serenity infers the type from the filename
    let attachment = CreateAttachment::bytes(bytes, &icon.filename);

    guild_id
        .edit_role(ctx.http(), role.id, EditRole::new().icon(Some(&attachment)))
        .await?;

    ctx.say(format!("Updated icon for role **{}**.", role.name))
        .await?;

    Ok(())
}

#[poise::command(prefix_command)]
pub async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

async fn u64_to_user(ctx: Context<'_>, ids: Vec<u64>) -> Vec<User> {
    let users: Vec<_> = stream::iter(ids)
        .filter_map(async |f| UserId::from(f).to_user(&ctx).await.ok())
        .collect()
        .await;
    users
}

#[poise::command(prefix_command, slash_command, guild_cooldown = 7200)]
pub async fn revive(ctx: Context<'_>, #[rest] question: String) -> Result<(), Error> {
    let Some(revive_role_id) = ctx
        .data()
        .config()
        .moderation
        .revive_role_id
        .map(|f| RoleId::new(f))
    else {
        ctx.say("No revive role configured").await?;
        return Ok(());
    };
    let embed = CreateEmbed::new()
        .title("Chat Revive!")
        .description(format!(
            "The chat is dead {}. Come back to life!",
            revive_role_id.mention()
        ))
        .field("Question", question, false);
    let reply = CreateReply::default()
        .embed(embed)
        .allowed_mentions(CreateAllowedMentions::default().all_roles(true))
        .content(revive_role_id.mention().to_string());
    ctx.send(reply).await?;
    Ok(())
}
pub fn fmt_user(
    User { id, name, .. }: &User,
    f: &mut impl std::fmt::Write,
) -> Result<(), std::fmt::Error> {
    write!(f, "{name} - {id}")
}
