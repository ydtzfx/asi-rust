//! Runtime Handshake v1 hello builder.
//!
//! This module advertises transport-neutral Runtime identity and caller-supplied
//! feature claims. Compatibility policy remains owned by the Control Plane.

use serde_json::{Value, json};

pub const HANDSHAKE_VERSION: &str = "1.0";
pub const ABI_VERSION: &str = "agent-runtime/1.1";
pub const PROTOCOL_VERSIONS: &[&str] = &["1.0", "1.1"];
pub const IMPLEMENTATION: &str = "rust";

pub fn build_runtime_hello(
    runtime_id: &str,
    implementation_version: &str,
    features: &[String],
) -> Value {
    let mut normalized_features = features.to_vec();
    normalized_features.sort();
    normalized_features.dedup();
    json!({
        "handshake_version": HANDSHAKE_VERSION,
        "runtime_id": runtime_id,
        "abi_version": ABI_VERSION,
        "protocol_versions": PROTOCOL_VERSIONS,
        "implementation": IMPLEMENTATION,
        "implementation_version": implementation_version,
        "features": normalized_features
    })
}
