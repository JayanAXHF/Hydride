use poise::serenity_prelude::{GetMessages, Permissions, Timestamp, User};
use serenity::all::{EditMember, Member, Mentionable, Message, UserId};
use tokio::time::{Duration, sleep};

use crate::{
    commands::{
        Context, Error, create_case_and_log, ensure_action_target, fetch_target_member,
        guild_settings, normalized_message_id, normalized_reason, require_moderator, send_status,
    },
    domain::actions::{ModerationActionType, NewModerationCase},
    domain::logging,
    state::AppState,
    util::parse_duration,
};

/// Issue a formal warning.
#[poise::command(prefix_command, slash_command, guild_only, category = "Moderation")]
pub async fn warn(
    ctx: Context<'_>,
    #[description = "Target user"] user: User,
    #[description = "Related message ID"]
    #[rename = "message-id"]
    message_id: Option<u64>,
    #[description = "Reason for warning"]
    #[rest]
    reason: Option<String>,
) -> Result<(), Error> {
    let (guild_id, settings) = guild_settings(ctx).await?;
    let (_guild_id, _guild, _actor) =
        require_moderator(ctx, &settings, Permissions::MODERATE_MEMBERS).await?;
    let message_id = normalized_message_id(message_id)?;
    let reason = normalized_reason(&settings, reason)?;
    ctx.defer_ephemeral().await?;

    let (case, logged) = create_case_and_log(
        ctx,
        NewModerationCase {
            guild_id: guild_id.get() as i64,
            action_type: ModerationActionType::Warn,
            target_user_id: Some(user.id.get() as i64),
            moderator_user_id: ctx.author().id.get() as i64,
            message_id,
            reason,
            duration_seconds: None,
            details: None,
            expires_at: None,
        },
    )
    .await?;

    let suffix = if logged {
        ""
    } else {
        " Audit channel delivery failed."
    };
    send_status(
        ctx,
        &settings,
        format!("Warned {} with case #{}.{}", user.tag(), case.id, suffix),
    )
    .await
}

/// Mute or timeout a user.
#[poise::command(prefix_command, slash_command, guild_only, category = "Moderation")]
pub async fn timeout(
    ctx: Context<'_>,
    #[description = "Target user"] user: User,
    #[description = "Duration like 30m, 4h, 7d"] duration: String,
    #[description = "Related message ID"]
    #[rename = "message-id"]
    message_id: Option<u64>,
    #[description = "Reason for timeout"]
    #[rest]
    reason: Option<String>,
) -> Result<(), Error> {
    let (guild_id, settings) = guild_settings(ctx).await?;
    let (_guild_id, guild, actor) =
        require_moderator(ctx, &settings, Permissions::MODERATE_MEMBERS).await?;
    let duration_seconds = parse_duration(&duration)?;
    let message_id = normalized_message_id(message_id)?;
    let reason = normalized_reason(&settings, reason)?;
    ctx.defer_ephemeral().await?;

    let mut target = fetch_target_member(ctx, guild_id, user.id).await?;
    ensure_action_target(ctx, &guild, &actor, &target, Permissions::MODERATE_MEMBERS).await?;

    let expires_at = time::OffsetDateTime::now_utc().unix_timestamp() + duration_seconds;
    let timeout_until = Timestamp::from_unix_timestamp(expires_at).map_err(anyhow::Error::from)?;
    target
        .disable_communication_until_datetime(ctx.serenity_context(), timeout_until)
        .await?;

    let (case, logged) = create_case_and_log(
        ctx,
        NewModerationCase {
            guild_id: guild_id.get() as i64,
            action_type: ModerationActionType::Timeout,
            target_user_id: Some(user.id.get() as i64),
            moderator_user_id: ctx.author().id.get() as i64,
            message_id,
            reason,
            duration_seconds: Some(duration_seconds),
            details: None,
            expires_at: Some(expires_at),
        },
    )
    .await?;

    let suffix = if logged {
        ""
    } else {
        " Audit channel delivery failed."
    };
    send_status(
        ctx,
        &settings,
        format!("Timed out {} with case #{}.{}", user.tag(), case.id, suffix),
    )
    .await
}

