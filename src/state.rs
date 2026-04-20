use std::sync::Arc;

use poise::serenity_prelude::{ChannelId, GuildId};

use crate::{
    config::{BootstrapConfig, RuntimeGuildSettings, RuntimeGuildSettingsDefaults},
    db::Database,
    error::{AppError, AppResult},
};

#[derive(Clone)]
pub struct AppState {
    config: Arc<BootstrapConfig>,
    database: Database,
}

impl AppState {
    pub fn new(config: Arc<BootstrapConfig>, database: Database) -> Self {
        Self { config, database }
    }

    pub fn config(&self) -> &Arc<BootstrapConfig> {
        &self.config
    }

    pub fn database(&self) -> &Database {
        &self.database
    }

    pub async fn guild_settings(&self, guild_id: GuildId) -> AppResult<RuntimeGuildSettings> {
        self.database
            .ensure_guild_settings(guild_id.get() as i64, self.guild_defaults())
            .await
    }

    pub fn guild_defaults(&self) -> RuntimeGuildSettingsDefaults {
        RuntimeGuildSettingsDefaults {
            log_channel_id: self.config.moderation.default_log_channel_id,
            require_reason: self.config.moderation.require_reason,
            ephemeral_slash_responses: self.config.moderation.ephemeral_slash_responses,
            notes_enabled: false,
            appeals_enabled: false,
        }
    }

    pub async fn audit_log_channel(&self, guild_id: GuildId) -> AppResult<ChannelId> {
        let settings = self.guild_settings(guild_id).await?;
        let channel_id = settings
            .log_channel_id
            .or(self.config.moderation.default_log_channel_id)
            .ok_or(AppError::MissingLogChannel {
                guild_id: guild_id.get(),
            })?;

        Ok(ChannelId::new(channel_id))
    }
}
