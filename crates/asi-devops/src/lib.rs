//! ASI DevOps — autonomous CI/CD pipeline powered by AI agents.
//!
//! Closed loop: PR → AI Review → Auto-Fix → Merge Gate → Deploy → Verify

pub mod auto_fixer;
pub mod deploy_verify;
pub mod merge_gate;
pub mod pipeline;
pub mod pr_reviewer;
