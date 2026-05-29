use std::str::FromStr;

use crate::error::AppError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModerationActionType {
    Warn,
    Timeout,
    Kick,
    Ban,
    Unban,
    Purge,
    Note,
}

impl ModerationActionType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Warn => "warn",
            Self::Timeout => "timeout",
            Self::Kick => "kick",
            Self::Ban => "ban",
            Self::Unban => "unban",
            Self::Purge => "purge",
            Self::Note => "note",
        }
    }
}

impl FromStr for ModerationActionType {
    type Err = AppError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "warn" => Ok(Self::Warn),
            "timeout" => Ok(Self::Timeout),
            "kick" => Ok(Self::Kick),
            "ban" => Ok(Self::Ban),
            "unban" => Ok(Self::Unban),
            "purge" => Ok(Self::Purge),
            "note" => Ok(Self::Note),
            _ => Err(AppError::GuildOnly), // this should not be possible, as this is only used when getting cases from the db
        }
    }
}

#[derive(Debug, Clone)]
pub struct NewModerationCase {
    pub guild_id: i64,
    pub action_type: ModerationActionType,
    pub target_user_id: Option<i64>,
    pub moderator_user_id: i64,
    pub message_id: Option<i64>,
    pub reason: Option<String>,
    pub duration_seconds: Option<i64>,
    pub details: Option<String>,
    pub expires_at: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct NewLeaveApplication {
    pub guild_id: i64,
    pub applicant_user_id: i64,
    pub applicant_name: String,
    pub duration_text: String,
    pub reason: String,
    pub created_by_user_id: i64,
    pub starts_at: Option<i64>,
    pub ends_at: Option<i64>,
    pub is_active: bool,
}
