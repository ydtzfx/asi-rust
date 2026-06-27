use sqlx::SqlitePool;

async fn test_pool() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../../migrations").run(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn test_upsert_and_get_user() {
    let pool = test_pool().await;
    let user = asi_db::queries::users::upsert_user(
        &pool,
        "user_1",
        "test@example.com",
        Some("Alice"),
        None,
        None,
        None,
    )
    .await
    .unwrap();
    assert_eq!(user.email, "test@example.com");
    assert_eq!(user.first_name.unwrap(), "Alice");

    let fetched = asi_db::queries::users::get_user(&pool, "user_1")
        .await
        .unwrap();
    assert!(fetched.is_some());
    assert_eq!(fetched.unwrap().email, "test@example.com");
}

#[tokio::test]
async fn test_session_crud() {
    let pool = test_pool().await;
    // Create user first (FK constraint)
    asi_db::queries::users::upsert_user(
        &pool,
        "user_1",
        "test@example.com",
        None,
        None,
        None,
        None,
    )
    .await
    .unwrap();

    let session =
        asi_db::queries::sessions::create_session(&pool, "sess_1", "user_1", Some("Test Session"))
            .await
            .unwrap();
    assert_eq!(session.title.unwrap(), "Test Session");
    assert_eq!(session.message_count, 0);

    let updated = asi_db::queries::sessions::update_session(
        &pool,
        "sess_1",
        "user_1",
        None,
        Some("{\"key\":\"val\"}"),
        10,
        500,
    )
    .await
    .unwrap();
    assert_eq!(updated.message_count, 10);
    assert_eq!(updated.token_used, 500);
    assert_eq!(updated.context_json.unwrap(), "{\"key\":\"val\"}");
}

#[tokio::test]
async fn test_session_cleanup() {
    let pool = test_pool().await;
    asi_db::queries::users::upsert_user(
        &pool,
        "user_1",
        "test@example.com",
        None,
        None,
        None,
        None,
    )
    .await
    .unwrap();

    asi_db::queries::sessions::create_session(&pool, "sess_old", "user_1", None)
        .await
        .unwrap();

    // Set session timestamp to 8 days ago (well beyond 7-day cleanup)
    sqlx::query("UPDATE sessions SET updated_at = ? WHERE id = ?")
        .bind(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64
                - 691200,
        ) // 8 days
        .bind("sess_old")
        .execute(&pool)
        .await
        .unwrap();

    let deleted = asi_db::session_cleanup::clean_stale_sessions(&pool, 604800)
        .await
        .unwrap();
    assert_eq!(deleted, 1);
}

#[tokio::test]
async fn test_audit_log_insert() {
    let pool = test_pool().await;
    asi_db::queries::users::upsert_user(
        &pool,
        "user_1",
        "test@example.com",
        None,
        None,
        None,
        None,
    )
    .await
    .unwrap();

    asi_db::queries::audit::insert_audit_log(
        &pool,
        "user_1",
        "test_action",
        "Test summary",
        None,
        None,
        Some("127.0.0.1"),
    )
    .await
    .unwrap();

    // Verify by counting
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM audit_log")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count.0, 1);
}
