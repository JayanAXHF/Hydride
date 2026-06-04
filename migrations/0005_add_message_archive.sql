CREATE TABLE IF NOT EXISTS message_archive (
    message_id INTEGER PRIMARY KEY NOT NULL,
    guild_id INTEGER NOT NULL,
    channel_id INTEGER NOT NULL,
    author_id INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    content_len INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_message_archive_channel_created_at
    ON message_archive (channel_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_message_archive_channel_author_created_at
    ON message_archive (channel_id, author_id, created_at DESC);