/// Kick a user from the server.
#[poise::command(prefix_command, slash_command, guild_only, category = "Moderation")]
pub async fn kick(
    ctx: Context<'_>,
    #[description = "Target user"] user: User,
    #[description = "Related message ID"]
    #[rename = "message-id"]
    message_id: Option<u64>,
    #[description = "Reason for kick"]
    #[rest]
    reason: Option<String>,
) -> Result<(), Error> {
    let (guild_id, settings) = guild_settings(ctx).await?;
    let (_guild_id, guild, actor) =
        require_moderator(ctx, &settings, Permissions::KICK_MEMBERS).await?;
    let message_id = normalized_message_id(message_id)?;
    let reason = normalized_reason(&settings, reason)?;
    ctx.defer_ephemeral().await?;

    let target = fetch_target_member(ctx, guild_id, user.id).await?;
    ensure_action_target(ctx, &guild, &actor, &target, Permissions::KICK_MEMBERS).await?;
    guild_id
        .kick_with_reason(
            ctx.serenity_context(),
            user.id,
            reason.as_deref().unwrap_or("No reason provided"),
        )
        .await?;

    let (case, logged) = create_case_and_log(
        ctx,
        NewModerationCase {
            guild_id: guild_id.get() as i64,
            action_type: ModerationActionType::Kick,
            target_user_id: Some(user.id.get() as i64),
            moderator_user_id: ctx.author().id.get() as i64,
            message_id,
            reason,
            duration_seconds: None,
            details: None,
            expires_at: None,
        },
    )
    .await?;

    let suffix = if logged {
        ""
    } else {
        " Audit channel delivery failed."
    };
    send_status(
        ctx,
        &settings,
        format!("Kicked {} with case #{}.{}", user.tag(), case.id, suffix),
    )
    .await
}

/// Ban a user from the server.
#[poise::command(prefix_command, slash_command, guild_only, category = "Moderation")]
pub async fn ban(
    ctx: Context<'_>,
    #[description = "Target user"] user: User,
    #[description = "Related message ID"]
    #[rename = "message-id"]
    message_id: Option<u64>,
    #[description = "Reason for ban"]
    #[rest]
    reason: Option<String>,
) -> Result<(), Error> {
    let (guild_id, settings) = guild_settings(ctx).await?;
    let (_guild_id, guild, actor) =
        require_moderator(ctx, &settings, Permissions::BAN_MEMBERS).await?;
    let message_id = normalized_message_id(message_id)?;
    let reason = normalized_reason(&settings, reason)?;
    ctx.defer_ephemeral().await?;

    let target = fetch_target_member(ctx, guild_id, user.id).await;

    if let Ok(ref member) = target {
        ensure_action_target(ctx, &guild, &actor, member, Permissions::BAN_MEMBERS).await?;
    }

    guild_id
        .ban_with_reason(
            ctx.serenity_context(),
            user.id,
            0,
            reason.as_deref().unwrap_or("No reason provided"),
        )
        .await?;

    let (case, logged) = create_case_and_log(
        ctx,
        NewModerationCase {
            guild_id: guild_id.get() as i64,
            action_type: ModerationActionType::Ban,
            target_user_id: Some(user.id.get() as i64),
            moderator_user_id: ctx.author().id.get() as i64,
            message_id,
            reason,
            duration_seconds: None,
            details: None,
            expires_at: None,
        },
    )
    .await?;

    let suffix = if logged {
        ""
    } else {
        " Audit channel delivery failed."
    };
    send_status(
        ctx,
        &settings,
        format!("Banned {} with case #{}.{}", user.tag(), case.id, suffix),
    )
    .await
}

