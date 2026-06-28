use serde::{Deserialize, Serialize};

/// A finding from AI code review.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewFinding {
    pub file: String,
    pub line: Option<u32>,
    pub severity: FindingSeverity,
    pub category: String,
    pub message: String,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum FindingSeverity {
    Critical,
    Major,
    Minor,
    Suggestion,
}

/// Result of an AI-powered PR review.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewResult {
    pub pr_title: String,
    pub findings: Vec<ReviewFinding>,
    pub summary: String,
    pub approved: bool,
    pub auto_fixable_count: usize,
}

/// AI PR Reviewer — analyzes code diffs and produces structured findings.
pub struct PrReviewer {
    /// Severity threshold for auto-approval (at most Minor findings).
    pub auto_approve_threshold: FindingSeverity,
}

impl PrReviewer {
    pub fn new() -> Self {
        Self {
            auto_approve_threshold: FindingSeverity::Minor,
        }
    }

    /// Review a diff and return findings.
    /// In production, this would call the AI provider with the diff.
    pub fn review(&self, _diff: &str, pr_title: &str) -> ReviewResult {
        // Simulation: in production, call LLM with the diff.
        // The agent would analyze the code and produce findings.
        let findings = self.analyze_diff(_diff);

        let critical_or_major = findings
            .iter()
            .filter(|f| {
                f.severity == FindingSeverity::Critical
                    || f.severity == FindingSeverity::Major
            })
            .count();

        let auto_fixable = findings
            .iter()
            .filter(|f| f.suggestion.is_some())
            .count();

        ReviewResult {
            pr_title: pr_title.to_string(),
            summary: format!(
                "{} findings: {} critical, {} major, {} minor, {} suggestions",
                findings.len(),
                findings.iter().filter(|f| f.severity == FindingSeverity::Critical).count(),
                findings.iter().filter(|f| f.severity == FindingSeverity::Major).count(),
                findings.iter().filter(|f| f.severity == FindingSeverity::Minor).count(),
                findings.iter().filter(|f| f.severity == FindingSeverity::Suggestion).count(),
            ),
            approved: critical_or_major == 0,
            auto_fixable_count: auto_fixable,
            findings,
        }
    }

    fn analyze_diff(&self, diff: &str) -> Vec<ReviewFinding> {
        let mut findings = Vec::new();

        // Rule-based checks as fallback (LLM provides richer analysis).
        if diff.contains("unwrap()") {
            findings.push(ReviewFinding {
                file: "unknown".into(),
                line: None,
                severity: FindingSeverity::Major,
                category: "error-handling".into(),
                message: "Avoid unwrap() in production code".into(),
                suggestion: Some("Use proper error handling with Result/thiserror".into()),
            });
        }
        if diff.contains("println!") {
            findings.push(ReviewFinding {
                file: "unknown".into(),
                line: None,
                severity: FindingSeverity::Minor,
                category: "logging".into(),
                message: "Use tracing crate instead of println!".into(),
                suggestion: Some("Replace with tracing::info! or tracing::debug!".into()),
            });
        }
        if diff.contains(".expect(") {
            findings.push(ReviewFinding {
                file: "unknown".into(),
                line: None,
                severity: FindingSeverity::Minor,
                category: "error-handling".into(),
                message: "Consider graceful error handling over .expect()".into(),
                suggestion: Some("Use proper error propagation".into()),
            });
        }
        if diff.contains("unsafe") {
            findings.push(ReviewFinding {
                file: "unknown".into(),
                line: None,
                severity: FindingSeverity::Critical,
                category: "safety".into(),
                message: "Unsafe block detected — requires justification".into(),
                suggestion: Some("Add SAFETY comment explaining why unsafe is needed".into()),
            });
        }
        if diff.contains("clone()") && diff.matches("clone()").count() > 3 {
            findings.push(ReviewFinding {
                file: "unknown".into(),
                line: None,
                severity: FindingSeverity::Suggestion,
                category: "performance".into(),
                message: "Multiple clone() calls detected — consider references".into(),
                suggestion: Some("Use & references or Arc/Rc where possible".into()),
            });
        }

        findings
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_review_detects_unwrap() {
        let reviewer = PrReviewer::new();
        let result = reviewer.review("let x = some_option.unwrap();", "test");
        assert!(!result.findings.is_empty());
        assert!(result.findings.iter().any(|f| f.message.contains("unwrap")));
    }

    #[test]
    fn test_clean_diff_approved() {
        let reviewer = PrReviewer::new();
        let result = reviewer.review("fn add(a: i32, b: i32) -> i32 { a + b }", "test");
        assert!(result.approved);
    }

    #[test]
    fn test_unsafe_blocks_review() {
        let reviewer = PrReviewer::new();
        let result = reviewer.review("unsafe { *ptr }", "test");
        assert!(!result.approved); // Critical finding
    }
}
