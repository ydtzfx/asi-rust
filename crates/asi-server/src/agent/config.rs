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

#[cfg(test)]
pub(super) static FLAG_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[cfg(test)]
mod tests {
    use super::*;
    use asi_lib::flags;

    #[test]
    fn test_config_flag_integration() {
        let _guard = FLAG_TEST_LOCK.lock().unwrap();
        flags::reset_flag("read-only-mode");
        let baseline_steps = get_max_steps();
        let baseline_compact = is_compact_mode();

        flags::set_flag("read-only-mode");
        assert_eq!(get_max_steps(), 5);
        assert!(is_compact_mode());

        flags::reset_flag("read-only-mode");
        assert_eq!(get_max_steps(), baseline_steps);
        assert_eq!(is_compact_mode(), baseline_compact);
    }
}
