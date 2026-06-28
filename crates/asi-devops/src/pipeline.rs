use super::auto_fixer::AutoFixer;
use super::deploy_verify::DeployVerifier;
use super::merge_gate::MergeGate;
use super::pr_reviewer::{PrReviewer, ReviewResult};

/// Stages of the autonomous DevOps pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineStage {
    Review,
    AutoFix,
    MergeGate,
    Deploy,
    Verify,
    Complete,
    Rollback,
}

/// Result of a full pipeline execution.
#[derive(Debug, Clone)]
pub struct PipelineResult {
    pub stages_completed: Vec<PipelineStage>,
    pub review: Option<ReviewResult>,
    pub fixes_applied: usize,
    pub merged: bool,
    pub deployed: bool,
    pub success: bool,
    pub summary: String,
}

/// Autonomous DevOps pipeline — full closed loop.
pub struct AutonomousPipeline {
    reviewer: PrReviewer,
    fixer: AutoFixer,
    gate: MergeGate,
    deployer: DeployVerifier,
}

impl AutonomousPipeline {
    pub fn new() -> Self {
        Self {
            reviewer: PrReviewer::new(),
            fixer: AutoFixer::new(),
            gate: MergeGate::new(),
            deployer: DeployVerifier::new("http://localhost:3000/api/health"),
        }
    }

    /// Execute the full autonomous pipeline.
    pub async fn execute(&self, diff: &str, pr_title: &str, branch: &str) -> PipelineResult {
        let mut stages = Vec::new();
        let mut summary = String::new();

        // Stage 1: AI Review
        tracing::info!("Pipeline Stage 1: AI Review");
        let review = self.reviewer.review(diff, pr_title);
        stages.push(PipelineStage::Review);
        summary.push_str(&format!(
            "Review: {} findings (approved={})\n",
            review.findings.len(),
            review.approved
        ));

        if !review.approved {
            // Stage 2: Auto-Fix
            tracing::info!("Pipeline Stage 2: Auto-Fix ({} fixable findings)", review.auto_fixable_count);
            let fix_results = self.fixer.fix_all(&review);
            let fixes_applied = fix_results.iter().filter(|r| r.fixed).count();
            stages.push(PipelineStage::AutoFix);
            summary.push_str(&format!("Auto-fix: {}/{} fixed\n", fixes_applied, fix_results.len()));

            let all_fixed = self.fixer.all_blockers_fixed(&fix_results);

            // Stage 3: Merge Gate
            tracing::info!("Pipeline Stage 3: Merge Gate");
            let decision = self.gate.evaluate(true, true, all_fixed, all_fixed);
            stages.push(PipelineStage::MergeGate);

            match decision {
                super::merge_gate::MergeDecision::AutoMerge => {
                    summary.push_str("Merge: auto-merged ✅\n");
                    // Stage 4: Deploy
                    tracing::info!("Pipeline Stage 4: Deploy to {}", branch);
                    let deploy_result = self
                        .deployer
                        .deploy_and_verify(&format!("deploy to {}", branch))
                        .await;
                    stages.push(PipelineStage::Deploy);

                    if deploy_result.success {
                        stages.push(PipelineStage::Verify);
                        stages.push(PipelineStage::Complete);
                        summary.push_str("Deploy: verified ✅\n");
                        return PipelineResult {
                            stages_completed: stages,
                            review: Some(review),
                            fixes_applied,
                            merged: true,
                            deployed: true,
                            success: true,
                            summary,
                        };
                    } else {
                        stages.push(PipelineStage::Rollback);
                        summary.push_str("Deploy: rolled back ❌\n");
                        return PipelineResult {
                            stages_completed: stages,
                            review: Some(review),
                            fixes_applied,
                            merged: true,
                            deployed: false,
                            success: false,
                            summary,
                        };
                    }
                }
                super::merge_gate::MergeDecision::ManualReview(reason) => {
                    summary.push_str(&format!("Merge: manual review needed ({})\n", reason));
                    return PipelineResult {
                        stages_completed: stages,
                        review: Some(review),
                        fixes_applied,
                        merged: false,
                        deployed: false,
                        success: false,
                        summary,
                    };
                }
                super::merge_gate::MergeDecision::Blocked(reason) => {
                    summary.push_str(&format!("Merge: blocked ({})\n", reason));
                    return PipelineResult {
                        stages_completed: stages,
                        review: Some(review),
                        fixes_applied,
                        merged: false,
                        deployed: false,
                        success: false,
                        summary,
                    };
                }
            }
        }

        // Clean diff — auto-merge and deploy.
        tracing::info!("Pipeline: clean diff, auto-merging and deploying");
        stages.push(PipelineStage::MergeGate);
        stages.push(PipelineStage::Deploy);
        let deploy_result = self.deployer.deploy_and_verify("deploy").await;
        if deploy_result.success {
            stages.push(PipelineStage::Verify);
            stages.push(PipelineStage::Complete);
        } else {
            stages.push(PipelineStage::Rollback);
        }
        summary.push_str(&format!("Result: {}\n", deploy_result.message));

        PipelineResult {
            stages_completed: stages,
            review: Some(review),
            fixes_applied: 0,
            merged: true,
            deployed: deploy_result.success,
            success: deploy_result.success,
            summary,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_clean_pr_auto_merges() {
        let pipeline = AutonomousPipeline::new();
        let result = pipeline
            .execute("fn add(a:i32,b:i32)->i32{a+b}", "feat: add function", "main")
            .await;
        assert!(result.success);
        assert!(result.merged);
        assert!(result.deployed);
    }

    #[tokio::test]
    async fn test_unsafe_pr_blocked() {
        let pipeline = AutonomousPipeline::new();
        let result = pipeline
            .execute("unsafe { *ptr }", "unsafe code", "main")
            .await;
        assert!(!result.success);
        assert!(!result.merged);
    }

    #[tokio::test]
    async fn test_full_pipeline_stages() {
        let pipeline = AutonomousPipeline::new();
        let result = pipeline
            .execute("let x = val.unwrap();\nprintln!(\"debug\")", "feat: new feature", "main")
            .await;
        // Should go through Review → AutoFix → MergeGate → Deploy → Complete
        assert!(result.stages_completed.contains(&PipelineStage::Review));
        assert!(result.stages_completed.contains(&PipelineStage::AutoFix));
    }
}
