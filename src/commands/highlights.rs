use crate::commands::{Context, Error, guild_id, guild_settings, send_status};
use poise::CreateReply;
use regex::RegexBuilder;
use serenity::all::CreateEmbed;

#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    subcommands("add", "remove", "list", "test"),
    subcommand_required
)]
pub async fn highlight(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(prefix_command, slash_command, guild_only, rename = "add")]
pub async fn add(
    ctx: Context<'_>,
    #[description = "Regex pattern to watch for"]
    #[rest]
    pattern: String,
) -> Result<(), Error> {
    let guild_id = guild_id(ctx).await?;
    let (_, settings) = guild_settings(ctx).await?;
    let author_id = ctx.author().id.get() as i64;

    // Validate regex pattern
    let mut builder = RegexBuilder::new(&pattern);
    builder.size_limit(1 << 16);
    builder.dfa_size_limit(1 << 16);
    if let Err(e) = builder.build() {
        send_status(ctx, &settings, format!("Invalid regex pattern: {e}")).await?;
        return Ok(());
    }

    let db = ctx.data().highlights_database();

    // Check count limit
    let count = db
        .count_highlights(guild_id.get() as i64, author_id)
        .await?;
    if count >= crate::domain::highlights::MAX_HIGHLIGHTS_PER_USER {
        send_status(
            ctx,
            &settings,
            format!(
                "You have reached the maximum limit of {} highlights in this server.",
                crate::domain::highlights::MAX_HIGHLIGHTS_PER_USER
            ),
        )
        .await?;
        return Ok(());
    }

    match db
        .add_highlight(guild_id.get() as i64, author_id, &pattern)
        .await?
    {
        None => {
            send_status(
                ctx,
                &settings,
                format!(
                    "The pattern `{}` is already tracked for you in this server.",
                    pattern
                ),
            )
            .await?;
        }
        Some(record) => {
            crate::domain::highlights::refresh_guild_cache(
                db,
                ctx.data().highlight_cache(),
                guild_id,
            )
            .await?;
            send_status(
                ctx,
                &settings,
                format!("Added highlight #{} for pattern `{}`.", record.id, pattern),
            )
            .await?;
        }
    }

    Ok(())
}

#[poise::command(prefix_command, slash_command, guild_only, rename = "remove")]
pub async fn remove(
    ctx: Context<'_>,
    #[description = "Highlight ID (from /highlight list)"] id: i64,
) -> Result<(), Error> {
    let guild_id = guild_id(ctx).await?;
    let (_, settings) = guild_settings(ctx).await?;
    let author_id = ctx.author().id.get() as i64;

    let db = ctx.data().highlights_database();
    let deleted = db
        .remove_highlight(guild_id.get() as i64, author_id, id)
        .await?;

    if deleted {
        crate::domain::highlights::refresh_guild_cache(db, ctx.data().highlight_cache(), guild_id)
            .await?;
        send_status(ctx, &settings, format!("Removed highlight #{}.", id)).await?;
    } else {
        send_status(
            ctx,
            &settings,
            format!(
                "No highlight with ID {} was found for you in this server.",
                id
            ),
        )
        .await?;
    }

    Ok(())
}

#[poise::command(prefix_command, slash_command, guild_only, rename = "list")]
pub async fn list(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = guild_id(ctx).await?;
    let (_, settings) = guild_settings(ctx).await?;
    let author_id = ctx.author().id.get() as i64;

    let db = ctx.data().highlights_database();
    let records = db.list_highlights(guild_id.get() as i64, author_id).await?;

    if records.is_empty() {
        send_status(ctx, &settings, "You have no highlights set in this server.").await?;
        return Ok(());
    }

    let mut description = String::new();
    for record in records {
        description.push_str(&format!("**[{}]** `{}`\n", record.id, record.pattern));
    }

    let embed = CreateEmbed::new()
        .title("Your Highlights")
        .description(description)
        .color(0x3498db);

    let ephemeral =
        matches!(ctx, poise::Context::Application(_)) && settings.ephemeral_slash_responses;
    ctx.send(CreateReply::default().embed(embed).ephemeral(ephemeral))
        .await?;

    Ok(())
}

#[poise::command(prefix_command, slash_command, guild_only, rename = "test")]
pub async fn test(
    ctx: Context<'_>,
    #[description = "Sample text to test your patterns against"]
    #[rest]
    text: String,
) -> Result<(), Error> {
    let guild_id = guild_id(ctx).await?;
    let (_, settings) = guild_settings(ctx).await?;
    let author_id = ctx.author().id.get() as i64;

    let db = ctx.data().highlights_database();
    let records = db.list_highlights(guild_id.get() as i64, author_id).await?;

    if records.is_empty() {
        send_status(ctx, &settings, "You have no highlights set in this server.").await?;
        return Ok(());
    }

    let mut matched_patterns = Vec::new();
    for record in records {
        let mut builder = RegexBuilder::new(&record.pattern);
        builder.size_limit(1 << 16);
        builder.dfa_size_limit(1 << 16);
        if builder.build().map(|re| re.is_match(&text)).unwrap_or(false) {
            matched_patterns.push(record.pattern);
        }
    }

    if matched_patterns.is_empty() {
        send_status(
            ctx,
            &settings,
            "None of your highlight patterns matched the sample text.",
        )
        .await?;
    } else {
        let list_str = matched_patterns
            .iter()
            .map(|pat| format!("`{}`", pat))
            .collect::<Vec<_>>()
            .join("\n");
        send_status(
            ctx,
            &settings,
            format!("The following highlight patterns matched:\n{}", list_str),
        )
        .await?;
    }

    Ok(())
}
