use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use poise::serenity_prelude::{
    ChannelId, CreateActionRow, CreateButton, CreateEmbed, CreateMessage, GuildId, Permissions,
    UserId,
};
use regex::{RegexSet, RegexSetBuilder};

use crate::db::HighlightsDatabase;
use crate::db::highlights::models::HighlightRecord;
use crate::error::AppResult;
use crate::state::AppState;

pub const MAX_HIGHLIGHTS_PER_USER: i64 = 25;

/// One guild's compiled highlight set.
#[derive(Clone)]
pub struct GuildHighlights {
    /// Combined set used for the fast first-pass `matches()` scan.
    set: RegexSet,
    /// Parallel array: index i corresponds to pattern i in `set`.
    /// (owner, highlight_id, raw_pattern)
    entries: Vec<(UserId, i64, String)>,
}

impl GuildHighlights {
    pub fn empty() -> Self {
        Self {
            set: RegexSet::empty(),
            entries: Vec::new(),
        }
    }

    pub fn from_records(records: &[HighlightRecord]) -> Self {
        let mut patterns = Vec::new();
        let mut entries = Vec::new();

        for record in records {
            patterns.push(record.pattern.clone());
            entries.push((
                UserId::new(record.user_id as u64),
                record.id,
                record.pattern.clone(),
            ));
        }

        if patterns.is_empty() {
            return Self::empty();
        }

        // Try compiling all patterns.
        let mut builder = RegexSetBuilder::new(&patterns);
        builder.size_limit(1 << 16);
        builder.dfa_size_limit(1 << 16);
        match builder.build() {
            Ok(set) => Self { set, entries },
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    "Failed to compile RegexSet for guild. Attempting individual compilation fallback..."
                );
                // Rebuild by filtering out patterns that fail compilation individually.
                let mut valid_patterns = Vec::new();
                let mut valid_entries = Vec::new();
                for (pattern, entry) in patterns.into_iter().zip(entries) {
                    let mut r_builder = regex::RegexBuilder::new(&pattern);
                    r_builder.size_limit(1 << 16);
                    r_builder.dfa_size_limit(1 << 16);
                    match r_builder.build() {
                        Ok(_) => {
                            valid_patterns.push(pattern);
                            valid_entries.push(entry);
                        }
                        Err(err) => {
                            tracing::warn!(
                                %pattern,
                                %err,
                                "Discarding invalid highlight pattern in fallback compilation"
                            );
                        }
                    }
                }
                if valid_patterns.is_empty() {
                    Self::empty()
                } else {
                    let mut builder = RegexSetBuilder::new(&valid_patterns);
                    builder.size_limit(1 << 16);
                    builder.dfa_size_limit(1 << 16);
                    match builder.build() {
                        Ok(set) => Self {
                            set,
                            entries: valid_entries,
                        },
                        Err(err) => {
                            tracing::error!(
                                %err,
                                "Absolute failure to compile RegexSet even after filtering valid patterns"
                            );
                            Self::empty()
                        }
                    }
                }
            }
        }
    }

    /// Returns, for each user with at least one matching pattern, the list of matched raw patterns.
    pub fn find(&self, haystack: &str) -> HashMap<UserId, Vec<(i64, String)>> {
        let mut out: HashMap<UserId, Vec<(i64, String)>> = HashMap::new();
        for idx in self.set.matches(haystack).into_iter() {
            if let Some((user_id, id, pattern)) = self.entries.get(idx) {
                out.entry(*user_id)
                    .or_default()
                    .push((*id, pattern.clone()));
            }
        }
        out
    }
}

#[derive(Default)]
pub struct HighlightCache {
    guilds: HashMap<GuildId, GuildHighlights>,
}

pub type SharedHighlightCache = Arc<RwLock<HighlightCache>>;

impl HighlightCache {
    pub fn get_or_empty(&self, guild_id: GuildId) -> Option<&GuildHighlights> {
        self.guilds.get(&guild_id)
    }

    pub fn set_guild(&mut self, guild_id: GuildId, gh: GuildHighlights) {
        self.guilds.insert(guild_id, gh);
    }
}

pub async fn build_initial_cache(db: &HighlightsDatabase) -> AppResult<SharedHighlightCache> {
    let records = db.all_highlights().await?;
    let mut groups: HashMap<i64, Vec<HighlightRecord>> = HashMap::new();
    for r in records {
        groups.entry(r.guild_id).or_default().push(r);
    }

    let mut cache = HighlightCache::default();
    for (guild_id_i64, guild_records) in groups {
        let guild_id = GuildId::new(guild_id_i64 as u64);
        let gh = GuildHighlights::from_records(&guild_records);
        cache.set_guild(guild_id, gh);
    }

    Ok(Arc::new(RwLock::new(cache)))
}

