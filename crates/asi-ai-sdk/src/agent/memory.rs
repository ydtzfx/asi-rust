use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::types::Message;

/// A single memory entry.
#[derive(Debug, Clone)]
pub struct MemoryEntry {
    pub content: String,
    pub role: String,
    pub timestamp: Instant,
    /// Simple bag-of-words embedding for similarity search.
    pub tokens: Vec<String>,
}

/// Long-term memory store with TTL and similarity search.
pub struct AgentMemory {
    entries: Mutex<Vec<MemoryEntry>>,
    ttl: Duration,
    max_entries: usize,
}

impl AgentMemory {
    pub fn new(ttl: Duration, max_entries: usize) -> Self {
        Self {
            entries: Mutex::new(Vec::new()),
            ttl,
            max_entries,
        }
    }

    /// Store a message in memory.
    pub fn remember(&self, content: &str, role: &str) {
        let tokens = tokenize(content);
        let entry = MemoryEntry {
            content: content.to_string(),
            role: role.to_string(),
            timestamp: Instant::now(),
            tokens,
        };

        let mut entries = self.entries.lock().unwrap();
        entries.push(entry);

        // Evict expired entries.
        entries.retain(|e| e.timestamp.elapsed() < self.ttl);

        // Trim to max size.
        while entries.len() > self.max_entries {
            entries.remove(0);
        }
    }

    /// Search memory for entries similar to the query.
    /// Uses simple Jaccard similarity on token sets.
    pub fn recall(&self, query: &str, top_k: usize) -> Vec<String> {
        let query_tokens: Vec<String> = tokenize(query);
        let entries = self.entries.lock().unwrap();

        let mut scored: Vec<(f64, String)> = entries
            .iter()
            .map(|e| (jaccard_similarity(&query_tokens, &e.tokens), e.content.clone()))
            .collect();

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        scored
            .into_iter()
            .take(top_k)
            .filter(|(score, _)| *score > 0.0)
            .map(|(_, content)| content)
            .collect()
    }

    /// Get recent memories as a context string.
    pub fn context(&self, max_items: usize) -> String {
        let entries = self.entries.lock().unwrap();
        entries
            .iter()
            .rev()
            .take(max_items)
            .map(|e| format!("[{}]: {}", e.role, e.content))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Import conversation into memory.
    pub fn import_conversation(&self, messages: &[Message]) {
        for msg in messages {
            self.remember(&msg.content, &format!("{:?}", msg.role));
        }
    }

    pub fn size(&self) -> usize {
        self.entries.lock().unwrap().len()
    }
}

/// Simple tokenization: lowercase + split on non-alphanumeric.
fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| t.len() > 1)
        .map(|t| t.to_string())
        .collect()
}

/// Jaccard similarity between two token sets.
fn jaccard_similarity(a: &[String], b: &[String]) -> f64 {
    let set_a: HashMap<&str, usize> = a.iter().fold(HashMap::new(), |mut m, t| {
        *m.entry(t.as_str()).or_default() += 1;
        m
    });
    let set_b: HashMap<&str, usize> = b.iter().fold(HashMap::new(), |mut m, t| {
        *m.entry(t.as_str()).or_default() += 1;
        m
    });

    let intersection: usize = set_a
        .iter()
        .map(|(k, v)| v.min(set_b.get(k).unwrap_or(&0)))
        .sum();
    let union: usize = set_a.values().sum::<usize>() + set_b.values().sum::<usize>() - intersection;

    if union == 0 {
        0.0
    } else {
        intersection as f64 / union as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remember_and_recall() {
        let mem = AgentMemory::new(Duration::from_secs(3600), 100);
        mem.remember("Rust is a systems programming language", "user");
        mem.remember("Python is great for data science", "user");
        mem.remember("Axum is a Rust web framework", "assistant");

        let results = mem.recall("Rust web framework", 3);
        assert!(!results.is_empty());
        // Most relevant should be about Axum/Rust.
        assert!(results[0].contains("Axum") || results[0].contains("Rust"));
    }

    #[test]
    fn test_jaccard() {
        let a = tokenize("hello world rust programming");
        let b = tokenize("hello world python programming");
        let sim = jaccard_similarity(&a, &b);
        assert!(sim > 0.4);
        assert!(sim < 1.0);
    }
}
