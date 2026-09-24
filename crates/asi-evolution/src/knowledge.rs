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

impl Default for KnowledgeBase {
    fn default() -> Self {
        Self::new()
    }
}

impl KnowledgeBase {
    pub fn new() -> Self {
        Self {
            insights: Mutex::new(HashMap::new()),
        }
    }

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

    pub fn get(&self, key: &str) -> Option<Insight> {
        self.insights.lock().unwrap().get(key).cloned()
    }

    pub fn search(&self, query: &str) -> Vec<Insight> {
        let map = self.insights.lock().unwrap();
        let q = query.to_lowercase();
        map.values()
            .filter(|i| i.key.to_lowercase().contains(&q) || i.content.to_lowercase().contains(&q))
            .cloned()
            .collect()
    }

    pub fn top(&self, n: usize) -> Vec<Insight> {
        let map = self.insights.lock().unwrap();
        let mut items: Vec<_> = map.values().cloned().collect();
        items.sort_by_key(|item| std::cmp::Reverse(item.times_used));
        items.truncate(n);
        items
    }

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
        let i = kb.get("rust_edition").unwrap();
        assert_eq!(i.content, "This project uses Rust edition 2024");
        assert_eq!(i.times_used, 1);
    }
    #[test]
    fn test_search() {
        let kb = KnowledgeBase::new();
        kb.upsert("rust_error_handling", "Use thiserror for library errors");
        kb.upsert("rust_async", "Use tokio for async runtime");
        assert_eq!(kb.search("error").len(), 1);
    }
    #[test]
    fn test_top() {
        let kb = KnowledgeBase::new();
        kb.upsert("a", "A");
        kb.upsert("b", "B");
        kb.upsert("b", "B");
        kb.upsert("b", "B");
        assert_eq!(kb.top(1)[0].key, "b");
    }
}
