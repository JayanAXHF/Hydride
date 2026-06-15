use std::io::stdout;

use poise::{
    CreateReply,
    serenity_prelude::{Channel, ChannelId, RoleId},
};

use crate::{
    bot::activity,
    commands::{Context, Error},
    domain::logging,
    terminal::print_channel_stats,
};

/// Fetch and display message activity statistics for a channel.
#[poise::command(prefix_command, slash_command, guild_only)]
pub async fn stats(
    ctx: Context<'_>,
    #[description = "Channel to analyse"] channel: Option<ChannelId>,
    #[description = "Lookback window (7, 14, or 30)"] days: Option<u32>,
    #[description = "Role to include in message share stats"] role: Option<RoleId>,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    let reply = ctx
        .send(
            CreateReply::default()
                .content("Loading stats...")
                .ephemeral(true),
        )
        .await?;

    let guild_id = ctx.guild_id().ok_or(crate::error::AppError::GuildOnly)?;
    let window_days = days.unwrap_or(30).clamp(7, 30);
    let channel_id = channel.unwrap_or_else(|| ctx.channel_id());
    let channel = channel_id.to_channel(ctx.serenity_context()).await?;
    let channel_name = match channel {
        Channel::Guild(guild_channel) => guild_channel.name,
        _ => channel_id.to_string(),
    };
    let role_id = role.unwrap_or_else(|| RoleId::new(activity::STARTUP_ROLE_ID));

    let snap = activity::fetch_live_channel_snapshot(
        ctx.serenity_context(),
        guild_id,
        channel_id,
        channel_name,
        window_days,
        Some(role_id),
    )
    .await?;

    let embed = logging::channel_stats_embed(&snap);
    if let Err(error) = print_channel_stats(&mut stdout(), &snap) {
        tracing::warn!(%error, "failed to print channel stats to terminal");
    }

    reply
        .edit(
            ctx,
            CreateReply::default()
                .content("")
                .embed(embed)
                .ephemeral(true),
        )
        .await?;
    Ok(())
}
