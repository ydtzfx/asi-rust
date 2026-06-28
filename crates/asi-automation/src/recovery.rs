use super::self_heal::FaultLevel;

/// Recovery action to take for a given fault level.
pub enum RecoveryAction {
    /// No action needed.
    None,
    /// Retry the operation with backoff.
    Retry,
    /// Switch to fallback (e.g., fallback AI model).
    Fallback,
    /// Attempt to reconnect (e.g., DB).
    Reconnect,
    /// Restart the server process.
    Restart,
    /// Rollback to previous deployment.
    Rollback,
}

/// Determine the recovery action for a fault level.
pub fn action_for(level: FaultLevel) -> RecoveryAction {
    match level {
        FaultLevel::Transient => RecoveryAction::Retry,
        FaultLevel::Degraded => RecoveryAction::Fallback,
        FaultLevel::Failing => RecoveryAction::Reconnect,
        FaultLevel::Critical => RecoveryAction::Restart,
        FaultLevel::Fatal => RecoveryAction::Rollback,
    }
}

/// Execute a recovery action.
pub async fn execute(action: RecoveryAction) {
    match action {
        RecoveryAction::None => {}
        RecoveryAction::Retry => {
            tracing::info!("Recovery: retry with backoff");
        }
        RecoveryAction::Fallback => {
            tracing::warn!("Recovery: switching to fallback provider");
            asi_lib::flags::set_flag("model-fallback");
        }
        RecoveryAction::Reconnect => {
            tracing::warn!("Recovery: attempting DB reconnection");
        }
        RecoveryAction::Restart => {
            tracing::error!("Recovery: restarting server process");
            // Signal the watchdog to restart.
            // In Docker/K8s, exit triggers container restart.
            std::process::exit(1);
        }
        RecoveryAction::Rollback => {
            tracing::error!("Recovery: triggering rollback to previous version");
            // Write a rollback marker file for the deploy script.
            let _ = std::fs::write("/tmp/asi_rollback", "1");
            std::process::exit(1);
        }
    }
}
