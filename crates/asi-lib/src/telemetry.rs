use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{LazyLock, Mutex};

/// Atomic counters keyed by route pattern.
static COUNTERS: LazyLock<Mutex<HashMap<String, AtomicU64>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Track an API call to the given route pattern.
///
/// Callers should pass a static route pattern (e.g. `/api/chat`), NOT a raw
/// request URI that may contain dynamic segments like `/api/sessions/{id}`.
/// Passing raw URIs with dynamic IDs will cause unbounded key growth.
pub fn track_api_call(route: &str) {
    let mut map = COUNTERS.lock().unwrap();
    map.entry(route.to_string())
        .or_insert_with(|| AtomicU64::new(0))
        .fetch_add(1, Ordering::Relaxed);
}

/// Normalize a URI path into a route pattern by stripping UUIDs and numeric
/// segments.  Use this when the caller only has access to the raw request URI.
///
/// Example: `/api/sessions/abc123-def456/export` → `/api/sessions/:id/export`
pub fn track_api_call_normalized(uri: &str) {
    let normalized = normalize_uri(uri);
    track_api_call(&normalized);
}

/// Strip UUID-like and purely numeric segments from a URI path.
fn normalize_uri(uri: &str) -> String {
    uri.split('/')
        .map(|segment| {
            if looks_like_dynamic(segment) {
                ":id"
            } else {
                segment
            }
        })
        .collect::<Vec<_>>()
        .join("/")
}

fn looks_like_dynamic(segment: &str) -> bool {
    if segment.is_empty() {
        return false;
    }
    // UUID: 36 chars with hyphens.
    if segment.len() == 36 && segment.chars().filter(|c| *c == '-').count() >= 4 {
        return true;
    }
    // Purely numeric (IDs).
    if segment.chars().all(|c| c.is_ascii_digit()) {
        return true;
    }
    false
}

/// Return a snapshot of all route call counts.
pub fn get_api_stats() -> HashMap<String, u64> {
    let map = COUNTERS.lock().unwrap();
    map.iter()
        .map(|(k, v)| (k.clone(), v.load(Ordering::Relaxed)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_static_route() {
        assert_eq!(normalize_uri("/api/chat"), "/api/chat");
    }

    #[test]
    fn test_normalize_uuid_segment() {
        let uri = "/api/sessions/550e8400-e29b-41d4-a716-446655440000/export";
        assert_eq!(normalize_uri(uri), "/api/sessions/:id/export");
    }

    #[test]
    fn test_normalize_numeric_id() {
        assert_eq!(normalize_uri("/api/users/12345"), "/api/users/:id");
    }
}
