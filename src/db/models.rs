use sqlx::FromRow;

#[allow(dead_code)]
#[derive(Debug, Clone, FromRow)]
pub struct GuildSettingsRecord {
    pub guild_id: i64,
    pub log_channel_id: Option<i64>,
    pub require_reason: bool,
    pub ephemeral_slash_responses: bool,
    pub notes_enabled: bool,
    pub appeals_enabled: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

#[allow(dead_code)]
#[derive(Debug, Clone, FromRow)]
pub struct ModerationCaseRecord {
    pub id: i64,
    pub guild_id: i64,
    pub action_type: String,
    pub target_user_id: Option<i64>,
    pub moderator_user_id: i64,
    pub message_id: Option<i64>,
    pub reason: Option<String>,
    pub duration_seconds: Option<i64>,
    pub details: Option<String>,
    pub created_at: i64,
    pub expires_at: Option<i64>,
    pub audit_log_channel_id: Option<i64>,
    pub audit_log_message_id: Option<i64>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, FromRow)]
pub struct CaseNoteRecord {
    pub id: i64,
    pub case_id: i64,
    pub author_user_id: i64,
    pub content: String,
    pub created_at: i64,
}
