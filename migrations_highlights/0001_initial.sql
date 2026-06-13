CREATE TABLE IF NOT EXISTS highlights (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    guild_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    pattern TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    UNIQUE (guild_id, user_id, pattern)
);

CREATE INDEX IF NOT EXISTS idx_highlights_guild
    ON highlights (guild_id);

CREATE INDEX IF NOT EXISTS idx_highlights_guild_user
    ON highlights (guild_id, user_id);