pub async fn refresh_guild_cache(
    db: &HighlightsDatabase,
    cache: &SharedHighlightCache,
    guild_id: GuildId,
) -> AppResult<()> {
    let records = db.highlights_for_guild(guild_id.get() as i64).await?;
    let gh = GuildHighlights::from_records(&records);
    cache.write().await.set_guild(guild_id, gh);
    Ok(())
}

pub async fn process_message(
    ctx: &poise::serenity_prelude::Context,
    data: &AppState,
    message: &poise::serenity_prelude::Message,
) -> anyhow::Result<()> {
    if message.author.bot {
        return Ok(());
    }
    let Some(guild_id) = message.guild_id else {
        return Ok(());
    };
    if message.content.is_empty() {
        return Ok(());
    }

    let matches = {
        let cache = data.highlight_cache().read().await;
        match cache.get_or_empty(guild_id) {
            Some(gh) => gh.find(&message.content),
            None => return Ok(()),
        }
    }; // lock dropped before any awaits below

    if matches.is_empty() {
        return Ok(());
    }

    let message_link = message.link();

    for (user_id, hits) in matches {
        if user_id == message.author.id {
            continue; // don't notify users about their own messages
        }

        let ctx = ctx.clone();
        let channel_id = message.channel_id;
        let message_link = message_link.clone();
        let author_name = message.author.name.clone();
        let content_preview = preview(&message.content);

        tokio::spawn(async move {
            if let Err(error) = notify_user(
                &ctx,
                guild_id,
                channel_id,
                user_id,
                hits,
                &message_link,
                &author_name,
                &content_preview,
            )
            .await
            {
                tracing::debug!(%user_id, %error, "failed to send highlight DM");
            }
        });
    }

    Ok(())
}

fn preview(content: &str) -> String {
    let clean = content.replace('@', "@\u{200b}");
    let mut chars = clean.chars();
    let truncated: String = chars.by_ref().take(200).collect();
    if chars.next().is_some() {
        format!("{}…", truncated)
    } else {
        truncated
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn notify_user(
    ctx: &poise::serenity_prelude::Context,
    guild_id: GuildId,
    channel_id: ChannelId,
    user_id: UserId,
    hits: Vec<(i64, String)>,
    message_link: &str,
    author_name: &str,
    content_preview: &str,
) -> anyhow::Result<()> {
    // 1. Fetch the Member. If user left guild, bail.
    let member = guild_id.member(&ctx.http, user_id).await?;

    // 2. Fetch the guild and channels.
    let guild = guild_id.to_partial_guild(&ctx.http).await?;
    let channels = guild.channels(&ctx.http).await?;

    let channel = match channels.get(&channel_id) {
        Some(c) => c.clone(),
        None => {
            // It could be a thread not returned by channels() or we don't have it.
            // Let's try to fetch it directly from the API.
            match channel_id.to_channel(&ctx.http).await? {
                poise::serenity_prelude::Channel::Guild(c) => c,
                _ => return Ok(()),
            }
        }
    };

    // Check user permissions in the channel.
    let perms = guild.user_permissions_in(&channel, &member);
    if !perms.contains(Permissions::VIEW_CHANNEL) {
        return Ok(());
    }

    // Handle thread-specific checks.
    let is_thread = channel.kind == poise::serenity_prelude::ChannelType::PublicThread
        || channel.kind == poise::serenity_prelude::ChannelType::PrivateThread
        || channel.kind == poise::serenity_prelude::ChannelType::NewsThread;

    if is_thread {
        if let Some(parent_channel) = channel.parent_id.and_then(|id| channels.get(&id)) {
            let parent_perms = guild.user_permissions_in(parent_channel, &member);
            if !parent_perms.contains(Permissions::VIEW_CHANNEL) {
                return Ok(());
            }
        }
        if channel.kind == poise::serenity_prelude::ChannelType::PrivateThread {
            let has_manage_threads = perms.contains(Permissions::MANAGE_THREADS);
            if !has_manage_threads {
                let members = ctx.http.get_channel_thread_members(channel_id).await?;
                let is_member = members.iter().any(|m| m.user_id == user_id);
                if !is_member {
                    return Ok(());
                }
            }
        }
    }

    // 3. Build DM embed.
    let patterns_str = hits
        .iter()
        .map(|(_, pat)| format!("`{}`", pat))
        .collect::<Vec<_>>()
        .join(", ");

    let embed = CreateEmbed::new()
        .title("Highlight Triggered")
        .description(format!(
            "Your highlight pattern(s) {} matched a message in **{}**.",
            patterns_str, guild.name
        ))
        .field("Author", author_name, true)
        .field("Channel", format!("<#{}>", channel_id), true)
        .field("Message Preview", content_preview, false)
        .color(0x3498db);

    let button = CreateButton::new_link(message_link).label("Jump to message");
    let row = CreateActionRow::Buttons(vec![button]);

    let dm_channel = user_id.create_dm_channel(&ctx.http).await?;
    let _ = dm_channel
        .send_message(
            &ctx.http,
            CreateMessage::new().embed(embed).components(vec![row]),
        )
        .await;

    Ok(())
}
