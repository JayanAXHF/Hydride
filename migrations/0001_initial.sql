CREATE TABLE IF NOT EXISTS guild_settings (
    guild_id INTEGER PRIMARY KEY NOT NULL,
    log_channel_id INTEGER,
    require_reason INTEGER NOT NULL DEFAULT 1,
    ephemeral_slash_responses INTEGER NOT NULL DEFAULT 1,
    notes_enabled INTEGER NOT NULL DEFAULT 0,
    appeals_enabled INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS guild_mod_roles (
    guild_id INTEGER NOT NULL,
    role_id INTEGER NOT NULL,
    PRIMARY KEY (guild_id, role_id),
    FOREIGN KEY (guild_id) REFERENCES guild_settings(guild_id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS moderation_cases (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    guild_id INTEGER NOT NULL,
    action_type TEXT NOT NULL,
    target_user_id INTEGER,
    moderator_user_id INTEGER NOT NULL,
    reason TEXT,
    duration_seconds INTEGER,
    details TEXT,
    created_at INTEGER NOT NULL,
    expires_at INTEGER,
    audit_log_channel_id INTEGER,
    audit_log_message_id INTEGER,
    FOREIGN KEY (guild_id) REFERENCES guild_settings(guild_id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS case_notes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    case_id INTEGER NOT NULL,
    author_user_id INTEGER NOT NULL,
    content TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    FOREIGN KEY (case_id) REFERENCES moderation_cases(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_cases_guild_created_at
    ON moderation_cases (guild_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_cases_target_created_at
    ON moderation_cases (target_user_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_cases_moderator_created_at
    ON moderation_cases (moderator_user_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_cases_action_created_at
    ON moderation_cases (action_type, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_case_notes_case_created_at
    ON case_notes (case_id, created_at ASC);
