use snafu::ResultExt;
use sqlx::{SqlitePool, query, query_as};

use crate::{
    db::blacklist::{models::BlacklistRecord, pool},
    error::{AppResult, DatabaseMigrationSnafu, DatabaseSnafu},
};

#[derive(Clone)]
pub struct BlacklistDatabase {
    pool: SqlitePool,
}

impl BlacklistDatabase {
    pub async fn connect(database_url: &str) -> AppResult<Self> {
        let pool = pool::connect(database_url).await?;
        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> AppResult<()> {
        sqlx::migrate!("./migrations_blacklist")
            .run(&self.pool)
            .await
            .context(DatabaseMigrationSnafu)
    }

    /// Insert a highlight. Returns Ok(None) if it already existed (ON CONFLICT DO NOTHING),
    /// Ok(Some(record)) if newly inserted.
    pub async fn add_blacklist(
        &self,
        guild_id: i64,
        user_id: i64,
        reason: Option<String>,
    ) -> AppResult<Option<BlacklistRecord>> {
        let result = query(
            "INSERT INTO blacklist (guild_id, user_id, reason, created_at)
             VALUES (?1, ?2, ?3, strftime('%s', 'now'))
             ON CONFLICT(guild_id, user_id) DO NOTHING",
        )
        .bind(guild_id)
        .bind(user_id)
        .bind(reason)
        .execute(&self.pool)
        .await
        .context(DatabaseSnafu)?;

        if result.rows_affected() == 0 {
            Ok(None)
        } else {
            let record = query_as::<_, BlacklistRecord>(
                "SELECT id, guild_id, user_id, reason, created_at
                 FROM blacklist
                 WHERE guild_id = ?1 AND user_id = ?2",
            )
            .bind(guild_id)
            .bind(user_id)
            .fetch_one(&self.pool)
            .await
            .context(DatabaseSnafu)?;

            Ok(Some(record))
        }
    }

    /// Delete a highlight by id, scoped to (guild_id, user_id) so users can't delete others' blacklist.
    /// Returns true if a row was deleted.
    pub async fn remove_blacklist(&self, guild_id: i64, id: i64) -> AppResult<bool> {
        let result = query(
            "DELETE FROM blacklist
             WHERE id = ?1 AND guild_id = ?2",
        )
        .bind(id)
        .bind(guild_id)
        .execute(&self.pool)
        .await
        .context(DatabaseSnafu)?;

        Ok(result.rows_affected() > 0)
    }

    /// All blacklist for a single guild (used to rebuild just that guild's cache entry).
    pub async fn blacklist_for_guild(&self, guild_id: i64) -> AppResult<Vec<BlacklistRecord>> {
        query_as::<_, BlacklistRecord>(
            "SELECT id, guild_id, user_id, reason, created_at
             FROM blacklist
             WHERE guild_id = ?1
             ORDER BY id ASC",
        )
        .bind(guild_id)
        .fetch_all(&self.pool)
        .await
        .context(DatabaseSnafu)
    }
}
