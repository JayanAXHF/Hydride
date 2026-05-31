use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use url::Url;

use crate::error::AppError;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub git: GitConfig,
    #[serde(default)]
    pub cliff: CliffConfig,
    #[serde(default)]
    pub discord: DiscordConfig,
    #[serde(default)]
    pub output: OutputConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GitConfig {
    #[serde(default = "default_repo_path")]
    pub repo_path: PathBuf,
    pub range: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CliffConfig {
    #[serde(default = "default_cliff_path")]
    pub config_path: PathBuf,
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct DiscordConfig {
    #[serde(default)]
    pub webhook_url: String,
    #[serde(default)]
    pub root_message_id: String,
    #[serde(default)]
    pub overflow_message_ids: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OutputConfig {
    #[serde(default = "default_max_content_chars")]
    pub max_content_chars: usize,
    #[serde(default)]
    pub heading: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ResolvedConfig {
    pub repo_path: PathBuf,
    pub cliff_config_path: PathBuf,
    pub discord: ResolvedDiscordConfig,
    pub output: OutputConfig,
    pub range: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ResolvedDiscordConfig {
    pub webhook_url: Url,
    pub root_message_id: String,
    pub overflow_message_ids: Vec<String>,
}

impl Default for GitConfig {
    fn default() -> Self {
        Self {
            repo_path: default_repo_path(),
            range: None,
        }
    }
}

impl Default for CliffConfig {
    fn default() -> Self {
        Self {
            config_path: default_cliff_path(),
        }
    }
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            max_content_chars: default_max_content_chars(),
            heading: None,
        }
    }
}

impl AppConfig {
    pub fn load(path: impl AsRef<Path>) -> Result<(Self, PathBuf), AppError> {
        let path = path.as_ref();
        let contents = fs::read_to_string(path).map_err(|source| AppError::ReadConfig {
            path: path.display().to_string(),
            source,
        })?;
        let config: Self = toml::from_str(&contents).map_err(|source| AppError::ParseConfig {
            path: path.display().to_string(),
            source,
        })?;
        Ok((config, path.to_path_buf()))
    }

    pub fn resolve(
        &self,
        source_path: &Path,
        webhook_override: Option<&str>,
    ) -> Result<ResolvedConfig, AppError> {
        self.validate(webhook_override)?;
        let base_dir = source_path.parent().unwrap_or_else(|| Path::new("."));
        let repo_path = resolve_relative(base_dir, &self.git.repo_path);
        if !repo_path.exists() {
            return Err(AppError::InvalidConfig(format!(
                "git.repo_path does not exist: {}",
                repo_path.display()
            )));
        }

        let cliff_config_path = resolve_relative(base_dir, &self.cliff.config_path);
        let webhook_url = Url::parse(webhook_override.unwrap_or(&self.discord.webhook_url))
            .map_err(|e| AppError::InvalidWebhook(e.to_string()))?;
        validate_webhook_url(&webhook_url)?;

        Ok(ResolvedConfig {
            repo_path,
            cliff_config_path,
            discord: ResolvedDiscordConfig {
                webhook_url,
                root_message_id: self.discord.root_message_id.clone(),
                overflow_message_ids: self.discord.overflow_message_ids.clone(),
            },
            output: self.output.clone(),
            range: self.git.range.clone(),
        })
    }

    fn validate(&self, webhook_override: Option<&str>) -> Result<(), AppError> {
        if webhook_override.is_none() && self.discord.webhook_url.trim().is_empty() {
            return Err(AppError::InvalidConfig(String::from(
                "discord.webhook_url must not be empty",
            )));
        }
        if self.discord.root_message_id.trim().is_empty() {
            return Err(AppError::InvalidConfig(String::from(
                "discord.root_message_id must not be empty",
            )));
        }
        validate_message_id(&self.discord.root_message_id)?;
        for id in &self.discord.overflow_message_ids {
            validate_message_id(id)?;
        }
        if self.output.max_content_chars == 0 || self.output.max_content_chars > 2000 {
            return Err(AppError::InvalidConfig(String::from(
                "output.max_content_chars must be between 1 and 2000",
            )));
        }
        Ok(())
    }
}

fn resolve_relative(base: &Path, value: &Path) -> PathBuf {
    if value.is_absolute() {
        value.to_path_buf()
    } else {
        base.join(value)
    }
}

fn validate_message_id(id: &str) -> Result<(), AppError> {
    if id.trim().parse::<u64>().is_err() {
        return Err(AppError::InvalidConfig(format!(
            "Discord message id is not a valid snowflake: {id}"
        )));
    }
    Ok(())
}

fn validate_webhook_url(url: &Url) -> Result<(), AppError> {
    let host = url
        .host_str()
        .ok_or_else(|| AppError::InvalidWebhook(String::from("missing host")))?;
    let valid_host =
        host == "discord.com" || host == "discordapp.com" || host.ends_with(".discord.com");
    if !valid_host {
        return Err(AppError::InvalidWebhook(format!(
            "unsupported Discord host: {host}"
        )));
    }
    let mut segments = url
        .path_segments()
        .ok_or_else(|| AppError::InvalidWebhook(String::from("missing webhook path")))?;
    if segments.next().is_none() || segments.next().is_none() {
        return Err(AppError::InvalidWebhook(String::from(
            "webhook url must contain webhook id and token",
        )));
    }
    Ok(())
}

fn default_repo_path() -> PathBuf {
    PathBuf::from(".")
}

fn default_cliff_path() -> PathBuf {
    PathBuf::from("cliff.toml")
}

fn default_max_content_chars() -> usize {
    1900
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn resolves_relative_paths() {
        let tempdir = TempDir::new().expect("tempdir");
        let repo_dir = tempdir.path().join("repo");
        std::fs::create_dir(&repo_dir).expect("repo dir");

        let config = AppConfig {
            git: GitConfig {
                repo_path: PathBuf::from("repo"),
                range: None,
            },
            cliff: CliffConfig {
                config_path: PathBuf::from("cliff.toml"),
            },
            discord: DiscordConfig {
                webhook_url: String::from("https://discord.com/api/webhooks/1/token"),
                root_message_id: String::from("1"),
                overflow_message_ids: vec![],
            },
            output: OutputConfig::default(),
        };

        let config_path = tempdir.path().join("config.toml");
        let resolved = config
            .resolve(&config_path, None)
            .expect("config should resolve");
        assert_eq!(resolved.repo_path, repo_dir);
        assert_eq!(
            resolved.cliff_config_path,
            tempdir.path().join("cliff.toml")
        );
        assert_eq!(resolved.discord.root_message_id, "1");
    }

    #[test]
    fn allows_webhook_override() {
        let tempdir = TempDir::new().expect("tempdir");
        let repo_dir = tempdir.path().join("repo");
        std::fs::create_dir(&repo_dir).expect("repo dir");

        let config = AppConfig {
            git: GitConfig {
                repo_path: PathBuf::from("repo"),
                range: None,
            },
            cliff: CliffConfig {
                config_path: PathBuf::from("cliff.toml"),
            },
            discord: DiscordConfig {
                webhook_url: String::new(),
                root_message_id: String::from("1"),
                overflow_message_ids: vec![],
            },
            output: OutputConfig::default(),
        };

        let config_path = tempdir.path().join("config.toml");
        let resolved = config
            .resolve(
                &config_path,
                Some("https://discord.com/api/webhooks/1/token"),
            )
            .expect("config should resolve");

        assert_eq!(
            resolved.discord.webhook_url.as_str(),
            "https://discord.com/api/webhooks/1/token"
        );
    }
}
