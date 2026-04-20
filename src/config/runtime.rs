use crate::db::models::GuildSettingsRecord;

#[derive(Debug, Clone)]
pub struct RuntimeGuildSettingsDefaults {
    pub log_channel_id: Option<u64>,
    pub require_reason: bool,
    pub ephemeral_slash_responses: bool,
    pub notes_enabled: bool,
    pub appeals_enabled: bool,
}

#[derive(Debug, Clone)]
pub struct RuntimeGuildSettings {
    #[allow(dead_code)]
    pub guild_id: u64,
    pub log_channel_id: Option<u64>,
    pub require_reason: bool,
    pub ephemeral_slash_responses: bool,
    pub notes_enabled: bool,
    pub appeals_enabled: bool,
    pub mod_role_ids: Vec<u64>,
}

impl RuntimeGuildSettings {
    pub fn from_record(record: GuildSettingsRecord, mod_role_ids: Vec<u64>) -> Self {
        Self {
            guild_id: record.guild_id as u64,
            log_channel_id: record.log_channel_id.map(|value| value as u64),
            require_reason: record.require_reason,
            ephemeral_slash_responses: record.ephemeral_slash_responses,
            notes_enabled: record.notes_enabled,
            appeals_enabled: record.appeals_enabled,
            mod_role_ids,
        }
    }
}
