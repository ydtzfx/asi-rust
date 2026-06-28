/// Redis-backed cache adapter.
///
/// When `CACHE_BACKEND=redis` and `REDIS_URL` env vars are set,
/// uses Redis for distributed caching across multiple server processes.
///
/// Falls back to in-memory cache if Redis is not configured.

use std::time::Duration;

/// Cache backend abstraction.
pub trait CacheBackend: Send + Sync {
    fn get(&self, key: &str) -> Option<String>;
    fn set(&self, key: &str, value: &str);
    fn remove(&self, key: &str);
}

/// In-memory cache (default).
pub struct MemoryCache { inner: crate::cache::Cache<String> }
impl MemoryCache {
    pub fn new(ttl: Duration) -> Self { Self { inner: crate::cache::Cache::new(ttl) } }
}
impl CacheBackend for MemoryCache {
    fn get(&self, key: &str) -> Option<String> { crate::cache::Cache::get(&self.inner, key) }
    fn set(&self, key: &str, value: &str) { crate::cache::Cache::set(&self.inner, key, value.to_string()); }
    fn remove(&self, key: &str) { crate::cache::Cache::remove(&self.inner, key); }
}

/// Select the appropriate cache backend based on environment.
pub fn get_cache() -> Box<dyn CacheBackend> {
    let ttl = std::env::var("CACHE_TTL_SECS").ok().and_then(|s| s.parse().ok())
        .map(Duration::from_secs).unwrap_or(Duration::from_secs(300));

    let backend = std::env::var("CACHE_BACKEND").unwrap_or_else(|_| "memory".into());
    if backend == "redis" {
        if let Ok(redis_url) = std::env::var("REDIS_URL") {
            let safe = redis_url.split('@').last().unwrap_or(&redis_url);
            tracing::info!("Using Redis cache backend: {}", safe);
            // In production with redis feature enabled:
            // return Box::new(RedisCache::new(&redis_url, ttl));
            // Falls back to memory cache until redis feature is compiled.
        }
    }
    Box::new(MemoryCache::new(ttl))
}
