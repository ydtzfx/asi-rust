use std::collections::HashSet;
use std::sync::Mutex;

/// A firewall rule that can be learned or manually configured.
#[derive(Debug, Clone)]
pub struct FirewallRule {
    pub id: String,
    pub pattern: String, // IP, CIDR, path pattern
    pub rule_type: RuleType,
    pub action: RuleAction,
    pub hits: u64,
    pub learned: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleType {
    IpBlock,
    PathBlock,
    MethodBlock,
    HeaderBlock,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleAction {
    Allow,
    Deny,
    RateLimit,
}

/// Adaptive firewall — learns from threats and adjusts rules.
pub struct AdaptiveFirewall {
    rules: Mutex<Vec<FirewallRule>>,
    deny_list: Mutex<HashSet<String>>,
}

impl AdaptiveFirewall {
    pub fn new() -> Self {
        let mut deny_list = HashSet::new();
        // Default deny list — common attack IPs.
        deny_list.insert("127.0.0.1: spoofing test".to_string());

        Self {
            rules: Mutex::new(vec![
                FirewallRule {
                    id: "rule_default_allow".into(),
                    pattern: "*".into(),
                    rule_type: RuleType::PathBlock,
                    action: RuleAction::Allow,
                    hits: 0,
                    learned: false,
                },
            ]),
            deny_list: Mutex::new(deny_list),
        }
    }

    /// Check if an IP is allowed.
    pub fn is_allowed(&self, ip: &str) -> bool {
        let deny = self.deny_list.lock().unwrap();
        !deny.iter().any(|entry| entry.starts_with(ip))
    }

    /// Learn a new rule from a detected threat.
    pub fn learn(&self, ip: &str, threat_category: &str) {
        let mut deny = self.deny_list.lock().unwrap();
        let entry = format!("{}: {}", ip, threat_category);
        if !deny.contains(&entry) {
            deny.insert(entry);
            tracing::info!(
                "Firewall: learned rule — blocking {} due to {}",
                ip,
                threat_category
            );
        }
    }

    /// Add a custom rule.
    pub fn add_rule(&self, rule: FirewallRule) {
        let mut rules = self.rules.lock().unwrap();
        rules.push(rule);
    }

    /// Check a request against all rules.
    pub fn check(&self, ip: &str, path: &str, method: &str) -> RuleAction {
        // Check deny list first.
        if !self.is_allowed(ip) {
            return RuleAction::Deny;
        }

        // Check rules.
        let rules = self.rules.lock().unwrap();
        for rule in rules.iter() {
            let matches = match rule.rule_type {
                RuleType::IpBlock => rule.pattern == ip,
                RuleType::PathBlock => {
                    rule.pattern == "*" || path.contains(&rule.pattern)
                }
                RuleType::MethodBlock => rule.pattern == method,
                RuleType::HeaderBlock => false, // Header rules require header inspection.
            };
            if matches {
                return rule.action;
            }
        }

        RuleAction::Allow
    }

    /// Get the deny list size.
    pub fn deny_count(&self) -> usize {
        self.deny_list.lock().unwrap().len()
    }

    /// Get all rules.
    pub fn rules(&self) -> Vec<FirewallRule> {
        self.rules.lock().unwrap().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_allow() {
        let fw = AdaptiveFirewall::new();
        assert_eq!(
            fw.check("10.0.0.1", "/api/health", "GET"),
            RuleAction::Allow
        );
    }

    #[test]
    fn test_learn_and_block() {
        let fw = AdaptiveFirewall::new();
        fw.learn("192.168.1.100", "sql_injection");
        assert_eq!(
            fw.check("192.168.1.100", "/api/chat", "POST"),
            RuleAction::Deny
        );
    }

    #[test]
    fn test_learn_does_not_block_other_ips() {
        let fw = AdaptiveFirewall::new();
        fw.learn("10.0.0.99", "rate_anomaly");
        // Different IP should still be allowed.
        assert!(fw.is_allowed("10.0.0.1"));
        assert!(!fw.is_allowed("10.0.0.99"));
    }
}
