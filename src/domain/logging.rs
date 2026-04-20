use poise::serenity_prelude::{ChannelId, Context, CreateEmbed, CreateMessage, Message};

use crate::{
    db::models::ModerationCaseRecord,
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
