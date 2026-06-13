use snafu::ResultExt;
use sqlx::{SqlitePool, query, query_as};

use crate::{
    db::highlights::{models::HighlightRecord, pool},
    error::{AppResult, DatabaseMigrationSnafu, DatabaseSnafu},
};

#[derive(Clone)]
pub struct HighlightsDatabase {
    pool: SqlitePool,
}

impl HighlightsDatabase {
    pub async fn connect(database_url: &str) -> AppResult<Self> {
        let pool = pool::connect(database_url).await?;
        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> AppResult<()> {
        sqlx::migrate!("./migrations_highlights")
            .run(&self.pool)
            .await
            .context(DatabaseMigrationSnafu)
    }

    /// Insert a highlight. Returns Ok(None) if it already existed (ON CONFLICT DO NOTHING),
    /// Ok(Some(record)) if newly inserted.
    pub async fn add_highlight(
        &self,
        guild_id: i64,
        user_id: i64,
        pattern: &str,
    ) -> AppResult<Option<HighlightRecord>> {
        let result = query(
            "INSERT INTO highlights (guild_id, user_id, pattern, created_at)
             VALUES (?1, ?2, ?3, strftime('%s', 'now'))
             ON CONFLICT(guild_id, user_id, pattern) DO NOTHING",
        )
        .bind(guild_id)
        .bind(user_id)
        .bind(pattern)
        .execute(&self.pool)
        .await
        .context(DatabaseSnafu)?;

        if result.rows_affected() == 0 {
            Ok(None)
        } else {
            let record = query_as::<_, HighlightRecord>(
                "SELECT id, guild_id, user_id, pattern, created_at
                 FROM highlights
                 WHERE guild_id = ?1 AND user_id = ?2 AND pattern = ?3",
            )
            .bind(guild_id)
            .bind(user_id)
            .bind(pattern)
            .fetch_one(&self.pool)
            .await
            .context(DatabaseSnafu)?;

            Ok(Some(record))
        }
    }

    /// Delete a highlight by id, scoped to (guild_id, user_id) so users can't delete others' highlights.
    /// Returns true if a row was deleted.
    pub async fn remove_highlight(&self, guild_id: i64, user_id: i64, id: i64) -> AppResult<bool> {
        let result = query(
            "DELETE FROM highlights
             WHERE id = ?1 AND guild_id = ?2 AND user_id = ?3",
        )
        .bind(id)
        .bind(guild_id)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .context(DatabaseSnafu)?;

        Ok(result.rows_affected() > 0)
    }

    /// List all highlights for a user in a guild (for the `list` command).
    pub async fn list_highlights(
        &self,
        guild_id: i64,
        user_id: i64,
    ) -> AppResult<Vec<HighlightRecord>> {
        query_as::<_, HighlightRecord>(
            "SELECT id, guild_id, user_id, pattern, created_at
             FROM highlights
             WHERE guild_id = ?1 AND user_id = ?2
             ORDER BY id ASC",
        )
        .bind(guild_id)
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .context(DatabaseSnafu)
    }

    /// Count highlights for a user in a guild (for enforcing a max-per-user limit).
    pub async fn count_highlights(&self, guild_id: i64, user_id: i64) -> AppResult<i64> {
        let record = query_as::<_, (i64,)>(
            "SELECT COUNT(*) FROM highlights WHERE guild_id = ?1 AND user_id = ?2",
        )
        .bind(guild_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .context(DatabaseSnafu)?;

        Ok(record.0)
    }

    /// Load ALL highlights, grouped however is convenient — used to build the in-memory cache at
    /// startup and on every add/remove.
    pub async fn all_highlights(&self) -> AppResult<Vec<HighlightRecord>> {
        query_as::<_, HighlightRecord>(
            "SELECT id, guild_id, user_id, pattern, created_at
             FROM highlights
             ORDER BY id ASC",
        )
        .fetch_all(&self.pool)
        .await
        .context(DatabaseSnafu)
    }

    /// All highlights for a single guild (used to rebuild just that guild's cache entry).
    pub async fn highlights_for_guild(&self, guild_id: i64) -> AppResult<Vec<HighlightRecord>> {
        query_as::<_, HighlightRecord>(
            "SELECT id, guild_id, user_id, pattern, created_at
             FROM highlights
             WHERE guild_id = ?1
             ORDER BY id ASC",
        )
        .bind(guild_id)
        .fetch_all(&self.pool)
        .await
        .context(DatabaseSnafu)
    }
}
