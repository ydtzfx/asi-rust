use std::time::Duration;

/// Run all startup hooks as background tasks.
///
/// Currently spawns:
/// - An auto-evolve timer that triggers every 30 minutes.
/// - A session-cleanup timer that triggers every 24 hours.
///
/// Both are stubs that log their invocation; real logic will be wired
/// when the respective agent and session modules are connected.
pub fn run_startup_hooks() {
    // Auto-evolve cycle: every 30 minutes
    tokio::spawn(async {
        let mut interval = tokio::time::interval(Duration::from_secs(30 * 60));
        // The first tick fires immediately; skip it so we don't run at startup.
        interval.tick().await;
        loop {
            interval.tick().await;
            asi_lib::logger::info(
                "Auto-evolve check triggered",
                &[("component", "startup")],
            );
        }
    });

    // Session cleanup: every 24 hours
    tokio::spawn(async {
        let mut interval = tokio::time::interval(Duration::from_secs(24 * 60 * 60));
        // The first tick fires immediately; skip it so we don't run at startup.
        interval.tick().await;
        loop {
            interval.tick().await;
            asi_lib::logger::info(
                "Session cleanup triggered",
                &[("component", "startup")],
            );
        }
    });
}
