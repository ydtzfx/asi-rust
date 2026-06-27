use crate::schema::User;
use sqlx::SqlitePool;

pub async fn upsert_user(
    pool: &SqlitePool,
    id: &str,
    email: &str,
    first_name: Option<&str>,
    last_name: Option<&str>,
    image_url: Option<&str>,
    org_id: Option<&str>,
) -> Result<User, sqlx::Error> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    sqlx::query_as::<_, User>(
        "INSERT INTO users (id, email, first_name, last_name, image_url, org_id, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(id) DO UPDATE SET
           email = excluded.email,
           first_name = excluded.first_name,
           last_name = excluded.last_name,
           image_url = excluded.image_url,
           org_id = excluded.org_id,
           updated_at = excluded.updated_at
         RETURNING *"
    )
    .bind(id)
    .bind(email)
    .bind(first_name)
    .bind(last_name)
    .bind(image_url)
    .bind(org_id)
    .bind(now)
    .bind(now)
    .fetch_one(pool)
    .await
}

pub async fn get_user(pool: &SqlitePool, id: &str) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}
