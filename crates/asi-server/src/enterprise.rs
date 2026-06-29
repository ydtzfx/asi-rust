//! Enterprise integration layer — wires all 15 crates together at runtime.

use std::sync::Arc;
use std::sync::OnceLock;

use asi_automation::self_heal::SelfHealEngine;
use asi_cortex::analyzer::CortexAnalyzer;
use asi_cortex::decision::CortexDecision;
use asi_cortex::monitor::CortexMonitor;
use asi_cortex::predictor::CortexPredictor;
use asi_defense::defense_layers::DefenseInDepth;
use asi_defense::threat_detector::ThreatDetector;
use asi_evolution::prompt_evo::PromptEvolution;
use asi_evolution::knowledge::KnowledgeBase;
use asi_evolution::ab_test::AbTestEngine;

/// Global enterprise state initialized at startup.
pub struct EnterpriseRuntime {
    pub defense: DefenseInDepth,
    pub detector: ThreatDetector,
    pub cortex_monitor: CortexMonitor,
    pub cortex_analyzer: CortexAnalyzer,
    pub cortex_decision: CortexDecision,
    pub cortex_predictor: tokio::sync::Mutex<CortexPredictor>,
    pub prompt_evo: PromptEvolution,
    pub knowledge: KnowledgeBase,
    pub ab_test: AbTestEngine,
    pub heal_engine: Arc<SelfHealEngine>,
}

static ENTERPRISE: OnceLock<Arc<tokio::sync::Mutex<EnterpriseRuntime>>> = OnceLock::new();

/// Initialize all enterprise subsystems. Called once at startup.
pub fn init_enterprise(heal_engine: Arc<SelfHealEngine>) {
    let runtime = EnterpriseRuntime {
        defense: DefenseInDepth::new(),
        detector: ThreatDetector::new(),
        cortex_monitor: CortexMonitor::new(),
        cortex_analyzer: CortexAnalyzer::new(),
        cortex_decision: CortexDecision::new(),
        cortex_predictor: tokio::sync::Mutex::new(CortexPredictor::new()),
        prompt_evo: PromptEvolution::new(),
        knowledge: KnowledgeBase::new(),
        ab_test: AbTestEngine::new(),
        heal_engine,
    };

    // Register base prompts for evolution.
    runtime.prompt_evo.register("code_agent", "You are a coding assistant.");
    runtime.prompt_evo.register("review_agent", "You are a code reviewer.");

    // Seed knowledge base with comprehensive project context.
    crate::knowledge_seed::seed_knowledge(&runtime.knowledge);

    ENTERPRISE
        .set(Arc::new(tokio::sync::Mutex::new(runtime)))
        .ok();

    tracing::info!("Enterprise runtime initialized — 15 crates integrated");
}

/// Get the enterprise runtime.
pub fn enterprise() -> Option<Arc<tokio::sync::Mutex<EnterpriseRuntime>>> {
    ENTERPRISE.get().cloned()
}

/// Start the enterprise background loop (cortex monitoring + predictions).
pub fn start_enterprise_loop() {
    tokio::spawn(async {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            if let Some(ent) = enterprise() {
                let mut rt = ent.lock().await;

                // Collect snapshot.
                let snapshot = rt.cortex_monitor.snapshot();

                // Analyze.
                let analysis = rt.cortex_analyzer.analyze(&snapshot);

                // Feed predictor.
                rt.cortex_predictor.lock().await.feed(snapshot);

                // Make decisions.
                let predictions = rt.cortex_predictor.lock().await.predict();
                let decisions = rt.cortex_decision.decide(&analysis, &predictions);

                // Log health.
                tracing::info!(
                    "Cortex tick — health={:?} score={}/100 — {} decisions — defense={}",
                    analysis.overall_health,
                    analysis.score,
                    decisions.len(),
                    rt.defense.posture()
                );

                // Detect threats and respond.
                if let Some(first_decision) = decisions.first() {
                    if first_decision.auto_execute {
                        tracing::warn!("Cortex auto-executing: {:?}", first_decision.action);
                    }
                }
            }
        }
    });

    tracing::info!("Enterprise cortex loop started (60s interval)");
}