/// Ban a user for a limited duration.
#[poise::command(prefix_command, slash_command, guild_only, category = "Moderation")]
pub async fn tempban(
    ctx: Context<'_>,
    #[description = "Target user"] user: User,
    #[description = "Duration like 30m, 4h, 7d"] duration: String,
    #[description = "Related message ID"]
    #[rename = "message-id"]
    message_id: Option<u64>,
    #[description = "Reason for tempban"]
    #[rest]
    reason: Option<String>,
) -> Result<(), Error> {
    let (guild_id, settings) = guild_settings(ctx).await?;
    let (_guild_id, guild, actor) =
        require_moderator(ctx, &settings, Permissions::BAN_MEMBERS).await?;
    let duration_seconds = parse_duration(&duration)?;
    let message_id = normalized_message_id(message_id)?;
    let reason = normalized_reason(&settings, reason)?;
    ctx.defer_ephemeral().await?;

    let target = fetch_target_member(ctx, guild_id, user.id).await;

    if let Ok(ref member) = target {
        ensure_action_target(ctx, &guild, &actor, member, Permissions::BAN_MEMBERS).await?;
    }

    guild_id
        .ban_with_reason(
            ctx.serenity_context(),
            user.id,
            0,
            reason.as_deref().unwrap_or("No reason provided"),
        )
        .await?;

    let expires_at = time::OffsetDateTime::now_utc().unix_timestamp() + duration_seconds;
    schedule_tempban_unban(
        ctx.serenity_context().clone(),
        guild_id,
        user.id,
        duration_seconds,
    );

    let (case, logged) = create_case_and_log(
        ctx,
        NewModerationCase {
            guild_id: guild_id.get() as i64,
            action_type: ModerationActionType::Ban,
            target_user_id: Some(user.id.get() as i64),
            moderator_user_id: ctx.author().id.get() as i64,
            message_id,
            reason,
            duration_seconds: Some(duration_seconds),
            details: Some(format!("Temporary ban until {expires_at}")),
            expires_at: Some(expires_at),
        },
    )
    .await?;

    let suffix = if logged {
        ""
    } else {
        " Audit channel delivery failed."
    };
    send_status(
        ctx,
        &settings,
        format!(
            "Temporarily banned {} for {} with case #{}.{}",
            user.tag(),
            duration,
            case.id,
            suffix
        ),
    )
    .await
}

/// Unban a previously banned user.
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    category = "Moderation",
    required_permissions = "MODERATE_MEMBERS"
)]
pub async fn unban(
    ctx: Context<'_>,
    #[description = "Target user"] user: User,
    #[description = "Related message ID"]
    #[rename = "message-id"]
    message_id: Option<u64>,
    #[description = "Reason for unban"]
    #[rest]
    reason: Option<String>,
) -> Result<(), Error> {
    let (guild_id, settings) = guild_settings(ctx).await?;
    require_moderator(ctx, &settings, Permissions::BAN_MEMBERS).await?;
    let message_id = normalized_message_id(message_id)?;
    let reason = normalized_reason(&settings, reason)?;

    guild_id.unban(ctx.serenity_context(), user.id).await?;

    let (case, logged) = create_case_and_log(
        ctx,
        NewModerationCase {
            guild_id: guild_id.get() as i64,
            action_type: ModerationActionType::Unban,
            target_user_id: Some(user.id.get() as i64),
            moderator_user_id: ctx.author().id.get() as i64,
            message_id,
            reason,
            duration_seconds: None,
            details: None,
            expires_at: None,
        },
    )
    .await?;

    let suffix = if logged {
        ""
    } else {
        " Audit channel delivery failed."
    };
    send_status(
        ctx,
        &settings,
        format!("Unbanned {} with case #{}.{}", user.tag(), case.id, suffix),
    )
    .await
}

