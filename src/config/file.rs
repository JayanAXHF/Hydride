use std::{fs, path::Path};

use serde::Deserialize;
use snafu::ResultExt;

use crate::error::{AppError, AppResult, ConfigParseSnafu, ConfigReadSnafu};

#[derive(Debug, Clone, Deserialize)]
pub struct BootstrapConfig {
    pub discord: DiscordConfig,
    pub database: DatabaseConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
    #[serde(default)]
    pub moderation: ModerationConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DiscordConfig {
    pub token: String,
    #[serde(default = "default_prefix")]
    pub prefix: String,
    #[serde(default)]
    pub owner_ids: Vec<u64>,
    #[serde(default)]
    pub dev_guild_ids: Vec<u64>,
    #[serde(default = "default_register_globally")]
    pub register_globally: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    #[serde(default = "default_database_url")]
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_filter")]
    pub filter: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModerationConfig {
    #[serde(default)]
    pub default_log_channel_id: Option<u64>,
    #[serde(default = "default_require_reason")]
    pub require_reason: bool,
    #[serde(default = "default_ephemeral_slash_responses")]
    pub ephemeral_slash_responses: bool,
    #[serde(default = "default_max_case_results")]
    pub max_case_results: u8,
}

impl BootstrapConfig {
    pub fn load(path: &Path) -> AppResult<Self> {
        let raw = fs::read_to_string(path).context(ConfigReadSnafu {
            path: path.to_path_buf(),
        })?;
        let config: Self = toml::from_str(&raw).context(ConfigParseSnafu {
            path: path.to_path_buf(),
        })?;
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> AppResult<()> {
        if self.discord.token.trim().is_empty() {
            return Err(AppError::InvalidConfig {
                message: "discord.token must not be empty".into(),
            });
        }

        if self.discord.prefix.trim().is_empty() {
            return Err(AppError::InvalidConfig {
                message: "discord.prefix must not be empty".into(),
            });
        }

        if !self.discord.register_globally && self.discord.dev_guild_ids.is_empty() {
            return Err(AppError::InvalidConfig {
                message:
                    "set discord.register_globally = true or provide at least one discord.dev_guild_ids entry"
                        .to_string(),
            });
        }

        if self.moderation.max_case_results == 0 {
            return Err(AppError::InvalidConfig {
                message: "moderation.max_case_results must be greater than zero".into(),
            });
        }

        Ok(())
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            filter: default_log_filter(),
        }
    }
}

impl Default for ModerationConfig {
    fn default() -> Self {
        Self {
            default_log_channel_id: None,
            require_reason: default_require_reason(),
            ephemeral_slash_responses: default_ephemeral_slash_responses(),
            max_case_results: default_max_case_results(),
        }
    }
}

fn default_prefix() -> String {
    "!".into()
}

fn default_register_globally() -> bool {
    false
}

fn default_database_url() -> String {
    "sqlite://moderation_bot.db".into()
}

fn default_log_filter() -> String {
    "info,serenity=warn,sqlx=warn".into()
}

fn default_require_reason() -> bool {
    true
}

fn default_ephemeral_slash_responses() -> bool {
    true
}

fn default_max_case_results() -> u8 {
    10
}
