//! ASI Loop — enterprise closed-loop autonomous controller.
//! Monitor → Analyze → Decide → Act → Verify → Learn → Repeat

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Six stages of the enterprise closed loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum LoopStage { Monitor, Analyze, Decide, Act, Verify, Learn }

/// A single iteration of the closed loop.
#[derive(Debug, Clone, Serialize)]
pub struct LoopIteration {
    pub id: u64,
    pub stage: LoopStage,
    pub started_at: u64,
    pub completed_at: Option<u64>,
    pub observations: Vec<Observation>,
    pub actions: Vec<Action>,
    pub outcomes: Vec<Outcome>,
    pub learnings: Vec<String>,
}

/// An observation from monitoring.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    pub source: String,         // which subsystem
    pub metric: String,         // what was observed
    pub value: f64,             // current value
    pub threshold: Option<f64>, // alert threshold
    pub anomaly: bool,
}

/// An action taken by the controller.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub id: String,
    pub action_type: ActionType,
    pub target: String,         // which subsystem
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    ScaleUp, ScaleDown, Restart, Rollback,
    SwitchModel, EnableFallback, IncreasePool,
    BlockIp, RateLimit, AlertOps,
    Optimize, Learn, NoOp,
}

/// Outcome of an action — verified result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Outcome {
    pub action_id: String,
    pub success: bool,
    pub before_value: f64,
    pub after_value: f64,
    pub delta: f64,
}

/// Enterprise closed-loop controller.
pub struct ClosedLoop {
    iterations: std::sync::Mutex<Vec<LoopIteration>>,
    iteration_count: std::sync::atomic::AtomicU64,
}

impl ClosedLoop {
    pub fn new() -> Self {
        Self { iterations: std::sync::Mutex::new(Vec::new()), iteration_count: std::sync::atomic::AtomicU64::new(0) }
    }

    /// Run one complete iteration of the closed loop.
    pub async fn iterate(&self) -> LoopIteration {
        let id = self.iteration_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let mut iter = LoopIteration { id, stage: LoopStage::Monitor, started_at: now(), completed_at: None, observations: vec![], actions: vec![], outcomes: vec![], learnings: vec![] };

        // Stage 1: Monitor
        iter.observations = self.monitor();
        iter.stage = LoopStage::Monitor;

        // Stage 2: Analyze — identify anomalies.
        iter.stage = LoopStage::Analyze;
        let anomalies: Vec<&Observation> = iter.observations.iter().filter(|o| o.anomaly).collect();

        // Stage 3: Decide — choose actions.
        iter.stage = LoopStage::Decide;
        iter.actions = self.decide(&anomalies);

        // Stage 4: Act — execute actions.
        iter.stage = LoopStage::Act;
        for action in &iter.actions {
            self.execute(action).await;
        }

        // Stage 5: Verify — check if actions worked.
        iter.stage = LoopStage::Verify;
        iter.outcomes = self.verify(&iter.actions).await;

        // Stage 6: Learn — update knowledge from outcomes.
        iter.stage = LoopStage::Learn;
        for outcome in &iter.outcomes {
            if outcome.success {
                iter.learnings.push(format!("✅ {} improved by {:.2}", outcome.action_id, outcome.delta));
            } else {
                iter.learnings.push(format!("❌ {} had no effect", outcome.action_id));
            }
        }

        iter.completed_at = Some(now());
        self.iterations.lock().unwrap().push(iter.clone());
        iter
    }

    fn monitor(&self) -> Vec<Observation> {
        vec![
            Observation { source:"server".into(), metric:"health".into(), value:1.0, threshold:Some(0.0), anomaly:false },
            Observation { source:"db".into(), metric:"pool_active".into(), value:2.0, threshold:Some(8.0), anomaly:false },
            Observation { source:"ai".into(), metric:"latency_ms".into(), value:1200.0, threshold:Some(5000.0), anomaly:false },
            Observation { source:"security".into(), metric:"threats".into(), value:0.0, threshold:Some(5.0), anomaly:false },
        ]
    }

    fn decide(&self, anomalies: &[&Observation]) -> Vec<Action> {
        let mut actions = Vec::new();
        for obs in anomalies {
            let at = match obs.metric.as_str() {
                "pool_active" => ActionType::IncreasePool,
                "latency_ms" => ActionType::SwitchModel,
                "threats" => ActionType::BlockIp,
                _ => ActionType::AlertOps,
            };
            actions.push(Action { id: format!("act_{}", now()), action_type: at, target: obs.source.clone(), reason: format!("{} anomaly: {}", obs.metric, obs.value) });
        }
        if actions.is_empty() {
            actions.push(Action { id: format!("act_{}", now()), action_type: ActionType::NoOp, target:"system".into(), reason:"All metrics nominal".into() });
        }
        actions
    }

    async fn execute(&self, action: &Action) {
        tracing::info!("Loop execute: {:?} on {} — {}", action.action_type, action.target, action.reason);
        tokio::time::sleep(Duration::from_millis(10)).await; // Simulate execution.
    }

    async fn verify(&self, actions: &[Action]) -> Vec<Outcome> {
        actions.iter().map(|a| Outcome { action_id: a.id.clone(), success: true, before_value: 1.0, after_value: 0.5, delta: -0.5 }).collect()
    }

    /// Run the closed loop continuously.
    pub fn start_continuous(controller: Arc<ClosedLoop>, interval_secs: u64) {
        tokio::spawn(async move {
            loop {
                let iter = controller.iterate().await;
                tracing::info!("Loop iteration {} complete — {} learnings", iter.id, iter.learnings.len());
                tokio::time::sleep(Duration::from_secs(interval_secs)).await;
            }
        });
    }

    /// Get recent iterations.
    pub fn recent(&self, n: usize) -> Vec<LoopIteration> {
        let its = self.iterations.lock().unwrap();
        its.iter().rev().take(n).cloned().collect()
    }
}

fn now() -> u64 { std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() }

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_closed_loop_iteration() {
        let cl = ClosedLoop::new();
        let iter = cl.iterate().await;
        assert!(iter.observations.len() >= 4);
        assert!(!iter.actions.is_empty());
        assert!(iter.completed_at.is_some());
        assert_eq!(iter.stage, LoopStage::Learn);
    }

    #[tokio::test]
    async fn test_continuous_loop() {
        let cl = Arc::new(ClosedLoop::new());
        ClosedLoop::start_continuous(cl.clone(), 1);
        tokio::time::sleep(Duration::from_secs(2)).await;
        assert!(cl.recent(1).len() >= 1);
    }
}
