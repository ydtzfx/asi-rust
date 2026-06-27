use crate::schema::Session;
use sqlx::SqlitePool;

pub async fn create_session(
    pool: &SqlitePool,
    id: &str,
    user_id: &str,
    title: Option<&str>,
) -> Result<Session, sqlx::Error> {
    let now = unix_now();
    sqlx::query_as::<_, Session>(
        "INSERT INTO sessions (id, user_id, title, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?) RETURNING *",
    )
    .bind(id)
    .bind(user_id)
    .bind(title)
    .bind(now)
    .bind(now)
    .fetch_one(pool)
    .await
}

pub async fn update_session(
    pool: &SqlitePool,
    id: &str,
    user_id: &str,
    title: Option<&str>,
    context_json: Option<&str>,
    message_count: i64,
    token_used: i64,
) -> Result<Session, sqlx::Error> {
    let now = unix_now();
    sqlx::query_as::<_, Session>(
        "UPDATE sessions SET
           title = COALESCE(?, title),
           context_json = COALESCE(?, context_json),
           message_count = MAX(message_count, ?),
           token_used = MAX(token_used, ?),
           updated_at = ?
         WHERE id = ? AND user_id = ?
         RETURNING *",
    )
    .bind(title)
    .bind(context_json)
    .bind(message_count)
    .bind(token_used)
    .bind(now)
    .bind(id)
    .bind(user_id)
    .fetch_one(pool)
    .await
}

pub async fn get_session(pool: &SqlitePool, id: &str) -> Result<Option<Session>, sqlx::Error> {
    sqlx::query_as::<_, Session>("SELECT * FROM sessions WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn list_sessions_by_user(
    pool: &SqlitePool,
    user_id: &str,
    limit: i64,
    offset: i64,
) -> Result<Vec<Session>, sqlx::Error> {
    sqlx::query_as::<_, Session>(
        "SELECT * FROM sessions WHERE user_id = ? ORDER BY updated_at DESC LIMIT ? OFFSET ?",
    )
    .bind(user_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}

pub async fn delete_session(
    pool: &SqlitePool,
    id: &str,
    user_id: &str,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM sessions WHERE id = ? AND user_id = ?")
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
