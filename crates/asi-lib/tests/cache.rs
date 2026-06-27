use asi_lib::cache::Cache;
use std::time::Duration;

#[test]
fn test_cache_set_get() {
    let cache: Cache<String> = Cache::new(Duration::from_secs(60));
    cache.set("key1", "value1".to_string());
    assert_eq!(cache.get("key1"), Some("value1".to_string()));
    assert_eq!(cache.get("nonexistent"), None);
}

#[test]
fn test_cache_remove_and_clear() {
    let cache: Cache<String> = Cache::new(Duration::from_secs(60));
    cache.set("key1", "value1".to_string());
    cache.set("key2", "value2".to_string());

    cache.remove("key1");
    assert_eq!(cache.get("key1"), None);
    assert_eq!(cache.get("key2"), Some("value2".to_string()));

    cache.clear();
    assert_eq!(cache.get("key2"), None);
}
