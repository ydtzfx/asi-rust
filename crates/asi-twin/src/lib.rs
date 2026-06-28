//! ASI Twin — digital twin laboratory.
//! Production-mirror sandbox for safe experimentation and validation.
pub mod sandbox;
pub mod simulation;
pub mod validator;

use serde::{Deserialize, Serialize};

/// A digital twin replica of the production system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigitalTwin { pub id: String, pub source: String, pub state_snapshot: String, pub created_at: u64 }

/// Result of a simulation run on the twin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult { pub success: bool, pub anomalies: Vec<String>, pub metrics_diff: Vec<(String, f64, f64)> }

#[test]
fn test_create_twin() { let t = DigitalTwin { id: "tw1".into(), source: "prod".into(), state_snapshot: "{}".into(), created_at: 1 }; assert_eq!(t.source, "prod"); }
