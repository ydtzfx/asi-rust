//! ASI Mesh — federated cognitive network.
//! Distributed intelligence across nodes with consensus and privacy.
pub mod consensus;
pub mod federated;
pub mod node;
pub mod sync;

use serde::{Deserialize, Serialize};

/// A node in the cognitive mesh.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshNode { pub id: String, pub region: String, pub capacity: f64, pub load: f64, pub peers: Vec<String> }

/// Shared knowledge fragment — privacy-preserving.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeShard { pub key: String, pub embedding: Vec<f64>, pub node_count: u32, pub confidence: f64 }

#[test]
fn test_mesh_node() { let n = MeshNode { id: "n1".into(), region: "us".into(), capacity: 100.0, load: 30.0, peers: vec![] }; assert_eq!(n.region, "us"); }
