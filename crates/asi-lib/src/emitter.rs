use std::collections::HashMap;
use std::sync::{Arc, Mutex};

type Callback<E> = Arc<dyn Fn(&E) + Send + Sync + 'static>;

/// A lightweight typed event emitter.
pub struct Emitter<E> {
    listeners: Mutex<HashMap<String, Vec<Callback<E>>>>,
}

impl<E: 'static> Emitter<E> {
    /// Create a new empty emitter.
    pub fn new() -> Self {
        Self {
            listeners: Mutex::new(HashMap::new()),
        }
    }

    /// Register a listener for the given event.
    pub fn on<F>(&self, event: &str, callback: F)
    where
        F: Fn(&E) + Send + Sync + 'static,
    {
        let mut map = self.listeners.lock().unwrap();
        map.entry(event.to_string())
            .or_default()
            .push(Arc::new(callback));
    }

    /// Remove all listeners for the given event.
    pub fn off(&self, event: &str) {
        let mut map = self.listeners.lock().unwrap();
        map.remove(event);
    }

    /// Emit an event, calling all registered listeners with the data.
    pub fn emit(&self, event: &str, data: &E) {
        let map = self.listeners.lock().unwrap();
        if let Some(callbacks) = map.get(event) {
            for cb in callbacks {
                cb(data);
            }
        }
    }
}

impl<E: 'static> Default for Emitter<E> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn test_emit_calls_listener() {
        let emitter: Emitter<&str> = Emitter::new();
        let called = Arc::new(AtomicBool::new(false));
        let called_clone = called.clone();

        emitter.on("test", move |_| {
            called_clone.store(true, Ordering::SeqCst);
        });

        emitter.emit("test", &"hello");
        assert!(called.load(Ordering::SeqCst));
    }

    #[test]
    fn test_off_removes_listener() {
        let emitter: Emitter<&str> = Emitter::new();
        let called = Arc::new(AtomicBool::new(false));
        let called_clone = called.clone();

        emitter.on("test", move |_| {
            called_clone.store(true, Ordering::SeqCst);
        });

        emitter.off("test");
        emitter.emit("test", &"hello");
        assert!(!called.load(Ordering::SeqCst));
    }

    #[test]
    fn test_emit_passes_data() {
        let emitter: Emitter<String> = Emitter::new();
        let received = Arc::new(Mutex::new(String::new()));
        let received_clone = received.clone();

        emitter.on("data", move |data: &String| {
            *received_clone.lock().unwrap() = data.clone();
        });

        emitter.emit("data", &"payload".to_string());
        assert_eq!(*received.lock().unwrap(), "payload");
    }
}
