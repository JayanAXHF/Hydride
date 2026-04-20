#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone)]
pub struct NewModerationCase {
    pub guild_id: i64,
    pub action_type: ModerationActionType,
    pub target_user_id: Option<i64>,
    pub moderator_user_id: i64,
    pub reason: Option<String>,
    pub duration_seconds: Option<i64>,
    pub details: Option<String>,
    pub expires_at: Option<i64>,
}
