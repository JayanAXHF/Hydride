use crate::{
    commands::{Context, Error, guild_settings, require_moderator, send_status},
    db::models::LeaveApplicationRecord,
    domain::actions::NewLeaveApplication,
    domain::logging,
    error::AppError,
    util::{format_timestamp, parse_leave_window},
};
use poise::{
    CreateReply,
    serenity_prelude::{Permissions, User},
};

/// Manage leave applications.
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    subcommands("add", "active", "user")
)]
pub async fn leave(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Create a new leave application for a user.
#[poise::command(prefix_command, slash_command, guild_only)]
pub async fn add(
    ctx: Context<'_>,
    #[description = "Applicant"] applicant: User,
    #[description = "Leave window like 2026-05-29T10:00:00Z..2026-06-05T18:00:00Z or 7d"]
    window: String,
    #[description = "Reason for the leave notice"]
    #[rest]
    reason: String,
) -> Result<(), Error> {
    let (guild_id, settings) = guild_settings(ctx).await?;
    require_moderator(ctx, &settings, Permissions::MANAGE_MESSAGES).await?;

    let (duration_text, starts_at, ends_at) = parse_leave_window(&window)?;
    let reason = normalize_required_text("reason", reason)?;
    let applicant_name = normalize_required_text("applicant name", applicant.name.clone())?;
    let window_label = match (starts_at, ends_at) {
        (Some(start), Some(end)) => {
            format!("{} .. {}", format_timestamp(start), format_timestamp(end))
        }
        _ => duration_text.clone(),
    };

    let leave = ctx
        .data()
        .database()
        .create_leave_application(&NewLeaveApplication {
            guild_id: guild_id.get() as i64,
            applicant_user_id: applicant.id.get() as i64,
            applicant_name: applicant_name.clone(),
            duration_text,
            reason: reason.clone(),
            created_by_user_id: ctx.author().id.get() as i64,
            starts_at,
            ends_at,
            is_active: true,
        })
        .await?;

    let logged = match ctx.data().leave_log_channel(guild_id).await {
        Ok(channel_id) => {
            match logging::send_leave_application_log(ctx.serenity_context(), channel_id, &leave)
                .await
            {
                Ok(_) => true,
                Err(error) => {
                    tracing::error!(leave_id = leave.id, %error, "failed to send leave application log");
                    false
                }
            }
        }
        Err(error) => {
            tracing::error!(leave_id = leave.id, %error, "leave log channel is not configured");
            false
        }
    };

    send_status(
        ctx,
        &settings,
        format!(
            "Created leave application #{} for <@{}> ({applicant_name}) with window {}.{}",
            leave.id,
            applicant.id.get(),
            window_label,
            if logged {
                ""
            } else {
                " Leave log delivery failed."
            }
        ),
    )
    .await
}

/// List all active leave applications in this server.
#[poise::command(prefix_command, slash_command, guild_only)]
pub async fn active(ctx: Context<'_>) -> Result<(), Error> {
    let (guild_id, settings) = guild_settings(ctx).await?;
    require_moderator(ctx, &settings, Permissions::MANAGE_MESSAGES).await?;

    let applications = ctx
        .data()
        .database()
        .list_active_leave_applications(guild_id.get() as i64)
        .await?;

    if applications.is_empty() {
        send_status(ctx, &settings, "No active leave applications found.").await?;
        return Ok(());
    }

    send_leave_application_list(ctx, &settings, "Active leave applications", &applications).await
}

/// List all leave applications for a specific user.
#[poise::command(prefix_command, slash_command, guild_only)]
pub async fn user(
    ctx: Context<'_>,
    #[description = "Target user"] applicant: User,
) -> Result<(), Error> {
    let (guild_id, settings) = guild_settings(ctx).await?;
    require_moderator(ctx, &settings, Permissions::MANAGE_MESSAGES).await?;

    let applications = ctx
        .data()
        .database()
        .list_leave_applications_for_user(guild_id.get() as i64, applicant.id.get() as i64)
        .await?;

    if applications.is_empty() {
        send_status(
            ctx,
            &settings,
            format!("No leave applications found for <@{}>.", applicant.id.get()),
        )
        .await?;
        return Ok(());
    }

    send_leave_application_list(
        ctx,
        &settings,
        &format!("Leave applications for <@{}>", applicant.id.get()),
        &applications,
    )
    .await
}

fn normalize_required_text(field: &'static str, value: String) -> Result<String, Error> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(AppError::InvalidInput {
            message: format!("{field} must not be empty"),
        }
        .into());
    }

    Ok(trimmed.to_string())
}

fn format_leave_application(leave: &LeaveApplicationRecord) -> String {
    let status = if leave.is_active {
        "active"
    } else {
        "inactive"
    };
    let window = match (leave.starts_at, leave.ends_at) {
        (Some(start), Some(end)) => {
            format!(
                "window: {} .. {}",
                format_timestamp(start),
                format_timestamp(end)
            )
        }
        _ => format!("duration: {}", single_line(&leave.duration_text)),
    };
    let reason = single_line(&leave.reason);

    format!(
        "#{} | <@{}> | {} | {} | {} | reason: {}",
        leave.id,
        leave.applicant_user_id,
        single_line(&leave.applicant_name),
        status,
        window,
        reason
    )
}

fn single_line(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn chunk_lines(header: &str, lines: &[String]) -> Vec<String> {
    const MAX_LEN: usize = 1_800;
    let mut chunks = Vec::new();
    let mut current = String::new();

    for line in lines {
        let candidate = if current.is_empty() {
            format!("{header}\n{line}")
        } else {
            format!("{current}\n{line}")
        };

        if candidate.len() > MAX_LEN && !current.is_empty() {
            chunks.push(current);
            current = format!("{header}\n{line}");
        } else {
            current = candidate;
        }
    }

    if !current.is_empty() {
        chunks.push(current);
    }

    chunks
}

async fn send_leave_application_list(
    ctx: Context<'_>,
    settings: &crate::config::RuntimeGuildSettings,
    header: &str,
    applications: &[LeaveApplicationRecord],
) -> Result<(), Error> {
    let lines = applications
        .iter()
        .map(format_leave_application)
        .collect::<Vec<_>>();
    let chunks = chunk_lines(header, &lines);
    let ephemeral =
        matches!(ctx, poise::Context::Application(_)) && settings.ephemeral_slash_responses;

    for chunk in chunks {
        ctx.send(CreateReply::default().content(chunk).ephemeral(ephemeral))
            .await?;
    }

    Ok(())
}
