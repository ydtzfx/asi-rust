use asi_ai_sdk::runtime_handshake_v1::build_runtime_hello;
use serde_json::Value;
use std::{env, fs};

#[test]
#[ignore = "requires canonical handshake vectors from agi-platform"]
fn runtime_hello_matches_canonical_rust_profile() {
    let vector_path = env::var("AGENT_RUNTIME_HANDSHAKE_VECTORS")
        .expect("AGENT_RUNTIME_HANDSHAKE_VECTORS must point to canonical vectors");
    let vectors: Value = serde_json::from_str(
        &fs::read_to_string(vector_path).expect("read canonical handshake vectors"),
    )
    .expect("parse canonical handshake vectors");
    let profile = &vectors["hello_profiles"]["rust"];
    let features: Vec<String> = profile["input"]["features"]
        .as_array()
        .expect("features array")
        .iter()
        .map(|value| value.as_str().expect("feature string").to_string())
        .collect();
    let actual = build_runtime_hello(
        profile["input"]["runtime_id"].as_str().expect("runtime_id"),
        profile["input"]["implementation_version"]
            .as_str()
            .expect("implementation_version"),
        &features,
    );
    assert_eq!(actual, profile["expect"]);
}
