//! PostgreSQL adapter — alternative backend for production deployments.
//! Activated via DATABASE_URL starting with "postgres://".

use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::PgPool;

/// Initialize a PostgreSQL pool.
/// Called when DATABASE_URL starts with "postgres://" or "postgresql://".
pub async fn init_postgres(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let pool_size: u32 = std::env::var("DATABASE_POOL_SIZE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(20); // PG can handle more connections than SQLite

    let opts: PgConnectOptions = database_url.parse()?;

    let pool = PgPoolOptions::new()
        .max_connections(pool_size)
        .test_before_acquire(true)
        .connect_with(opts)
        .await?;

    // Run migrations on PG.
    sqlx::migrate!("../../migrations").run(&pool).await?;

    tracing::info!("PostgreSQL pool initialized (size={})", pool_size);
    Ok(pool)
}

/// Check if a database URL is PostgreSQL.
pub fn is_postgres_url(url: &str) -> bool {
    url.starts_with("postgres://") || url.starts_with("postgresql://")
}
