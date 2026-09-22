/// Returns the maximum number of agent loop steps.
///
/// When the `read-only-mode` feature flag is enabled, the agent operates in
/// a restricted mode with fewer steps. Otherwise the full step budget is used.
///
/// - Read-only mode: 5 steps
/// - Normal mode: 20 steps
pub fn get_max_steps() -> usize {
    if asi_lib::flags::flag("read-only-mode") {
        5
    } else {
        20
    }
}

/// Returns whether the agent should use compact instructions.
///
/// Compact mode is enabled when `read-only-mode` is active, since the
/// agent only needs instructions for readFile and listDirectory.
pub fn is_compact_mode() -> bool {
    asi_lib::flags::flag("read-only-mode")
}

/// Serializes tests that mutate the process-global `read-only-mode` override.
///
/// `asi-lib` is a dependency of this crate's tests, so its own `cfg(test)`
/// thread-local override storage is not active here.
#[cfg(test)]
pub(crate) static FLAG_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[cfg(test)]
mod tests {
    use super::*;
    use asi_lib::flags;

    /// Tests are combined into one to avoid races on the global flag state.
    #[test]
    fn test_config_flag_integration() {
        let _guard = FLAG_TEST_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        // Default (no override)
        flags::reset_flag("read-only-mode");
        assert_eq!(get_max_steps(), 20);
        assert!(!is_compact_mode());

        // With read-only-mode enabled
        flags::set_flag("read-only-mode");
        assert_eq!(get_max_steps(), 5);
        assert!(is_compact_mode());

        // Clean up
        flags::reset_flag("read-only-mode");

        // Verify restored
        assert_eq!(get_max_steps(), 20);
        assert!(!is_compact_mode());
    }
}