/// Bulk delete messages in the current channel.
#[poise::command(
    prefix_command,
    slash_command,
    required_permissions = "MANAGE_MESSAGES"
)]
pub async fn purge(
    ctx: Context<'_>,
    #[description = "User to delete messages from"] user: Option<User>,
    #[description = "Number of recent messages to delete"] count: u8,
    #[description = "Related message ID"]
    #[rename = "message-id"]
    message_id: Option<u64>,
    #[description = "Reason for purge"]
    #[rest]
    reason: Option<String>,
) -> Result<(), Error> {
    let (guild_id, settings) = guild_settings(ctx).await?;
    require_moderator(ctx, &settings, Permissions::MANAGE_MESSAGES).await?;
    let message_id = normalized_message_id(message_id)?;
    let reason = normalized_reason(&settings, reason)?;

    let channel_id = ctx.channel_id();
    let messages_to_delete = match user {
        Some(ref user) => get_messages_from_user(user.clone(), count, &ctx).await?,
        None => get_n_messages(count, &ctx).await?,
    };

    ctx.defer_ephemeral().await?;
    if count as usize > messages_to_delete.len() {
        ctx.say("Not enough messages to delete, deleting as many as possible.")
            .await?;
    }
    let chunks = messages_to_delete.chunks(100);
    for chunk in chunks {
        channel_id.delete_messages(&ctx, chunk).await?;
    }
    let user_str = match user {
        Some(ref user) => format!("by {}", user.mention()),
        None => String::new(),
    };

    ctx.say(format!("Deleted {count} messages {user_str}"))
        .await?;

    let details = format!("Deleted {count} messages in <#{}>", channel_id.get());
    let (case, logged) = create_case_and_log(
        ctx,
        NewModerationCase {
            guild_id: guild_id.get() as i64,
            action_type: ModerationActionType::Purge,
            target_user_id: None,
            moderator_user_id: ctx.author().id.get() as i64,
            message_id,
            reason,
            duration_seconds: None,
            details: Some(details),
            expires_at: None,
        },
    )
    .await?;

    let suffix = if logged {
        ""
    } else {
        " Audit channel delivery failed."
    };
    send_status(
        ctx,
        &settings,
        format!("Purged {count} messages with case #{}.{}", case.id, suffix),
    )
    .await
}

#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    category = "Moderation",
    required_permissions = "MODERATE_MEMBERS"
)]
pub async fn nick(
    ctx: Context<'_>,
    #[description = "Target User"] mut member: Member,
    #[description = "New displayname"] new_name: String,
    #[rest]
    #[description = "Reason for change"]
    reason: Option<String>,
) -> Result<(), Error> {
    let (guild_id, settings) = guild_settings(ctx).await?;
    let reason = reason.unwrap_or_else(|| "Innappropriate username".to_string());
    let edit_member = EditMember::new().nickname(&new_name);
    let old_nick = member.display_name().to_string();
    member.edit(&ctx, edit_member).await?;

    let (case, logged) = create_case_and_log(
        ctx,
        NewModerationCase {
            guild_id: guild_id.into(),
            action_type: ModerationActionType::NicknameChange,
            target_user_id: Some(member.user.id.into()),
            moderator_user_id: ctx.author().id.get() as i64,
            message_id: None,
            reason: Some(reason),
            duration_seconds: None,
            details: Some(format!("Old: {old_nick}; New: {new_name}")),
            expires_at: None,
        },
    )
    .await?;

    let suffix = if logged {
        ""
    } else {
        " Audit channel delivery failed."
    };
    send_status(
        ctx,
        &settings,
        format!(
            "Changed nick of {} from {old_nick} to {new_name}. Case ID #{}. {suffix}",
            member.mention(),
            case.id
        ),
    )
    .await?;

    Ok(())
}

async fn get_messages_from_user(
    user: User,
    amount: u8,
    ctx: &Context<'_>,
) -> Result<Vec<Message>, Error> {
    let channel_id = ctx.channel_id();
    let mut messages_to_delete = Vec::with_capacity(amount as usize);
    while messages_to_delete.len() < amount as usize {
        let messages = channel_id
            .messages(ctx.http(), GetMessages::new().limit(100))
            .await?;

        if messages.is_empty() {
            break; // No more messages to fetch
        }

        let user_messages: Vec<Message> = messages
            .into_iter()
            .filter(|m| m.author.id == user.id)
            .collect();

        messages_to_delete.extend(user_messages);
    }

    Ok(messages_to_delete)
}

