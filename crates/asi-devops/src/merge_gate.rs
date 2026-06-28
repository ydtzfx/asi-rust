/// Merge gate decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MergeDecision {
    /// Auto-merge: all checks passed.
    AutoMerge,
    /// Manual review required: issues found that couldn't be auto-fixed.
    ManualReview(String),
    /// Blocked: critical issues remain.
    Blocked(String),
}

/// Merge gate — determines if a PR can be auto-merged.
pub struct MergeGate {
    pub require_tests_pass: bool,
    pub require_lint_pass: bool,
    pub require_auto_fix_complete: bool,
}

impl MergeGate {
    pub fn new() -> Self {
        Self {
            require_tests_pass: true,
            require_lint_pass: true,
            require_auto_fix_complete: true,
        }
    }

    /// Evaluate all gates and return the merge decision.
    pub fn evaluate(
        &self,
        tests_pass: bool,
        lint_pass: bool,
        auto_fix_complete: bool,
        review_approved: bool,
    ) -> MergeDecision {
        let mut blockers = Vec::new();

        if self.require_tests_pass && !tests_pass {
            blockers.push("Tests failed".to_string());
        }
        if self.require_lint_pass && !lint_pass {
            blockers.push("Lint failed".to_string());
        }
        if self.require_auto_fix_complete && !auto_fix_complete {
            blockers.push("Auto-fix incomplete".to_string());
        }
        if !review_approved {
            blockers.push("Review not approved".to_string());
        }

        if blockers.is_empty() {
            MergeDecision::AutoMerge
        } else if blockers.len() <= 1 {
            MergeDecision::ManualReview(blockers.join(", "))
        } else {
            MergeDecision::Blocked(blockers.join("; "))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_pass_auto_merge() {
        let gate = MergeGate::new();
        assert_eq!(
            gate.evaluate(true, true, true, true),
            MergeDecision::AutoMerge
        );
    }

    #[test]
    fn test_failed_tests_block() {
        let gate = MergeGate::new();
        assert_eq!(
            gate.evaluate(false, true, true, true),
            MergeDecision::ManualReview("Tests failed".into())
        );
    }

    #[test]
    fn test_multiple_failures_block() {
        let gate = MergeGate::new();
        let result = gate.evaluate(false, false, false, false);
        assert!(matches!(result, MergeDecision::Blocked(_)));
    }
}
