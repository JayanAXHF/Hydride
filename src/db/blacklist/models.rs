use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
#[allow(dead_code)]
pub struct BlacklistRecord {
    pub id: i64,
    pub guild_id: i64,
    pub user_id: i64,
    pub reason: Option<String>,
    pub created_at: i64,
}
