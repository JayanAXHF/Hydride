use reqwest::StatusCode;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("failed to read config from {path}: {source}")]
    ReadConfig {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse TOML config from {path}: {source}")]
    ParseConfig {
        path: String,
        #[source]
        source: toml::de::Error,
    },
    #[error("invalid config: {0}")]
    InvalidConfig(String),
    #[error("failed to load git-cliff config from {path}: {source}")]
    CliffConfig {
        path: String,
        #[source]
        source: git_cliff_core::error::Error,
    },
    #[error("failed to inspect git repository at {path}: {source}")]
    Repository {
        path: String,
        #[source]
        source: git_cliff_core::error::Error,
    },
    #[error("failed to generate changelog: {source}")]
    Changelog {
        #[source]
        source: git_cliff_core::error::Error,
    },
    #[error("generated changelog is not valid UTF-8: {source}")]
    Utf8 {
        #[source]
        source: std::string::FromUtf8Error,
    },
    #[error("failed to build Discord request: {source}")]
    HttpBuild {
        #[source]
        source: reqwest::Error,
    },
    #[error("Discord request failed with {status}: {body}")]
    DiscordApi { status: StatusCode, body: String },
    #[error("failed to send Discord request: {source}")]
    Http {
        #[source]
        source: reqwest::Error,
    },
    #[error("failed to decode Discord response: {source}")]
    DiscordResponse {
        #[source]
        source: reqwest::Error,
    },
    #[error("failed to parse Discord JSON response: {source}")]
    DiscordJson {
        #[source]
        source: serde_json::Error,
    },
    #[error("invalid webhook url: {0}")]
    InvalidWebhook(String),
}
