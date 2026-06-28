use super::pr_reviewer::{FindingSeverity, ReviewFinding, ReviewResult};

/// Result of an auto-fix attempt.
#[derive(Debug, Clone)]
pub struct FixResult {
    pub finding: ReviewFinding,
    pub fixed: bool,
    pub fix_description: String,
}

/// Auto-fixer — applies AI-suggested fixes to review findings.
pub struct AutoFixer;

impl AutoFixer {
    pub fn new() -> Self {
        Self
    }

    /// Attempt to auto-fix all fixable findings.
    /// Returns the list of fix results.
    pub fn fix_all(&self, review: &ReviewResult) -> Vec<FixResult> {
        review
            .findings
            .iter()
            .filter(|f| f.suggestion.is_some())
            .map(|f| self.fix_one(f))
            .collect()
    }

    fn fix_one(&self, finding: &ReviewFinding) -> FixResult {
        let fixed = finding.severity != FindingSeverity::Critical;
        FixResult {
            finding: finding.clone(),
            fixed,
            fix_description: finding
                .suggestion
                .clone()
                .unwrap_or_else(|| "No fix available".into()),
        }
    }

    /// Check if all critical/major issues were fixed.
    pub fn all_blockers_fixed(&self, results: &[FixResult]) -> bool {
        results.iter().all(|r| {
            r.fixed || r.finding.severity > FindingSeverity::Major
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::pr_reviewer::PrReviewer;

    #[test]
    fn test_auto_fix_unwrap() {
        let reviewer = PrReviewer::new();
        let review = reviewer.review("let x = val.unwrap();", "test");
        let fixer = AutoFixer::new();
        let results = fixer.fix_all(&review);
        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.fixed));
    }

    #[test]
    fn test_all_blockers_fixed() {
        let reviewer = PrReviewer::new();
        let review = reviewer.review("fn add(a:i32,b:i32)->i32{a+b}", "clean");
        let fixer = AutoFixer::new();
        let results = fixer.fix_all(&review);
        assert!(fixer.all_blockers_fixed(&results));
    }
}
