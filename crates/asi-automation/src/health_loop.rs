use std::sync::Arc;
use tokio::time::MissedTickBehavior;

use super::self_heal::SelfHealEngine;
use super::AutomationConfig;

/// Start the continuous health-check loop.
pub fn start_health_loop(config: &AutomationConfig, engine: Arc<SelfHealEngine>) {
    let interval = config.health_interval;

    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(interval);
        ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);

        loop {
            ticker.tick().await;

            let pool = asi_db::get_db();
            let db_ok = pool.try_acquire().is_some();
            if db_ok {
                engine.record_db_success();
            } else {
                let level = engine.record_db_failure();
                tracing::warn!("Health check: DB degraded (level={:?})", level);
            }

            tracing::info!(
                "Health loop tick — {} — db_ok={}",
                engine.health_summary(),
                db_ok
            );
        }
    });
}
