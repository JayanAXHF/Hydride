use poise::serenity_prelude::{ChannelId, Context, CreateEmbed, CreateMessage, Message};
use serenity::all::Colour;

use crate::{
    bar,
    db::models::{LeaveApplicationRecord, ModerationCaseRecord},
    stats::{self, ChannelSnapshot},
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
        .send_message(
            &ctx.http,
            CreateMessage::new().embed(leave_application_embed(leave)),
        )
        .await
}

#[allow(dead_code)]
pub async fn send_channel_stats(
    ctx: &Context,
    channel_id: ChannelId,
    snap: &ChannelSnapshot,
) -> Result<Message, serenity::Error> {
    channel_id
        .send_message(
            &ctx.http,
            CreateMessage::new().embed(channel_stats_embed(snap)),
        )
        .await
}

pub fn channel_stats_embed(snap: &ChannelSnapshot) -> CreateEmbed {
    let stats = stats::compute(snap);
    let sorted_daily = sorted_f64(&snap.daily_counts);
    let p25 = stats::percentile(&sorted_daily, 0.25);
    let p75 = stats::percentile(&sorted_daily, 0.75);

    let channel_name = display_channel_name(snap);
    let mut embed = CreateEmbed::new()
        .title(format!("📊 {channel_name} · {}d window", snap.window_days))
        .colour(activity_colour(stats.mean_per_day))
        .field(
            "Volume (mean ± σ)",
            format!(
                "{} msg/day ± {}",
                code(format!("{:.1}", stats.mean_per_day)),
                code(format!("{:.1}", stats.stddev_per_day)),
            ),
            true,
        )
        .field(
            "Range",
            format!(
                "min {} · max {}",
                code(stats.min_per_day),
                code(stats.max_per_day),
            ),
            true,
        )
        .field(
            "Trend",
            format!(
                "{} {} msg/day²",
                trend_arrow(stats.trend_slope),
                code(format!("{:+.1}", stats.trend_slope)),
            ),
            true,
        )
        .field(
            "Hourly activity",
            format!(
                "{}\nPeak: {} UTC",
                bar::hourly_spark(&snap.hourly_buckets),
                code(format!("{:02}:00", stats.peak_hour_utc)),
            ),
            false,
        )
        .field(
            "Daily trend (last N days)",
            bar::daily_spark(&snap.daily_counts, &stats.outlier_days),
            false,
        )
        .field(
            "Message length",
            format!(
                "avg {} chars ± {} · med {}",
                code(format!("{:.0}", stats.mean_length.round())),
                code(format!("{:.0}", stats.stddev_length.round())),
                code(format!("{:.0}", stats.median_length.round())),
            ),
            true,
        )
        .field("Authors", top_poster_text(snap), true)
        .field("Role share", role_share_text(snap), true)
        .field(
            "Concentration (Gini)",
            format!(
                "{} · {}",
                code(format!("{:.2}", stats.gini)),
                gini_label(stats.gini),
            ),
            true,
        )
        .field(
            "Percentiles",
            format!(
                "p5 {} · p25 {} · p50 {} · p75 {} · p95 {}",
                code(format!("{:.0}", stats.p5_per_day.round())),
                code(format!("{:.0}", p25.round())),
                code(format!("{:.0}", stats.median_per_day.round())),
                code(format!("{:.0}", p75.round())),
                code(format!("{:.0}", stats.p95_per_day.round())),
            ),
            false,
        );

    if !stats.outlier_days.is_empty() {
        embed = embed.field(
            "Outlier days",
            format!(
                "{} days flagged (>mean + 2σ)",
                code(stats.outlier_days.len()),
            ),
            false,
        );
    }

    embed
}

pub fn leave_application_embed(leave: &LeaveApplicationRecord) -> CreateEmbed {
    let mut embed = CreateEmbed::new()
        .title(format!("Leave Application #{}", leave.id))
        .field("Applicant", format!("<@{}>", leave.applicant_user_id), true)
        .field("Applicant Name", leave.applicant_name.clone(), true)
        .field(
            "Created By",
            format!("<@{}>", leave.created_by_user_id),
            true,
        )
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
        if leave.is_active {
            "Active"
        } else {
            "Inactive"
        },
        true,
    );

    embed
}

fn code(value: impl std::fmt::Display) -> String {
    format!("`{value}`")
}

fn display_channel_name(snap: &ChannelSnapshot) -> String {
    let name = snap.channel_name.trim().trim_start_matches('#');
    if name.is_empty() {
        format!("#{}", snap.channel_id)
    } else {
        format!("#{name}")
    }
}

fn top_poster_text(snap: &ChannelSnapshot) -> String {
    let unique_authors = code(snap.unique_authors);
    match snap.top_posters.first() {
        Some((user_id, count)) => format!(
            "{} unique · Top: <@{}> ({} msgs)",
            unique_authors,
            user_id,
            code(*count),
        ),
        None => format!("{unique_authors} unique · Top: none"),
    }
}

fn role_share_text(snap: &ChannelSnapshot) -> String {
    match &snap.role_share {
        Some(share) => format!(
            "<@&{}>: {} / {} msgs ({})",
            share.role_id,
            code(share.role_messages),
            code(share.total_messages),
            code(format!("{:.1}%", share.percentage)),
        ),
        None => "not calculated".into(),
    }
}

fn activity_colour(mean_per_day: f64) -> Colour {
    let colour = if mean_per_day < 10.0 {
        0x95a5a6
    } else if mean_per_day < 50.0 {
        0x3498db
    } else if mean_per_day < 200.0 {
        0x2ecc71
    } else {
        0xf1c40f
    };

    Colour::new(colour)
}

fn gini_label(gini: f64) -> &'static str {
    if gini < 0.2 {
        "uniform"
    } else if gini < 0.5 {
        "moderate"
    } else {
        "high"
    }
}

fn trend_arrow(slope: f64) -> &'static str {
    if slope > 0.0 {
        "▲"
    } else if slope < 0.0 {
        "▼"
    } else {
        "•"
    }
}

fn sorted_f64(values: &[u32]) -> Vec<f64> {
    let mut sorted: Vec<f64> = values.iter().map(|&value| value as f64).collect();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    sorted
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
