use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// A simple TTL-based in-memory cache.
pub struct Cache<V> {
    inner: Mutex<HashMap<String, (V, Instant)>>,
    ttl: Duration,
}

impl<V> Cache<V> {
    pub fn new(ttl: Duration) -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
            ttl,
        }
    }

    pub fn get(&self, key: &str) -> Option<V>
    where
        V: Clone,
    {
        let map = self.inner.lock().ok()?;
        if let Some((value, expiry)) = map.get(key) {
            if Instant::now() < *expiry {
                return Some(value.clone());
            }
        }
        None
    }

    pub fn set(&self, key: &str, value: V) {
        if let Ok(mut map) = self.inner.lock() {
            map.insert(key.to_string(), (value, Instant::now() + self.ttl));
        }
    }

    pub fn remove(&self, key: &str) {
        if let Ok(mut map) = self.inner.lock() {
            map.remove(key);
        }
    }

    pub fn clear(&self) {
        if let Ok(mut map) = self.inner.lock() {
            map.clear();
        }
    }
}
