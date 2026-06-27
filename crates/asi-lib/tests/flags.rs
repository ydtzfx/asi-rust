use asi_lib::flags::*;
use std::env;

#[test]
fn test_flag_defaults() {
    // Without env vars, all flags default to off
    assert!(!flag("multi-agent"));
    assert!(flag("prompt-injection-defense")); // on by default
}

#[test]
fn test_set_and_reset_flag() {
    set_flag("test-flag");
    assert!(flag("test-flag"));
    reset_flag("test-flag");
    assert!(!flag("test-flag"));
}

#[test]
fn test_env_var_flag() {
    // SAFETY: test-only env var mutations, single-threaded test
    unsafe {
        env::set_var("FEATURE_TEST_ENV", "1");
    }
    assert!(flag("test-env"));
    unsafe {
        env::remove_var("FEATURE_TEST_ENV");
    }
    reset_flag("test-env"); // clean up runtime state
}
