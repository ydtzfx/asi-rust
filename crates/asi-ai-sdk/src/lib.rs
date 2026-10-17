//! # ASI AI SDK
//!
//! AI provider abstraction layer with multi-provider support, tool-calling
//! agent loop, and AGI reasoning capabilities.
//!
//! ## Architecture
//!
//! ```text
//! agent/          — Agent implementations (ToolLoop, Deep, Coordinator, MCTS Explorer)
//! provider/       — AI providers (Ollama, DeepSeek, Fallback)
//! types.rs        — Chat API types (Message, ChatRequest, StreamChunk)
//! token.rs        — Token counting utilities
//! protocol_v1_1.rs — isolated Agent Protocol v1.1 behavioral adapter
//! runtime_handshake_v1.rs — Runtime Handshake v1 hello builder
//! ```
//!
//! ## Stability
//!
//! The `types` and `provider::AiProvider` trait are **stable** (semver minor).
//! Agent modules are **evolving** — APIs may change across minor versions.

pub mod agent;
pub mod protocol_v1_1;
pub mod provider;
pub mod runtime_handshake_v1;
pub mod token;
pub mod types;
