use std::path::PathBuf;

use snafu::Snafu;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum AppError {
    #[snafu(display("failed to read config from {}: {}", path.display(), source))]
    ConfigRead {
        path: PathBuf,
        source: std::io::Error,
    },
    #[snafu(display("failed to parse config from {}: {}", path.display(), source))]
    ConfigParse {
        path: PathBuf,
        source: toml::de::Error,
    },
    #[snafu(display("invalid config: {message}"))]
    InvalidConfig { message: String },
    #[snafu(display("database error: {source}"))]
    Database { source: sqlx::Error },
    #[snafu(display("database migration error: {source}"))]
    DatabaseMigration { source: sqlx::migrate::MigrateError },
    #[snafu(display("database url error: {source}"))]
    DatabaseUrl { source: sqlx::Error },
    #[snafu(display("discord API error: {source}"))]
    Discord { source: Box<serenity::Error> },
    #[snafu(display("this command can only be used in a guild"))]
    GuildOnly,
    #[snafu(display("permission denied: {message}"))]
    PermissionDenied { message: String },
    #[snafu(display("bot permission denied: {message}"))]
    BotPermissionDenied { message: String },
    #[snafu(display("role hierarchy prevents this action: {message}"))]
    RoleHierarchy { message: String },
    #[snafu(display("missing moderation log channel for guild {guild_id}"))]
    MissingLogChannel { guild_id: u64 },
    #[snafu(display("{entity} not found: {id}"))]
    NotFound { entity: &'static str, id: String },
    #[snafu(display("invalid input: {message}"))]
    InvalidInput { message: String },
}
