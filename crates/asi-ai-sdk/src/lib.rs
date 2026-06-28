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
//! ```
//!
//! ## Stability
//!
//! The `types` and `provider::AiProvider` trait are **stable** (semver minor).
//! Agent modules are **evolving** — APIs may change across minor versions.

pub mod agent;
pub mod provider;
pub mod token;
pub mod types;
