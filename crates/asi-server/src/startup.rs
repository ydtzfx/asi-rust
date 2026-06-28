use std::time::Duration;

/// Run all startup hooks as background tasks.
///
/// Currently spawns:
/// - An auto-evolve timer (logging-only placeholder — evolve is triggered
///   manually via `POST /api/evolve` for now; automatic self-evolution requires
///   a completed implementation of the evolve feedback loop).
/// - A session-cleanup timer that deletes sessions older than 7 days.
pub fn run_startup_hooks() {
    // Auto-evolve cycle: every 30 minutes.
    // Intentionally a logging stub for now — full auto-evolve requires:
    // 1. Agent evaluation metrics collection
    // 2. Prompt optimization from feedback
    // 3. Safe model update mechanism
    // Manual evolve is available via POST /api/evolve.
    tokio::spawn(async {
        let mut interval = tokio::time::interval(Duration::from_secs(30 * 60));
        interval.tick().await;
        loop {
            interval.tick().await;
            asi_lib::logger::info("Auto-evolve check triggered", &[("component", "startup")]);
        }
    });

    // Session cleanup: every 24 hours, deletes sessions older than 7 days.
    tokio::spawn(async {
        let mut interval = tokio::time::interval(Duration::from_secs(24 * 60 * 60));
        // The first tick fires immediately; skip it so we don't run at startup.
        interval.tick().await;
        loop {
            interval.tick().await;
            let pool = asi_db::get_db();
            match asi_db::session_cleanup::clean_stale_sessions(pool, 7 * 24 * 3600).await {
                Ok(count) => {
                    asi_lib::logger::info(
                        "Session cleanup completed",
                        &[("component", "startup"), ("deleted", &count.to_string())],
                    );
                }
                Err(e) => {
                    asi_lib::logger::error(
                        "Session cleanup failed",
                        &[("component", "startup"), ("error", &e.to_string())],
                    );
                }
            }
        }
    });
}
