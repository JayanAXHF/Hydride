CREATE TABLE IF NOT EXISTS blacklist (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    guild_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    UNIQUE (guild_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_blacklist_guild
    ON blacklist (guild_id);
