use crate::queries::sessions;
use crate::schema::Session;
use sqlx::SqlitePool;
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize)]
pub struct NewSession {
    pub id: String,
    pub user_id: String,
    pub title: Option<String>,
}

pub struct SessionUpdate {
    pub title: Option<String>,
    pub context_json: Option<String>,
    pub message_count: Option<i64>,
    pub token_used: Option<i64>,
}

pub async fn create_new_session(
    pool: &SqlitePool,
    user_id: &str,
    title: Option<&str>,
) -> Result<NewSession, sqlx::Error> {
    let id = Uuid::new_v4().to_string();
    sessions::create_session(pool, &id, user_id, title).await?;
    Ok(NewSession {
        id,
        user_id: user_id.to_string(),
        title: title.map(String::from),
    })
}

pub async fn update_existing_session(
    pool: &SqlitePool,
    id: &str,
    user_id: &str,
    update: SessionUpdate,
) -> Result<Session, sqlx::Error> {
    sessions::update_session(
        pool,
        id,
        user_id,
        update.title.as_deref(),
        update.context_json.as_deref(),
        update.message_count.unwrap_or(0),
        update.token_used.unwrap_or(0),
    )
    .await
}

pub async fn get_session(
    pool: &SqlitePool,
    id: &str,
) -> Result<Option<Session>, sqlx::Error> {
    sessions::get_session(pool, id).await
}
