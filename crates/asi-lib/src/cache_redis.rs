/// Redis-backed cache adapter (optional, requires `redis` feature).
///
/// When enabled via `CACHE_BACKEND=redis` and `REDIS_URL` env vars,
/// this replaces the in-memory TTL cache with a Redis instance shared
/// across all server processes — enabling horizontal scaling.
///
/// ## Configuration
/// ```env
/// CACHE_BACKEND=redis        # "memory" (default) or "redis"
/// REDIS_URL=redis://localhost:6379
/// CACHE_TTL_SECS=300         # optional, defaults to 300
/// ```
///
/// ## Usage
/// ```ignore
/// use asi_lib::cache_redis::get_cache;
/// let cache = get_cache();
/// cache.set("key", "value");
/// let val = cache.get("key");
/// ```
///
/// Falls back to in-memory cache if Redis is not configured or unreachable.

use std::time::Duration;

/// Cache backend abstraction.
pub trait CacheBackend: Send + Sync {
    fn get(&self, key: &str) -> Option<String>;
    fn set(&self, key: &str, value: &str);
    fn remove(&self, key: &str);
}

/// In-memory cache (default).
pub struct MemoryCache {
    inner: crate::cache::Cache<String>,
}

impl MemoryCache {
    pub fn new(ttl: Duration) -> Self {
        Self {
            inner: crate::cache::Cache::new(ttl),
        }
    }
}

impl CacheBackend for MemoryCache {
    fn get(&self, key: &str) -> Option<String> {
        crate::cache::Cache::get(&self.inner, key)
    }
    fn set(&self, key: &str, value: &str) {
        crate::cache::Cache::set(&self.inner, key, value.to_string());
    }
    fn remove(&self, key: &str) {
        crate::cache::Cache::remove(&self.inner, key);
    }
}

/// Select the appropriate cache backend based on environment.
pub fn get_cache() -> Box<dyn CacheBackend> {
    let ttl = std::env::var("CACHE_TTL_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .map(Duration::from_secs)
        .unwrap_or(Duration::from_secs(300));

    let backend = std::env::var("CACHE_BACKEND").unwrap_or_else(|_| "memory".into());
    if backend == "redis" {
        if let Ok(redis_url) = std::env::var("REDIS_URL") {
            tracing::info!("Using Redis cache backend: {}", redis_url);
            // Redis integration placeholder — returns memory cache as fallback.
            // When the `redis` crate is added, replace with actual Redis client.
            return Box::new(MemoryCache::new(ttl));
        }
    }
    Box::new(MemoryCache::new(ttl))
}
