mod input_bar;
mod message_item;
mod message_list;
mod reasoning_block;
mod tool_block;

pub use input_bar::InputBar;
pub use message_item::{MessageItem, role_style};
pub use message_list::MessageList;
pub use reasoning_block::ReasoningBlock;
pub use tool_block::ToolBlock;

// ---------------------------------------------------------------------------
// Shared chat types
// ---------------------------------------------------------------------------

/// A single part inside a chat message.
#[derive(Debug, Clone)]
pub enum MessagePart {
    /// Plain text content (whitespace-pre-wrap).
    Text(String),
    /// Model reasoning / chain-of-thought (rendered as collapsible `<details>`).
    Reasoning(String),
    /// A tool call (rendered as collapsible JSON).
    ToolCall {
        id: String,
        name: String,
        arguments: serde_json::Value,
        result: Option<String>,
    },
}

/// A single chat message in the conversation.
#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub id: String,
    pub role: String,
    pub content: String,
    pub parts: Vec<MessagePart>,
}

impl ChatMessage {
    pub fn new_user(id: String, content: String) -> Self {
        Self {
            id,
            role: "user".into(),
            parts: vec![MessagePart::Text(content.clone())],
            content,
        }
    }

    pub fn new_assistant(id: String) -> Self {
        Self {
            id,
            role: "assistant".into(),
            parts: Vec::new(),
            content: String::new(),
        }
    }

    /// Append a text delta to the last Text part, or start a new one.
    pub fn push_text(&mut self, delta: &str) {
        self.content.push_str(delta);
        match self.parts.last_mut() {
            Some(MessagePart::Text(existing)) => existing.push_str(delta),
            _ => self.parts.push(MessagePart::Text(delta.to_string())),
        }
    }

    /// Append a reasoning segment.
    pub fn push_reasoning(&mut self, content: &str) {
        match self.parts.last_mut() {
            Some(MessagePart::Reasoning(existing)) => existing.push_str(content),
            _ => self.parts.push(MessagePart::Reasoning(content.to_string())),
        }
    }

    /// Append a tool-call segment.
    pub fn push_tool_call(&mut self, id: String, name: String, arguments: serde_json::Value) {
        self.parts.push(MessagePart::ToolCall {
            id,
            name,
            arguments,
            result: None,
        });
    }

    /// Attach a result to the last tool-call part whose result is still `None`.
    pub fn push_tool_result(&mut self, name: &str, result: String) {
        for part in self.parts.iter_mut().rev() {
            if let MessagePart::ToolCall {
                name: n,
                result: res @ None,
                ..
            } = part
                && n == name
            {
                *res = Some(result);
                return;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Colour helpers
// ---------------------------------------------------------------------------

/// Returns the Tailwind text colour class for a given role.
pub fn message_part_color(role: &str) -> &'static str {
    match role {
        "user" => "text-blue-600",
        "assistant" => "text-gray-800",
        _ => "text-gray-500",
    }
}
