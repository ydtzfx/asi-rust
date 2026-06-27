use tokio::signal;

/// Returns a future that resolves when a graceful-shutdown signal is received.
///
/// On Unix: listens for SIGTERM (e.g. `kill` / systemd stop) and SIGINT (Ctrl+C).
/// On Windows: only Ctrl+C is available via `signal::ctrl_c()`.
pub async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            asi_lib::logger::info("Shutdown signal received (Ctrl+C)", &[]);
        }
        _ = terminate => {
            asi_lib::logger::info("Shutdown signal received (SIGTERM)", &[]);
        }
    }
}
