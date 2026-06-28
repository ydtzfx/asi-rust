/// Pricing tier for the ASI platform.
/// Reserved for future enforcement — currently unused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Tier {
    Free,
    Pro,
    Enterprise,
}

/// Resource limits associated with a tier.
/// Reserved for future enforcement — currently unused.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TierLimits {
    pub max_sessions: u32,
    pub max_tokens_per_session: u32,
    pub max_agents: u32,
}

impl Tier {
    /// Return the resource limits for this tier.
    pub fn limits(&self) -> TierLimits {
        match self {
            Tier::Free => TierLimits {
                max_sessions: 1,
                max_tokens_per_session: 10_000,
                max_agents: 0,
            },
            Tier::Pro => TierLimits {
                max_sessions: 10,
                max_tokens_per_session: 100_000,
                max_agents: 2,
            },
            Tier::Enterprise => TierLimits {
                max_sessions: u32::MAX,
                max_tokens_per_session: u32::MAX,
                max_agents: u32::MAX,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_free_limits() {
        let limits = Tier::Free.limits();
        assert_eq!(limits.max_sessions, 1);
        assert_eq!(limits.max_tokens_per_session, 10_000);
        assert_eq!(limits.max_agents, 0);
    }

    #[test]
    fn test_pro_limits() {
        let limits = Tier::Pro.limits();
        assert_eq!(limits.max_sessions, 10);
        assert_eq!(limits.max_tokens_per_session, 100_000);
        assert_eq!(limits.max_agents, 2);
    }

    #[test]
    fn test_enterprise_unlimited() {
        let limits = Tier::Enterprise.limits();
        assert_eq!(limits.max_sessions, u32::MAX);
        assert_eq!(limits.max_agents, u32::MAX);
    }
}
