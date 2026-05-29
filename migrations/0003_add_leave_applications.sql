CREATE TABLE IF NOT EXISTS leave_applications (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    guild_id INTEGER NOT NULL,
    applicant_user_id INTEGER NOT NULL,
    applicant_name TEXT NOT NULL,
    duration_text TEXT NOT NULL,
    reason TEXT NOT NULL,
    created_by_user_id INTEGER NOT NULL,
    starts_at INTEGER,
    ends_at INTEGER,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (guild_id) REFERENCES guild_settings(guild_id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_leave_applications_guild_active_created_at
    ON leave_applications (guild_id, is_active, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_leave_applications_guild_applicant_created_at
    ON leave_applications (guild_id, applicant_user_id, created_at DESC);
