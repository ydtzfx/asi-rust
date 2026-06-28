use std::sync::Arc;
use std::sync::OnceLock;
use std::time::Duration;

use asi_automation::AutomationConfig;
use asi_automation::self_heal::SelfHealEngine;
use asi_automation::watchdog::Watchdog;

static ENGINE: OnceLock<Arc<SelfHealEngine>> = OnceLock::new();

pub fn get_heal_engine() -> Arc<SelfHealEngine> {
    ENGINE.get_or_init(|| Arc::new(SelfHealEngine::new())).clone()
}

pub fn run_startup_hooks() {
    let config = AutomationConfig::from_env();
    let engine = get_heal_engine();

    // ---- Initialize enterprise runtime (cross-crate integration) ----
    crate::enterprise::init_enterprise(engine.clone());
    crate::enterprise::start_enterprise_loop();

    // ---- Watchdog (kill on hang) ----
    if config.watchdog_enabled {
        let wd = Watchdog::new(config.watchdog_timeout);
        wd.start();
        asi_lib::logger::info(
            "Watchdog started",
            &[("timeout_secs", &config.watchdog_timeout.as_secs().to_string())],
        );
    }

    // ---- Health-check loop ----
    if config.self_heal_enabled {
        asi_automation::health_loop::start_health_loop(&config, engine);
        asi_lib::logger::info(
            "Self-healing health loop started",
            &[("interval_secs", &config.health_interval.as_secs().to_string())],
        );
    }

    // ---- Auto-backup ----
    if config.auto_backup_enabled {
        let db_path = std::env::var("DATABASE_URL").unwrap_or_else(|_| "asi.db".into());
        asi_automation::auto_backup::start_auto_backup(db_path, 24, 7);
        asi_lib::logger::info("Auto-backup started (every 24h, keep 7 days)", &[]);
    }

    // ---- Auto-evolve timer ----
    tokio::spawn(async {
        let mut interval = tokio::time::interval(Duration::from_secs(30 * 60));
        interval.tick().await;
        loop {
            interval.tick().await;
            asi_lib::logger::info("Auto-evolve check triggered", &[("component", "startup")]);
        }
    });

    // ---- Session cleanup ----
    tokio::spawn(async {
        let mut interval = tokio::time::interval(Duration::from_secs(24 * 60 * 60));
        interval.tick().await;
        loop {
            interval.tick().await;
            let pool = asi_db::get_db();
            match asi_db::session_cleanup::clean_stale_sessions(pool, 7 * 24 * 3600).await {
                Ok(count) => asi_lib::logger::info(
                    "Session cleanup completed",
                    &[("deleted", &count.to_string())],
                ),
                Err(e) => asi_lib::logger::error(
                    "Session cleanup failed",
                    &[("error", &e.to_string())],
                ),
            }
        }
    });
}
