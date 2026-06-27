use sqlx::SqlitePool;
use tracing::info;

/// Remove sessions older than `max_age_seconds` (default 7 days).
pub async fn clean_stale_sessions(
    pool: &SqlitePool,
    max_age_seconds: i64,
) -> Result<u64, sqlx::Error> {
    let cutoff = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
        - max_age_seconds;

    let result = sqlx::query("DELETE FROM sessions WHERE updated_at < ?")
        .bind(cutoff)
        .execute(pool)
        .await?;

    let deleted = result.rows_affected();
    if deleted > 0 {
        info!(
            "Cleaned {} stale sessions (older than {}s)",
            deleted, max_age_seconds
        );
    }
    Ok(deleted)
}
