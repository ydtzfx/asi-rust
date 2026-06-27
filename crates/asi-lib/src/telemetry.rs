use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{LazyLock, Mutex};

/// Atomic counters keyed by route name.
static COUNTERS: LazyLock<Mutex<HashMap<String, AtomicU64>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Track an API call to the given route.
///
/// Increments the counter for `route` atomically.
pub fn track_api_call(route: &str) {
    let mut map = COUNTERS.lock().unwrap();
    map.entry(route.to_string())
        .or_insert_with(|| AtomicU64::new(0))
        .fetch_add(1, Ordering::Relaxed);
}

/// Return a snapshot of all route call counts.
pub fn get_api_stats() -> HashMap<String, u64> {
    let map = COUNTERS.lock().unwrap();
    map.iter()
        .map(|(k, v)| (k.clone(), v.load(Ordering::Relaxed)))
        .collect()
}
