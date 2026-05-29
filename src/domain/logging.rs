use poise::serenity_prelude::{ChannelId, Context, CreateEmbed, CreateMessage, Message};

use crate::{
    db::models::{LeaveApplicationRecord, ModerationCaseRecord},
    util::{format_duration, format_timestamp},
};

pub async fn send_case_log(
    ctx: &Context,
    channel_id: ChannelId,
    case: &ModerationCaseRecord,
) -> Result<Message, serenity::Error> {
    channel_id
        .send_message(&ctx.http, CreateMessage::new().embed(case_embed(case)))
        .await
}

pub fn case_embed(case: &ModerationCaseRecord) -> CreateEmbed {
    let mut embed = CreateEmbed::new()
        .title(format!("{} Case #{}", title_for(case), case.id))
        .field("Moderator", format!("<@{}>", case.moderator_user_id), true)
        .field(
            "Target",
            case.target_user_id
                .map(|user_id| format!("<@{}>", user_id))
                .unwrap_or_else(|| "N/A".into()),
            true,
        )
        .field("Created", format_timestamp(case.created_at), false)
        .field(
            "Reason",
            case.reason
                .clone()
                .unwrap_or_else(|| "No reason provided".into()),
            false,
        );

    if let Some(message_id) = case.message_id {
        embed = embed.field("Message ID", message_id.to_string(), true);
    }

    if let Some(duration_seconds) = case.duration_seconds {
        embed = embed.field("Duration", format_duration(duration_seconds), true);
    }

    if let Some(expires_at) = case.expires_at {
        embed = embed.field("Expires", format_timestamp(expires_at), true);
    }

    if let Some(details) = &case.details {
        embed = embed.field("Details", details, false);
    }

    embed
}

pub async fn send_leave_application_log(
    ctx: &Context,
    channel_id: ChannelId,
    leave: &LeaveApplicationRecord,
) -> Result<Message, serenity::Error> {
    channel_id
        .send_message(&ctx.http, CreateMessage::new().embed(leave_application_embed(leave)))
        .await
}

pub fn leave_application_embed(leave: &LeaveApplicationRecord) -> CreateEmbed {
    let mut embed = CreateEmbed::new()
        .title(format!("Leave Application #{}", leave.id))
        .field("Applicant", format!("<@{}>", leave.applicant_user_id), true)
        .field("Applicant Name", leave.applicant_name.clone(), true)
        .field("Created By", format!("<@{}>", leave.created_by_user_id), true)
        .field("Created", format_timestamp(leave.created_at), false)
        .field("Reason", leave.reason.clone(), false);

    if let (Some(start), Some(end)) = (leave.starts_at, leave.ends_at) {
        embed = embed.field(
            "Window",
            format!("{} .. {}", format_timestamp(start), format_timestamp(end)),
            true,
        );
    } else {
        embed = embed.field("Duration", leave.duration_text.clone(), true);
    }

    embed = embed.field(
        "Status",
        if leave.is_active { "Active" } else { "Inactive" },
        true,
    );

    embed
}

fn title_for(case: &ModerationCaseRecord) -> &'static str {
    match case.action_type.as_str() {
        "warn" => "Warn",
        "timeout" => "Timeout",
        "kick" => "Kick",
        "ban" => "Ban",
        "unban" => "Unban",
        "purge" => "Purge",
        "note" => "Note",
        _ => "Moderation",
    }
}
