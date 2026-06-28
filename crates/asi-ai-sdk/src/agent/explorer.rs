//! Explorer — MCTS + Tree-of-Thought reasoning engine.
//! Explores multiple reasoning paths, evaluates intermediates, selects best.

use std::collections::HashMap;

/// A node in the reasoning tree.
#[derive(Debug, Clone)]
pub struct ThoughtNode {
    pub id: String,
    pub content: String,
    pub score: f64,           // 0.0-1.0 quality score
    pub visits: u32,          // MCTS visit count
    pub parent_id: Option<String>,
    pub children: Vec<String>,
    pub is_terminal: bool,
    pub depth: u32,
}

/// Reasoning tree — supports MCTS and Tree-of-Thought exploration.
pub struct ThoughtTree {
    nodes: HashMap<String, ThoughtNode>,
    root_id: Option<String>,
    next_id: u64,
}

impl ThoughtTree {
    pub fn new() -> Self {
        Self { nodes: HashMap::new(), root_id: None, next_id: 0 }
    }

    /// Create the root node from the problem statement.
    pub fn set_root(&mut self, problem: &str) -> String {
        let id = self.next_id();
        let node = ThoughtNode {
            id: id.clone(), content: problem.to_string(), score: 0.5,
            visits: 1, parent_id: None, children: vec![], is_terminal: false, depth: 0,
        };
        self.nodes.insert(id.clone(), node);
        self.root_id = Some(id.clone());
        id
    }

    /// Add a child thought to a parent node.
    pub fn branch(&mut self, parent_id: &str, thought: &str, score: f64) -> String {
        let child_id = self.next_id();
        let parent_depth = self.nodes.get(parent_id).map(|n| n.depth).unwrap_or(0);
        let node = ThoughtNode {
            id: child_id.clone(), content: thought.to_string(), score,
            visits: 1, parent_id: Some(parent_id.to_string()),
            children: vec![], is_terminal: score > 0.95, depth: parent_depth + 1,
        };
        if let Some(parent) = self.nodes.get_mut(parent_id) {
            parent.children.push(child_id.clone());
        }
        self.nodes.insert(child_id.clone(), node);
        child_id
    }

    /// Select the best leaf to expand (MCTS UCB1 selection from root).
    pub fn select_best_leaf(&self) -> Option<String> {
        let root_id = self.root_id.as_ref()?;
        let mut current = root_id.clone();
        loop {
            let node = self.nodes.get(&current)?;
            if node.children.is_empty() { return Some(current); }
            // UCB1: pick child with highest score + exploration bonus
            current = node.children.iter()
                .filter_map(|cid| self.nodes.get(cid).map(|c| (cid.clone(), self.ucb1(c, node.visits))))
                .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(id, _)| id).unwrap_or_default();
        }
    }

    fn ucb1(&self, node: &ThoughtNode, parent_visits: u32) -> f64 {
        let exploitation = node.score;
        let exploration = (2.0 * (parent_visits as f64).ln() / node.visits.max(1) as f64).sqrt();
        exploitation + exploration
    }

    /// Back-propagate a score up the tree.
    pub fn backpropagate(&mut self, leaf_id: &str, score: f64) {
        let mut current = Some(leaf_id.to_string());
        while let Some(id) = current {
            if let Some(node) = self.nodes.get_mut(&id) {
                node.visits += 1;
                node.score = (node.score * (node.visits - 1) as f64 + score) / node.visits as f64;
                current = node.parent_id.clone();
            } else { break; }
        }
    }

    /// Get the best path from root to highest-scoring leaf.
    pub fn best_path(&self) -> Vec<String> {
        let root_id = match &self.root_id { Some(id) => id.clone(), None => return vec![] };
        let mut path = vec![];
        let mut current = root_id;
        loop {
            let node = match self.nodes.get(&current) { Some(n) => n, None => break };
            path.push(node.content.clone());
            if node.children.is_empty() { break; }
            current = node.children.iter()
                .filter_map(|cid| self.nodes.get(cid).map(|c| (cid.clone(), c.score)))
                .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(id, _)| id).unwrap_or_default();
        }
        path
    }

    /// Total nodes in the tree.
    pub fn size(&self) -> usize { self.nodes.len() }

    /// Average score across all nodes.
    pub fn avg_score(&self) -> f64 {
        if self.nodes.is_empty() { return 0.0; }
        self.nodes.values().map(|n| n.score).sum::<f64>() / self.nodes.len() as f64
    }

    fn next_id(&mut self) -> String {
        let id = format!("thought_{}", self.next_id);
        self.next_id += 1;
        id
    }
}

/// Simple heuristic to score a thought's quality.
pub fn score_thought(thought: &str) -> f64 {
    let mut score: f64 = 0.5;
    if thought.contains("therefore") || thought.contains("结论") { score += 0.15; }
    if thought.contains("because") || thought.contains("因为") { score += 0.1; }
    if thought.contains("step") || thought.contains("步骤") { score += 0.1; }
    if thought.len() > 50 { score += 0.1; }
    if thought.len() > 200 { score += 0.05; }
    score.min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thought_tree_branch_and_select() {
        let mut tree = ThoughtTree::new();
        let root = tree.set_root("How to optimize this function?");
        tree.branch(&root, "Profile first to find bottleneck", 0.7);
        tree.branch(&root, "Try algorithmic improvement", 0.8);
        tree.branch(&root, "Add caching layer", 0.6);

        let leaf = tree.select_best_leaf();
        assert!(leaf.is_some()); // Should select one of the three branches
        assert_eq!(tree.size(), 4);
    }

    #[test]
    fn test_mcts_backpropagate() {
        let mut tree = ThoughtTree::new();
        let root = tree.set_root("problem");
        let a = tree.branch(&root, "approach A", 0.6);
        tree.branch(&a, "A step 1", 0.7);
        tree.backpropagate(&a, 0.9);
        // Root score should be updated
        assert!(tree.avg_score() > 0.6);
    }

    #[test]
    fn test_best_path() {
        let mut tree = ThoughtTree::new();
        let root = tree.set_root("solve X");
        let a = tree.branch(&root, "good path", 0.9);
        tree.branch(&root, "bad path", 0.3);
        tree.branch(&a, "excellent conclusion", 0.95);
        let path = tree.best_path();
        assert_eq!(path.len(), 3);
        assert!(path[1].contains("good path"));
    }

    #[test]
    fn test_score_thought() {
        assert!(score_thought("hello") > 0.4);
        assert!(score_thought("I think therefore the answer is 42") > 0.6);
    }
}
