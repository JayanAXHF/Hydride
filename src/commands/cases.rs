use std::str::FromStr;

use crate::{
    commands::{
        Context, Error, create_case_and_log, guild_settings, normalized_reason, require_moderator,
        send_status,
    },
    domain::actions::{ModerationActionType, NewModerationCase},
    util::{format_duration, format_timestamp},
};
use poise::{
    CreateReply,
    serenity_prelude::{Permissions, User},
};
use serenity::all::CreateEmbed;

/// View details of a case by ID.
#[poise::command(prefix_command, slash_command, guild_only, category = "Cases")]
pub async fn case(
    ctx: Context<'_>,
    #[description = "Case number"] case_id: i64,
) -> Result<(), Error> {
    let (guild_id, settings) = guild_settings(ctx).await?;
    require_moderator(ctx, &settings, Permissions::MANAGE_MESSAGES).await?;
    let mut embed = CreateEmbed::new();

    let case = ctx
        .data()
        .database()
        .guild_case_by_id(guild_id.get() as i64, case_id)
        .await?;

    let mut fields = vec![
        ("Case #", case.id.to_string(), true),
        ("Action", case.action_type.to_string(), true),
        ("Moderator", format!("<@{}>", case.moderator_user_id), true),
        (
            "Target",
            case.target_user_id
                .map(|id| format!("<@{}>", id))
                .unwrap_or_else(|| "N/A".into()),
            true,
        ),
        (
            "Reason",
            case.reason.unwrap_or_else(|| "No reason provided".into()),
            false,
        ),
    ];

    if let Some(message_id) = case.message_id {
        fields.push(("Message ID", message_id.to_string(), false));
    }

    if let Some(duration_seconds) = case.duration_seconds {
        fields.push(("Duration", format_duration(duration_seconds), true));
    }

    if let Some(expires_at) = case.expires_at {
        fields.push(("Expires", format_timestamp(expires_at), false));
    }

    if let Some(details) = case.details {
        fields.push(("Details", details, false));
    }

    embed = embed.fields(fields);
    let reply = CreateReply::default().embed(embed);
    ctx.send(reply).await?;
    Ok(())
}

/// List moderation cases for a user.
#[poise::command(prefix_command, slash_command, guild_only, category = "Cases")]
pub async fn cases(
    ctx: Context<'_>,
    #[description = "Target user"] user: User,
    #[description = "Number of cases to show"] limit: Option<u8>,
) -> Result<(), Error> {
    let (guild_id, settings) = guild_settings(ctx).await?;
    require_moderator(ctx, &settings, Permissions::MANAGE_MESSAGES).await?;

    let limit = limit
        .unwrap_or(ctx.data().config().moderation.max_case_results)
        .min(ctx.data().config().moderation.max_case_results);
    let cases = ctx
        .data()
        .database()
        .list_cases_for_user(guild_id.get() as i64, user.id.get() as i64, limit)
        .await?;

    if cases.is_empty() {
        send_status(
            ctx,
            &settings,
            format!("No cases found for {}.", user.tag()),
        )
        .await?;
        return Ok(());
    }

    let summary = cases
        .into_iter()
        .map(|case| {
            let reason = case.reason.unwrap_or_else(|| "No reason".into());
            format!(
                "#{} [{}] {} - {}",
                case.id,
                case.action_type,
                format_timestamp(case.created_at),
                reason
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    send_status(ctx, &settings, summary).await
}

/// Remove a warning case by ID.
#[poise::command(prefix_command, slash_command, guild_only, category = "Cases")]
pub async fn remove_warn(
    ctx: Context<'_>,
    #[description = "Case number of the warning to remove"] case_id: i64,
    #[description = "Reason for removing the warning"]
    #[rest]
    reason: Option<String>,
) -> Result<(), Error> {
    let (guild_id, settings) = guild_settings(ctx).await?;
    require_moderator(ctx, &settings, Permissions::MANAGE_MESSAGES).await?;

    let case = ctx
        .data()
        .database()
        .guild_case_by_id(guild_id.get() as i64, case_id)
        .await?;

    if ModerationActionType::from_str(&case.action_type)? != ModerationActionType::Warn {
        send_status(
            ctx,
            &settings,
            format!("Case #{case_id} is not a warning (it is a `{}`). Only warn cases can be removed with this command.", case.action_type),
        )
        .await?;
        return Ok(());
    }

    let target_user_id = case.target_user_id;
    let normalized = normalized_reason(&settings, reason)?;
    let removal_note = normalized
        .clone()
        .unwrap_or_else(|| "Removed by moderator".into());

    ctx.data()
        .database()
        .update_case_reason(case_id, &format!("[REMOVED] {removal_note}"))
        .await?;

    let (_case, _logged) = create_case_and_log(
        ctx,
        NewModerationCase {
            guild_id: guild_id.get() as i64,
            action_type: ModerationActionType::Note,
            target_user_id,
            moderator_user_id: ctx.author().id.get() as i64,
            message_id: None,
            reason: normalized,
            duration_seconds: None,
            details: Some(format!("Removed warning case #{case_id}")),
            expires_at: None,
        },
    )
    .await?;

    let target_mention = target_user_id
        .map(|id| format!(" for <@{id}>"))
        .unwrap_or_default();

    send_status(
        ctx,
        &settings,
        format!("Warning case #{case_id}{target_mention} has been removed."),
    )
    .await
}

/// Update the reason for an existing case.
#[poise::command(prefix_command, slash_command, guild_only, category = "Cases")]
pub async fn update_reason(
    ctx: Context<'_>,
    #[description = "Case number to update"] case_id: i64,
    #[rest]
    #[description = "New reason"]
    reason: String,
) -> Result<(), Error> {
    let (guild_id, settings) = guild_settings(ctx).await?;
    require_moderator(ctx, &settings, Permissions::MANAGE_MESSAGES).await?;

    // Verify the case exists and belongs to this guild before updating.
    let _case = ctx
        .data()
        .database()
        .guild_case_by_id(guild_id.get() as i64, case_id)
        .await?;

    let normalized = normalized_reason(&settings, Some(reason))?;
    let reason_str = normalized.clone().unwrap_or_default();

    ctx.data()
        .database()
        .update_case_reason(case_id, &reason_str)
        .await?;

    send_status(
        ctx,
        &settings,
        format!("Case #{case_id} reason updated to: {reason_str}"),
    )
    .await
}

/// Attach a note to an case.
#[allow(dead_code)]
pub async fn add_note(
    ctx: Context<'_>,
    case_id: i64,
    content: Option<String>,
) -> Result<(), Error> {
    let (guild_id, settings) = guild_settings(ctx).await?;
    require_moderator(ctx, &settings, Permissions::MANAGE_MESSAGES).await?;

    let reason = normalized_reason(&settings, content)?;
    let note_content = reason.clone().unwrap_or_default();
    ctx.data()
        .database()
        .add_note(case_id, ctx.author().id.get() as i64, &note_content)
        .await?;

    let (_case, _logged) = create_case_and_log(
        ctx,
        NewModerationCase {
            guild_id: guild_id.get() as i64,
            action_type: ModerationActionType::Note,
            target_user_id: None,
            moderator_user_id: ctx.author().id.get() as i64,
            message_id: None,
            reason,
            duration_seconds: None,
            details: Some(format!("Attached note to case #{case_id}")),
            expires_at: None,
        },
    )
    .await?;

    Ok(())
}
