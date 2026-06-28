//! ASI Business — autonomous business operations.
//! Usage billing, SLA management, capacity planning, cost optimization.
pub mod billing;
pub mod capacity;
pub mod sla;

use serde::{Deserialize, Serialize};

/// Customer tier with resource limits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerTier { pub name: String, pub max_tokens: u64, pub max_sessions: u32, pub max_agents: u32 }

/// Usage record for billing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageRecord { pub customer_id: String, pub tokens_used: u64, pub sessions: u32, pub timestamp: u64 }

impl CustomerTier {
    pub fn free() -> Self { Self { name: "Free".into(), max_tokens: 100_000, max_sessions: 10, max_agents: 1 } }
    pub fn pro() -> Self { Self { name: "Pro".into(), max_tokens: 1_000_000, max_sessions: 100, max_agents: 5 } }
    pub fn enterprise() -> Self { Self { name: "Enterprise".into(), max_tokens: u64::MAX, max_sessions: u32::MAX, max_agents: 50 } }
}

#[test]
fn test_tiers() { assert_eq!(CustomerTier::free().max_tokens, 100_000); assert!(CustomerTier::enterprise().max_tokens > 1_000_000); }
