//! ASI Events — event-driven architecture with pub/sub bus, event store, CQRS.
pub mod bus;
pub mod handler;
pub mod projection;
pub mod store;

use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// A domain event — immutable, append-only.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainEvent {
    pub id: String,
    pub event_type: String,
    pub aggregate_id: String,
    pub data: serde_json::Value,
    pub timestamp: u64,
    pub version: u64,
}
impl DomainEvent {
    pub fn new(event_type: &str, aggregate_id: &str, data: serde_json::Value) -> Self {
        Self { id: uuid(), event_type: event_type.into(), aggregate_id: aggregate_id.into(), data, timestamp: now(), version: 1 }
    }
}

/// Event handler trait.
#[async_trait::async_trait]
pub trait EventHandler: Send + Sync {
    async fn handle(&self, event: &DomainEvent);
    fn subscribed_to(&self) -> Vec<String>;
}

/// Event bus — pub/sub for domain events.
pub struct EventBus {
    handlers: std::sync::Mutex<Vec<Arc<dyn EventHandler>>>,
    store: std::sync::Mutex<Vec<DomainEvent>>,
}
impl EventBus {
    pub fn new() -> Self { Self { handlers: std::sync::Mutex::new(Vec::new()), store: std::sync::Mutex::new(Vec::new()) } }
    pub fn subscribe(&self, handler: Arc<dyn EventHandler>) { self.handlers.lock().unwrap().push(handler); }
    pub async fn publish(&self, event: DomainEvent) {
        self.store.lock().unwrap().push(event.clone());
        let handlers = self.handlers.lock().unwrap();
        for h in handlers.iter() {
            if h.subscribed_to().contains(&event.event_type) { h.handle(&event).await; }
        }
    }
    pub fn replay(&self, event_type: &str) -> Vec<DomainEvent> {
        self.store.lock().unwrap().iter().filter(|e| e.event_type == event_type).cloned().collect()
    }
}

fn uuid() -> String { format!("evt_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos()) }
fn now() -> u64 { std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() }

#[cfg(test)]
mod tests {
    use super::*;
    struct TestHandler;
    #[async_trait::async_trait]
    impl EventHandler for TestHandler {
        async fn handle(&self, _e: &DomainEvent) {}
        fn subscribed_to(&self) -> Vec<String> { vec!["test".into()] }
    }
    #[tokio::test]
    async fn test_publish_and_replay() {
        let bus = EventBus::new();
        bus.subscribe(Arc::new(TestHandler));
        bus.publish(DomainEvent::new("test", "agg1", serde_json::json!({"x":1}))).await;
        let events = bus.replay("test");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].aggregate_id, "agg1");
    }
}
