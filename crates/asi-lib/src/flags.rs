use std::collections::HashMap;
use std::sync::{LazyLock, OnceLock, RwLock};

/// Default values for production-safe flags.
static DEFAULTS: OnceLock<HashMap<&'static str, bool>> = OnceLock::new();

fn defaults() -> &'static HashMap<&'static str, bool> {
    DEFAULTS.get_or_init(|| {
        let mut m = HashMap::new();
        m.insert("multi-agent", false);
        m.insert("prompt-injection-defense", true);
        m.insert("audit-logging", false);
        m.insert("session-persistence", false);
        m.insert("model-fallback", false);
        m.insert("user-feedback", false);
        m.insert("read-only-mode", false);
        m
    })
}

static OVERRIDES: LazyLock<RwLock<HashMap<String, bool>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

/// Check if a feature flag is enabled.
/// Priority: runtime set_flag > FEATURE_<NAME> env var > default.
pub fn flag(name: &str) -> bool {
    // Runtime override
    if let Ok(overrides) = OVERRIDES.read()
        && let Some(&v) = overrides.get(name)
    {
        return v;
    }

    // Environment variable
    let env_key = format!("FEATURE_{}", name.to_uppercase().replace('-', "_"));
    if let Ok(val) = std::env::var(&env_key) {
        return val == "1" || val.to_lowercase() == "true";
    }

    // Default
    defaults().get(name).copied().unwrap_or(false)
}

pub fn set_flag(name: &str) {
    if let Ok(mut overrides) = OVERRIDES.write() {
        overrides.insert(name.to_string(), true);
    }
}

pub fn reset_flag(name: &str) {
    if let Ok(mut overrides) = OVERRIDES.write() {
        overrides.remove(name);
    }
}

pub fn get_all_flags() -> Vec<(String, bool)> {
    defaults()
        .keys()
        .map(|&k| (k.to_string(), flag(k)))
        .collect()
}
