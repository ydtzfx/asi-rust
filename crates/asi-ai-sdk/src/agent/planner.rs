use serde::{Deserialize, Serialize};

/// A sub-task in a goal decomposition plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubTask {
    pub id: String,
    pub description: String,
    pub agent_type: AgentType,
    /// IDs of tasks that must complete before this one.
    pub depends_on: Vec<String>,
    pub status: TaskStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentType {
    Code,
    Review,
    Architect,
    Search,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed(String),
}

/// A goal plan: a DAG of sub-tasks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub goal: String,
    pub tasks: Vec<SubTask>,
}

impl Plan {
    /// Create a simple linear plan (no dependencies).
    pub fn linear(goal: &str, descriptions: &[&str], agent_type: AgentType) -> Self {
        let tasks: Vec<SubTask> = descriptions
            .iter()
            .enumerate()
            .map(|(i, desc)| {
                let deps: Vec<String> = if i > 0 {
                    vec![format!("task_{}", i - 1)]
                } else {
                    vec![]
                };
                SubTask {
                    id: format!("task_{}", i),
                    description: desc.to_string(),
                    agent_type,
                    depends_on: deps,
                    status: TaskStatus::Pending,
                }
            })
            .collect();

        Self {
            goal: goal.to_string(),
            tasks,
        }
    }

    /// Get the next ready task (all dependencies completed).
    pub fn next_ready(&self) -> Option<&SubTask> {
        self.tasks.iter().find(|t| {
            t.status == TaskStatus::Pending
                && t.depends_on.iter().all(|dep_id| {
                    self.tasks
                        .iter()
                        .any(|dt| dt.id == *dep_id && dt.status == TaskStatus::Completed)
                })
        })
    }

    /// Mark a task as completed.
    pub fn complete(&mut self, task_id: &str) {
        if let Some(t) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            t.status = TaskStatus::Completed;
        }
    }

    /// Mark a task as failed.
    pub fn fail(&mut self, task_id: &str, error: &str) {
        if let Some(t) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            t.status = TaskStatus::Failed(error.to_string());
        }
    }

    /// Whether all tasks are resolved (completed or failed).
    pub fn is_done(&self) -> bool {
        self.tasks.iter().all(|t| {
            matches!(t.status, TaskStatus::Completed | TaskStatus::Failed(_))
        })
    }

    /// Progress summary.
    pub fn progress(&self) -> String {
        let total = self.tasks.len();
        let completed = self
            .tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Completed)
            .count();
        let failed = self
            .tasks
            .iter()
            .filter(|t| matches!(t.status, TaskStatus::Failed(_)))
            .count();
        format!("{}/{} done, {} failed", completed, total, failed)
    }
}

/// Decompose a user goal into a plan using simple heuristics.
pub fn decompose_goal(goal: &str) -> Plan {
    let goal_lower = goal.to_lowercase();

    if goal_lower.contains("build") || goal_lower.contains("create") || goal_lower.contains("write")
    {
        Plan::linear(
            goal,
            &[
                "Analyze requirements and design architecture",
                "Implement the core logic",
                "Write tests",
                "Review and refine",
            ],
            AgentType::Code,
        )
    } else if goal_lower.contains("fix") || goal_lower.contains("debug") || goal_lower.contains("bug")
    {
        Plan::linear(
            goal,
            &[
                "Search codebase for relevant code",
                "Identify root cause",
                "Implement fix",
                "Verify fix with tests",
            ],
            AgentType::Code,
        )
    } else if goal_lower.contains("review") || goal_lower.contains("audit") {
        Plan::linear(
            goal,
            &[
                "Scan codebase structure",
                "Review for issues",
                "Generate report",
            ],
            AgentType::Review,
        )
    } else {
        // Default: single-step plan.
        Plan::linear(goal, &["Execute request"], AgentType::Code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_plan_dependencies() {
        let plan = Plan::linear("test", &["A", "B", "C"], AgentType::Code);
        assert_eq!(plan.tasks.len(), 3);
        assert!(plan.tasks[0].depends_on.is_empty());
        assert_eq!(plan.tasks[1].depends_on, vec!["task_0"]);
        assert_eq!(plan.tasks[2].depends_on, vec!["task_1"]);
    }

    #[test]
    fn test_next_ready_respects_deps() {
        let mut plan = Plan::linear("test", &["A", "B"], AgentType::Code);
        let first = plan.next_ready().unwrap();
        assert_eq!(first.id, "task_0");
        plan.complete("task_0");
        let second = plan.next_ready().unwrap();
        assert_eq!(second.id, "task_1");
    }

    #[test]
    fn test_decompose_build_goal() {
        let plan = decompose_goal("build a REST API");
        assert!(plan.tasks.len() >= 3);
    }
}
