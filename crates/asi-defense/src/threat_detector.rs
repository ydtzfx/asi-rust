use serde::Serialize;
use std::collections::HashMap;
use std::sync::Mutex;

/// Threat severity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum ThreatLevel {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

/// A detected threat.
#[derive(Debug, Clone, Serialize)]
pub struct Threat {
    pub id: String,
    pub category: String,
    pub level: ThreatLevel,
    pub source_ip: Option<String>,
    pub description: String,
    pub detected_at: u64,
    pub auto_blocked: bool,
}

/// Anomaly-based threat detector.
pub struct ThreatDetector {
    threats: Mutex<Vec<Threat>>,
    ip_scores: Mutex<HashMap<String, u32>>,
    pattern_counts: Mutex<HashMap<String, u64>>,
}

impl ThreatDetector {
    pub fn new() -> Self {
        Self {
            threats: Mutex::new(Vec::new()),
            ip_scores: Mutex::new(HashMap::new()),
            pattern_counts: Mutex::new(HashMap::new()),
        }
    }

    /// Analyze a request for threats.
    pub fn analyze(
        &self,
        ip: &str,
        path: &str,
        body: &str,
        headers: &HashMap<String, String>,
    ) -> Vec<Threat> {
        let mut threats = Vec::new();

        // Check for SQL injection attempts.
        if body.contains("SELECT") || body.contains("DROP") || body.contains("UNION") {
            let count = self.incr_pattern("sql_injection");
            threats.push(Threat {
                id: format!("threat_{}", now()),
                category: "sql_injection".into(),
                level: if count > 3 { ThreatLevel::High } else { ThreatLevel::Medium },
                source_ip: Some(ip.to_string()),
                description: "SQL injection pattern detected in request body".into(),
                detected_at: now(),
                auto_blocked: count > 5,
            });
        }

        // Check for path traversal.
        if path.contains("../") || path.contains("..\\") {
            threats.push(Threat {
                id: format!("threat_{}", now()),
                category: "path_traversal".into(),
                level: ThreatLevel::High,
                source_ip: Some(ip.to_string()),
                description: "Path traversal attempt detected".into(),
                detected_at: now(),
                auto_blocked: true,
            });
        }

        // Check for excessive request rate per IP.
        let ip_score = self.bump_ip_score(ip);
        if ip_score > 100 {
            threats.push(Threat {
                id: format!("threat_{}", now()),
                category: "rate_anomaly".into(),
                level: if ip_score > 500 { ThreatLevel::Critical } else { ThreatLevel::High },
                source_ip: Some(ip.to_string()),
                description: format!("High request rate from IP: {} requests", ip_score),
                detected_at: now(),
                auto_blocked: ip_score > 500,
            });
        }

        // Check for suspicious headers.
        if headers.contains_key("x-forwarded-for")
            && headers.get("x-forwarded-for").map_or(false, |v| v.contains(','))
        {
            threats.push(Threat {
                id: format!("threat_{}", now()),
                category: "header_spoofing".into(),
                level: ThreatLevel::Low,
                source_ip: Some(ip.to_string()),
                description: "Multiple IPs in X-Forwarded-For — possible spoofing".into(),
                detected_at: now(),
                auto_blocked: false,
            });
        }

        // Store threats.
        if !threats.is_empty() {
            let mut store = self.threats.lock().unwrap();
            store.extend(threats.clone());
        }

        threats
    }

    fn bump_ip_score(&self, ip: &str) -> u32 {
        let mut scores = self.ip_scores.lock().unwrap();
        let score = scores.entry(ip.to_string()).or_insert(0);
        *score += 1;
        *score
    }

    fn incr_pattern(&self, pattern: &str) -> u64 {
        let mut counts = self.pattern_counts.lock().unwrap();
        let count = counts.entry(pattern.to_string()).or_insert(0);
        *count += 1;
        *count
    }

    /// Get recent threats.
    pub fn recent(&self, limit: usize) -> Vec<Threat> {
        let store = self.threats.lock().unwrap();
        let mut threats = store.clone();
        threats.sort_by(|a, b| b.detected_at.cmp(&a.detected_at));
        threats.truncate(limit);
        threats
    }

    /// Reset IP scores (call periodically).
    pub fn decay_scores(&self) {
        let mut scores = self.ip_scores.lock().unwrap();
        for score in scores.values_mut() {
            *score = score.saturating_sub(10);
        }
        scores.retain(|_, v| *v > 0);
    }
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_sql_injection() {
        let detector = ThreatDetector::new();
        let threats = detector.analyze(
            "10.0.0.1",
            "/api/search",
            "SELECT * FROM users",
            &HashMap::new(),
        );
        assert!(!threats.is_empty());
        assert_eq!(threats[0].category, "sql_injection");
    }

    #[test]
    fn test_detect_path_traversal() {
        let detector = ThreatDetector::new();
        let threats = detector.analyze("10.0.0.1", "/../etc/passwd", "", &HashMap::new());
        assert_eq!(threats[0].category, "path_traversal");
    }

    #[test]
    fn test_ip_score_tracking() {
        let detector = ThreatDetector::new();
        for _ in 0..101 {
            detector.analyze("10.0.0.99", "/api", "", &HashMap::new());
        }
        let recent = detector.recent(5);
        assert!(recent.iter().any(|t| t.category == "rate_anomaly"));
    }
}
