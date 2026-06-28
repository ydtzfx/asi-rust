pub mod queries;
pub mod schema;
pub mod session_cleanup;
pub mod session_store;

use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::sync::OnceLock;

static DB_POOL: OnceLock<SqlitePool> = OnceLock::new();

/// Initialize the database pool. Must be called once at startup.
/// Enables WAL mode, foreign keys, and a busy timeout for concurrent writes.
/// Pool size is configurable via `DATABASE_POOL_SIZE` env var (default: 10).
pub async fn init_db(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    let pool_size: u32 = std::env::var("DATABASE_POOL_SIZE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(10);

    let opts = SqliteConnectOptions::new()
        .filename(database_url)
        .pragma("journal_mode", "WAL")
        .pragma("foreign_keys", "ON")
        .pragma("busy_timeout", "5000")
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(pool_size)
        .connect_with(opts)
        .await?;

    sqlx::migrate!("../../migrations").run(&pool).await?;

    Ok(pool)
}

/// Get a reference to the initialized database pool.
/// Panics if `init_db` hasn't been called.
pub fn get_db() -> &'static SqlitePool {
    DB_POOL
        .get()
        .expect("Database not initialized. Call init_db first.")
}

/// Set the global database pool. Used by `init_db` or tests.
pub fn set_db_pool(pool: SqlitePool) {
    DB_POOL.set(pool).ok();
}
