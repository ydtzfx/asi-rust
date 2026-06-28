use serde::Serialize;

/// Defense layer in the security stack.
#[derive(Debug, Clone, Serialize)]
pub struct DefenseLayer {
    pub name: String,
    pub level: u8, // 1 (outermost) to 6 (innermost)
    pub category: LayerCategory,
    pub controls: Vec<String>,
    pub active: bool,
    pub breach_count: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum LayerCategory {
    Network,
    Application,
    Authentication,
    Data,
    AI,
    Infrastructure,
}

/// Six-layer defense-in-depth architecture.
pub struct DefenseInDepth {
    pub layers: Vec<DefenseLayer>,
}

impl DefenseInDepth {
    pub fn new() -> Self {
        Self {
            layers: vec![
                DefenseLayer {
                    name: "L1: Network Perimeter".into(),
                    level: 1,
                    category: LayerCategory::Network,
                    controls: vec![
                        "Rate limiting (60 req/min)".into(),
                        "CORS restrictions".into(),
                        "TLS termination (Caddy)".into(),
                        "DDoS protection (K8s Ingress)".into(),
                    ],
                    active: true,
                    breach_count: 0,
                },
                DefenseLayer {
                    name: "L2: Application Firewall".into(),
                    level: 2,
                    category: LayerCategory::Application,
                    controls: vec![
                        "Input validation (message limits)".into(),
                        "Prompt injection defense (16 patterns)".into(),
                        "SQL injection prevention (parameterized)".into(),
                        "XSS protection (output escaping)".into(),
                    ],
                    active: true,
                    breach_count: 0,
                },
                DefenseLayer {
                    name: "L3: Authentication".into(),
                    level: 3,
                    category: LayerCategory::Authentication,
                    controls: vec![
                        "Clerk JWT verification".into(),
                        "typ=JWT + alg=RS256 validation".into(),
                        "Dev-bypass two-factor gate".into(),
                        "JWKS fetch with timeout".into(),
                    ],
                    active: true,
                    breach_count: 0,
                },
                DefenseLayer {
                    name: "L4: Data Protection".into(),
                    level: 4,
                    category: LayerCategory::Data,
                    controls: vec![
                        "Path containment (safe_path)".into(),
                        "TOCTOU verification".into(),
                        "Command allowlist".into(),
                        "File read limit (1MB)".into(),
                        "Audit logging".into(),
                    ],
                    active: true,
                    breach_count: 0,
                },
                DefenseLayer {
                    name: "L5: AI Safety".into(),
                    level: 5,
                    category: LayerCategory::AI,
                    controls: vec![
                        "Provider fallback".into(),
                        "CancelToken (client disconnect)".into(),
                        "Circuit breaker".into(),
                        "Retry with backoff".into(),
                        "LLM cache".into(),
                    ],
                    active: true,
                    breach_count: 0,
                },
                DefenseLayer {
                    name: "L6: Infrastructure".into(),
                    level: 6,
                    category: LayerCategory::Infrastructure,
                    controls: vec![
                        "Watchdog (60s timeout)".into(),
                        "Self-heal engine (5-level)".into(),
                        "Auto-rollback on deploy failure".into(),
                        "Health loop (30s interval)".into(),
                        "Session cleanup (7d)".into(),
                    ],
                    active: true,
                    breach_count: 0,
                },
            ],
        }
    }

    /// Record a breach at a specific layer.
    pub fn record_breach(&mut self, layer_level: u8) {
        if let Some(layer) = self.layers.iter_mut().find(|l| l.level == layer_level) {
            layer.breach_count += 1;
        }
    }

    /// Check if all layers are active.
    pub fn integrity_check(&self) -> bool {
        self.layers.iter().all(|l| l.active)
    }

    /// Get the defense posture summary.
    pub fn posture(&self) -> String {
        let total_breaches: u64 = self.layers.iter().map(|l| l.breach_count).sum();
        if total_breaches == 0 {
            "Clean — no breaches detected".into()
        } else if total_breaches < 5 {
            format!("Minor — {} total breaches across layers", total_breaches)
        } else {
            format!("Elevated — {} total breaches, review required", total_breaches)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_six_layers_all_active() {
        let defense = DefenseInDepth::new();
        assert_eq!(defense.layers.len(), 6);
        assert!(defense.integrity_check());
    }

    #[test]
    fn test_record_breach() {
        let mut defense = DefenseInDepth::new();
        defense.record_breach(2);
        assert_eq!(defense.layers[1].breach_count, 1);
    }

    #[test]
    fn test_posture_clean() {
        let defense = DefenseInDepth::new();
        assert!(defense.posture().contains("Clean"));
    }
}
