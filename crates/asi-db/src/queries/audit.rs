use sqlx::SqlitePool;

pub async fn insert_audit_log(
    pool: &SqlitePool,
    user_id: &str,
    action: &str,
    summary: &str,
    detail_json: Option<&str>,
    session_id: Option<&str>,
    ip: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO audit_log (user_id, action, summary, detail_json, session_id, ip)
         VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(user_id)
    .bind(action)
    .bind(summary)
    .bind(detail_json)
    .bind(session_id)
    .bind(ip)
    .execute(pool)
    .await?;
    Ok(())
}