async fn get_n_messages(amount: u8, ctx: &Context<'_>) -> Result<Vec<Message>, Error> {
    let channel_id = ctx.channel_id();
    if amount <= 100 {
        let messages = channel_id
            .messages(ctx.http(), GetMessages::new().limit(amount))
            .await?;
        return Ok(messages);
    }

    // Fetch messages in batches of 100 until we have enough
    let q = amount / 100;
    let r = amount % 100;
    let mut messages_to_delete = Vec::with_capacity(amount as usize);
    let mut last_message_id = None;
    for _ in 0..q {
        let mut builder = GetMessages::new().limit(100);
        if let Some(last_id) = last_message_id {
            builder = builder.before(last_id);
        }
        let messages = channel_id.messages(ctx.http(), builder).await?;
        if messages.is_empty() {
            break; // No more messages to fetch
        }
        last_message_id = messages.last().map(|m| m.id);
        messages_to_delete.extend(messages);
    }
    if r > 0 {
        let mut builder = GetMessages::new().limit(r);
        if let Some(last_id) = last_message_id {
            builder = builder.before(last_id);
        }
        let messages = channel_id.messages(ctx.http(), builder).await?;
        messages_to_delete.extend(messages);
    }
    Ok(messages_to_delete)
}

fn schedule_tempban_unban(
    serenity_context: poise::serenity_prelude::Context,
    guild_id: poise::serenity_prelude::GuildId,
    user_id: poise::serenity_prelude::UserId,
    duration_seconds: i64,
) {
    tokio::spawn(async move {
        let wait_seconds = u64::try_from(duration_seconds).unwrap_or(0);
        sleep(Duration::from_secs(wait_seconds)).await;

        if let Err(error) = guild_id.unban(&serenity_context, user_id).await {
            tracing::error!(
                guild_id = guild_id.get(),
                user_id = user_id.get(),
                %error,
                "failed to unban user after tempban duration"
            );
        }
    });
}

pub async fn reconcile_expired_tempbans(
    ctx: &poise::serenity_prelude::Context,
    state: &AppState,
    bot_user_id: UserId,
) -> Result<(), Error> {
    let now = time::OffsetDateTime::now_utc().unix_timestamp();
    let expired_cases = state.database().list_expired_tempban_cases(now).await?;

    for case in expired_cases {
        let Some(target_user_id) = case.target_user_id else {
            continue;
        };

        let guild_id = poise::serenity_prelude::GuildId::new(case.guild_id as u64);
        let user_id = UserId::new(target_user_id as u64);

        if let Err(error) = guild_id.unban(ctx, user_id).await {
            tracing::warn!(
                guild_id = guild_id.get(),
                user_id = user_id.get(),
                case_id = case.id,
                %error,
                "failed to unban expired tempban during startup reconciliation"
            );
            continue;
        }

        let unban_case = NewModerationCase {
            guild_id: case.guild_id,
            action_type: ModerationActionType::Unban,
            target_user_id: case.target_user_id,
            moderator_user_id: bot_user_id.get() as i64,
            message_id: None,
            reason: Some("Automatic unban after tempban expiry".into()),
            duration_seconds: None,
            details: Some(format!(
                "Automatic unban for expired tempban case #{}",
                case.id
            )),
            expires_at: None,
        };

        let unban_case = state.database().create_case(&unban_case).await?;
        let channel_id = match state.audit_log_channel(guild_id).await {
            Ok(channel_id) => channel_id,
            Err(error) => {
                tracing::warn!(
                    guild_id = guild_id.get(),
                    case_id = case.id,
                    %error,
                    "missing audit log channel while recording automatic unban"
                );
                continue;
            }
        };

        match logging::send_case_log(ctx, channel_id, &unban_case).await {
            Ok(message) => {
                state
                    .database()
                    .update_case_audit_message(
                        unban_case.id,
                        channel_id.get() as i64,
                        message.id.get() as i64,
                    )
                    .await?;
            }
            Err(error) => {
                tracing::warn!(
                    guild_id = guild_id.get(),
                    case_id = unban_case.id,
                    %error,
                    "failed to send automatic unban audit log"
                );
            }
        }
    }

    Ok(())
}
