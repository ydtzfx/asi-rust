use std::collections::HashMap;
use std::sync::Mutex;

/// A reusable insight learned from interactions.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Insight {
    pub key: String,
    pub content: String,
    pub confidence: f64,
    pub times_used: u64,
    pub created_at: u64,
}

/// Knowledge base — accumulates insights and serves the most relevant ones.
pub struct KnowledgeBase {
    insights: Mutex<HashMap<String, Insight>>,
}

impl KnowledgeBase {
    pub fn new() -> Self {
        Self {
            insights: Mutex::new(HashMap::new()),
        }
    }

    /// Add or update an insight.
    pub fn upsert(&self, key: &str, content: &str) {
        let mut map = self.insights.lock().unwrap();
        if let Some(existing) = map.get_mut(key) {
            existing.times_used += 1;
            existing.confidence = (existing.confidence + 0.9).min(1.0);
        } else {
            map.insert(
                key.to_string(),
                Insight {
                    key: key.to_string(),
                    content: content.to_string(),
                    confidence: 0.5,
                    times_used: 1,
                    created_at: now(),
                },
            );
        }
    }

    /// Retrieve an insight by key.
    pub fn get(&self, key: &str) -> Option<Insight> {
        let map = self.insights.lock().unwrap();
        map.get(key).cloned()
    }

    /// Search insights whose key or content contains the query.
    pub fn search(&self, query: &str) -> Vec<Insight> {
        let map = self.insights.lock().unwrap();
        let q = query.to_lowercase();
        map.values()
            .filter(|i| i.key.to_lowercase().contains(&q) || i.content.to_lowercase().contains(&q))
            .cloned()
            .collect()
    }

    /// Get the top N most-used insights.
    pub fn top(&self, n: usize) -> Vec<Insight> {
        let map = self.insights.lock().unwrap();
        let mut items: Vec<_> = map.values().cloned().collect();
        items.sort_by(|a, b| b.times_used.cmp(&a.times_used));
        items.truncate(n);
        items
    }

    /// Export all insights as a context string for agent prompts.
    pub fn as_context(&self, max_items: usize) -> String {
        let top = self.top(max_items);
        if top.is_empty() {
            return String::new();
        }
        let mut ctx = String::from("## Relevant Knowledge\n\n");
        for insight in &top {
            ctx.push_str(&format!("- **{}**: {}\n", insight.key, insight.content));
        }
        ctx
    }

    pub fn size(&self) -> usize {
        self.insights.lock().unwrap().len()
    }
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upsert_and_get() {
        let kb = KnowledgeBase::new();
        kb.upsert("rust_edition", "This project uses Rust edition 2024");
        let insight = kb.get("rust_edition").unwrap();
        assert_eq!(insight.content, "This project uses Rust edition 2024");
        assert_eq!(insight.times_used, 1);
    }

    #[test]
    fn test_search() {
        let kb = KnowledgeBase::new();
        kb.upsert("rust_error_handling", "Use thiserror for library errors");
        kb.upsert("rust_async", "Use tokio for async runtime");
        let results = kb.search("error");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_top() {
        let kb = KnowledgeBase::new();
        kb.upsert("a", "A");
        kb.upsert("b", "B");
        // Use b more
        kb.upsert("b", "B");
        kb.upsert("b", "B");

        let top = kb.top(1);
        assert_eq!(top[0].key, "b");
    }
}
