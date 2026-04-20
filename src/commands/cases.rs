use crate::{
    commands::{
        Context, Error, create_case_and_log, guild_settings, normalized_reason, require_moderator,
        send_status,
    },
    domain::actions::{ModerationActionType, NewModerationCase},
    util::{format_duration, format_timestamp},
};
use poise::serenity_prelude::{Permissions, User};

#[poise::command(prefix_command, slash_command, guild_only)]
pub async fn case(
    ctx: Context<'_>,
    #[description = "Case number"] case_id: i64,
) -> Result<(), Error> {
    let (guild_id, settings) = guild_settings(ctx).await?;
    require_moderator(ctx, &settings, Permissions::MANAGE_MESSAGES).await?;

    let case = ctx
        .data()
        .database()
        .guild_case_by_id(guild_id.get() as i64, case_id)
        .await?;

    let mut lines = vec![
        format!("Case #{}", case.id),
        format!("Action: {}", case.action_type),
        format!("Moderator: <@{}>", case.moderator_user_id),
        format!(
            "Target: {}",
            case.target_user_id
                .map(|id| format!("<@{}>", id))
                .unwrap_or_else(|| "N/A".into())
        ),
        format!("Created: {}", format_timestamp(case.created_at)),
        format!(
            "Reason: {}",
            case.reason.unwrap_or_else(|| "No reason provided".into())
        ),
    ];

    if let Some(duration_seconds) = case.duration_seconds {
        lines.push(format!("Duration: {}", format_duration(duration_seconds)));
    }

    if let Some(expires_at) = case.expires_at {
        lines.push(format!("Expires: {}", format_timestamp(expires_at)));
    }

    if let Some(details) = case.details {
        lines.push(format!("Details: {}", details));
    }

    send_status(ctx, &settings, lines.join("\n")).await
}

#[poise::command(prefix_command, slash_command, guild_only)]
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
            reason,
            duration_seconds: None,
            details: Some(format!("Attached note to case #{case_id}")),
            expires_at: None,
        },
    )
    .await?;

    Ok(())
}
