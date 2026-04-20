use poise::serenity_prelude::{ChannelId, Permissions, RoleId};

use crate::commands::{Context, Error, guild_settings, require_config_manager, send_status};

#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    subcommands(
        "view",
        "set_log_channel",
        "clear_log_channel",
        "set_require_reason",
        "set_ephemeral",
        "add_mod_role",
        "remove_mod_role"
    )
)]
pub async fn config(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(prefix_command, slash_command, guild_only)]
pub async fn view(ctx: Context<'_>) -> Result<(), Error> {
    let (_guild_id, settings) = guild_settings(ctx).await?;
    require_config_manager(ctx, &settings).await?;

    let mod_roles = if settings.mod_role_ids.is_empty() {
        "none".into()
    } else {
        settings
            .mod_role_ids
            .iter()
            .map(|role_id| format!("<@&{}>", role_id))
            .collect::<Vec<_>>()
            .join(", ")
    };

    let content = format!(
        "Guild settings\nlog_channel: {}\nrequire_reason: {}\nephemeral_slash_responses: {}\nnotes_enabled: {}\nappeals_enabled: {}\nmod_roles: {}",
        settings
            .log_channel_id
            .map(|channel_id| format!("<#{channel_id}>"))
            .unwrap_or_else(|| "not configured".into()),
        settings.require_reason,
        settings.ephemeral_slash_responses,
        settings.notes_enabled,
        settings.appeals_enabled,
        mod_roles,
    );

    send_status(ctx, &settings, content).await
}

#[poise::command(prefix_command, slash_command, guild_only)]
pub async fn set_log_channel(
    ctx: Context<'_>,
    #[description = "Channel to receive moderation logs"] channel: ChannelId,
) -> Result<(), Error> {
    let (guild_id, settings) = guild_settings(ctx).await?;
    require_config_manager(ctx, &settings).await?;

    ctx.data()
        .database()
        .set_log_channel(guild_id.get() as i64, Some(channel.get() as i64))
        .await?;

    let settings = ctx.data().guild_settings(guild_id).await?;
    send_status(
        ctx,
        &settings,
        format!("Moderation log channel set to <#{}>.", channel.get()),
    )
    .await
}

#[poise::command(prefix_command, slash_command, guild_only)]
pub async fn clear_log_channel(ctx: Context<'_>) -> Result<(), Error> {
    let (guild_id, settings) = guild_settings(ctx).await?;
    require_config_manager(ctx, &settings).await?;

    ctx.data()
        .database()
        .set_log_channel(guild_id.get() as i64, None)
        .await?;

    let settings = ctx.data().guild_settings(guild_id).await?;
    send_status(ctx, &settings, "Moderation log channel cleared.").await
}

#[poise::command(prefix_command, slash_command, guild_only)]
pub async fn set_require_reason(
    ctx: Context<'_>,
    #[description = "Whether moderation actions must include a reason"] value: bool,
) -> Result<(), Error> {
    let (guild_id, settings) = guild_settings(ctx).await?;
    require_config_manager(ctx, &settings).await?;

    ctx.data()
        .database()
        .set_require_reason(guild_id.get() as i64, value)
        .await?;

    let settings = ctx.data().guild_settings(guild_id).await?;
    send_status(ctx, &settings, format!("require_reason set to {value}.")).await
}

#[poise::command(prefix_command, slash_command, guild_only)]
pub async fn set_ephemeral(
    ctx: Context<'_>,
    #[description = "Whether slash command confirmations should be ephemeral"] value: bool,
) -> Result<(), Error> {
    let (guild_id, settings) = guild_settings(ctx).await?;
    require_config_manager(ctx, &settings).await?;

    ctx.data()
        .database()
        .set_ephemeral_slash_responses(guild_id.get() as i64, value)
        .await?;

    let settings = ctx.data().guild_settings(guild_id).await?;
    send_status(
        ctx,
        &settings,
        format!("ephemeral_slash_responses set to {value}."),
    )
    .await
}

#[poise::command(prefix_command, slash_command, guild_only)]
pub async fn add_mod_role(
    ctx: Context<'_>,
    #[description = "Role allowed to use moderation commands"] role: RoleId,
) -> Result<(), Error> {
    let (guild_id, settings) = guild_settings(ctx).await?;
    require_config_manager(ctx, &settings).await?;

    ctx.data()
        .database()
        .add_mod_role(guild_id.get() as i64, role.get() as i64)
        .await?;

    let settings = ctx.data().guild_settings(guild_id).await?;
    send_status(
        ctx,
        &settings,
        format!("Added <@&{}> as a moderator role.", role.get()),
    )
    .await
}

#[poise::command(prefix_command, slash_command, guild_only)]
pub async fn remove_mod_role(
    ctx: Context<'_>,
    #[description = "Role to remove from moderator access"] role: RoleId,
) -> Result<(), Error> {
    let (guild_id, settings) = guild_settings(ctx).await?;
    require_config_manager(ctx, &settings).await?;

    ctx.data()
        .database()
        .remove_mod_role(guild_id.get() as i64, role.get() as i64)
        .await?;

    let settings = ctx.data().guild_settings(guild_id).await?;
    send_status(
        ctx,
        &settings,
        format!("Removed <@&{}> from moderator roles.", role.get()),
    )
    .await
}

#[allow(dead_code)]
fn _config_permission_marker() -> Permissions {
    Permissions::MANAGE_GUILD
}
