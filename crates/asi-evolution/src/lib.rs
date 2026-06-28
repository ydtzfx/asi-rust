//! ASI Evolution — continuous self-improvement system.
//!
//! Modules:
//! - prompt_evo: Prompt mutation + fitness-based selection
//! - tool_factory: Dynamic tool creation from requirements
//! - knowledge: Accumulated reusable insights
//! - model_picker: Task-based model routing
//! - ab_test: A/B experiment framework

pub mod ab_test;
pub mod knowledge;
pub mod model_picker;
pub mod prompt_evo;
pub mod tool_factory;
