/// Property-based tests for agent types and utilities.
use asi_ai_sdk::types::{Message, Role};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

fn conversation_hash(messages: &[Message]) -> String {
    let mut hasher = DefaultHasher::new();
    for msg in messages {
        msg.role.hash(&mut hasher);
        msg.content.hash(&mut hasher);
    }
    format!("{:x}", hasher.finish())
}

#[test]
fn property_same_input_same_hash() {
    let msgs1 = vec![
        Message {
            role: Role::User,
            content: "hello".into(),
            tool_calls: None,
            tool_call_id: None,
        },
        Message {
            role: Role::Assistant,
            content: "hi".into(),
            tool_calls: None,
            tool_call_id: None,
        },
    ];
    let msgs2 = msgs1.clone();
    assert_eq!(conversation_hash(&msgs1), conversation_hash(&msgs2));
}

#[test]
fn property_different_input_different_hash() {
    let msgs1 = vec![Message {
        role: Role::User,
        content: "hello".into(),
        tool_calls: None,
        tool_call_id: None,
    }];
    let msgs2 = vec![Message {
        role: Role::User,
        content: "world".into(),
        tool_calls: None,
        tool_call_id: None,
    }];
    assert_ne!(conversation_hash(&msgs1), conversation_hash(&msgs2));
}

#[test]
fn property_order_matters() {
    let msgs1 = vec![
        Message {
            role: Role::User,
            content: "a".into(),
            tool_calls: None,
            tool_call_id: None,
        },
        Message {
            role: Role::User,
            content: "b".into(),
            tool_calls: None,
            tool_call_id: None,
        },
    ];
    let msgs2 = vec![
        Message {
            role: Role::User,
            content: "b".into(),
            tool_calls: None,
            tool_call_id: None,
        },
        Message {
            role: Role::User,
            content: "a".into(),
            tool_calls: None,
            tool_call_id: None,
        },
    ];
    // Different order should produce different hash
    assert_ne!(conversation_hash(&msgs1), conversation_hash(&msgs2));
}

#[test]
fn property_empty_gives_consistent_hash() {
    let empty: Vec<Message> = vec![];
    // Same input should always produce same hash
    assert_eq!(conversation_hash(&empty), conversation_hash(&empty));
    // Hash should not be empty
    assert!(!conversation_hash(&empty).is_empty());
}
