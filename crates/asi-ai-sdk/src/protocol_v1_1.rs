//! Agent Protocol v1.1 behavioral adapter contract.
//!
//! This module is deliberately isolated from ASI business logic. It specifies
//! cross-runtime observable semantics for conformance without replacing the
//! existing ToolLoopAgent, providers, server routes, tools, or persistence.

use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

pub const PROTOCOL_VERSION: &str = "1.1";

const SENSITIVE_SEGMENTS: &[&str] = &[".git", ".ssh", ".aws", ".kube"];
const PRIVATE_KEY_NAMES: &[&str] = &["id_rsa", "id_dsa", "id_ecdsa", "id_ed25519"];
const PRIVATE_KEY_SUFFIXES: &[&str] = &[".pem", ".key", ".p12", ".pfx"];

fn segments(value: &str) -> Vec<&str> {
    value
        .split(['/', '\\'])
        .filter(|part| !part.is_empty() && *part != ".")
        .collect()
}

fn normalized_absolute(value: &str) -> String {
    let mut out: Vec<&str> = Vec::new();
    for part in segments(value) {
        if part == ".." {
            out.pop();
        } else {
            out.push(part);
        }
    }
    format!("/{}", out.join("/"))
}

fn within(root: &str, candidate: &str) -> bool {
    let root = normalized_absolute(root).trim_end_matches('/').to_string();
    let candidate = normalized_absolute(candidate);
    candidate == root || candidate.starts_with(&format!("{root}/"))
}

pub fn authorize_workspace_path(workspace: &str, requested_path: &str) -> Value {
    let mut relative: Vec<&str> = Vec::new();
    for part in segments(requested_path) {
        if part == ".." {
            if relative.is_empty() {
                return json!({"allowed": false, "code": "PATH_TRAVERSAL"});
            }
            relative.pop();
        } else {
            relative.push(part);
        }
    }
    let root = normalized_absolute(workspace)
        .trim_end_matches('/')
        .to_string();
    json!({
        "allowed": true,
        "code": Value::Null,
        "resolved": format!("{root}/{}", relative.join("/"))
    })
}

pub fn authorize_resolved_path(
    workspace: &str,
    requested_path: &str,
    resolved_path: &str,
) -> Value {
    let lexical = authorize_workspace_path(workspace, requested_path);
    if lexical["allowed"] == Value::Bool(false) {
        return lexical;
    }
    if !within(workspace, resolved_path) {
        return json!({"allowed": false, "code": "SYMLINK_ESCAPE"});
    }
    json!({
        "allowed": true,
        "code": Value::Null,
        "resolved": lexical["resolved"].clone(),
        "real_path": normalized_absolute(resolved_path)
    })
}

pub fn authorize_non_sensitive_path(workspace: &str, requested_path: &str) -> Value {
    let lexical = authorize_workspace_path(workspace, requested_path);
    if lexical["allowed"] == Value::Bool(false) {
        return lexical;
    }
    for segment in segments(requested_path) {
        let lower = segment.to_ascii_lowercase();
        let sensitive = SENSITIVE_SEGMENTS.contains(&lower.as_str())
            || lower == ".env"
            || lower.starts_with(".env.")
            || PRIVATE_KEY_NAMES.contains(&lower.as_str())
            || PRIVATE_KEY_SUFFIXES
                .iter()
                .any(|suffix| lower.ends_with(suffix));
        if sensitive {
            return json!({"allowed": false, "code": "SENSITIVE_PATH", "segment": segment});
        }
    }
    lexical
}

pub fn authorize_capabilities(required: &[String], granted: &[String]) -> Value {
    let missing: Vec<String> = required
        .iter()
        .filter(|capability| !granted.contains(capability))
        .cloned()
        .collect();
    json!({"allowed": missing.is_empty(), "missing": missing})
}

pub fn authorize_tool(tool_name: &str, declared_tools: &[String]) -> Value {
    if declared_tools.iter().any(|tool| tool == tool_name) {
        json!({"allowed": true, "code": Value::Null})
    } else {
        json!({"allowed": false, "code": "UNDECLARED_TOOL"})
    }
}

pub fn apply_approval_gate(run_state: &Value, approval: &Value, now: &str) -> Value {
    if approval["required"] != Value::Bool(true) {
        return run_state.clone();
    }
    let mut result = run_state.clone();
    let object = result.as_object_mut().expect("run state must be an object");
    let sequence = run_state["sequence"]
        .as_u64()
        .expect("sequence must be u64");
    object.insert("state".into(), json!("waiting_approval"));
    object.insert("sequence".into(), json!(sequence + 1));
    object.insert("reason_code".into(), json!("APPROVAL_REQUIRED"));
    object.insert(
        "reason".into(),
        json!(format!(
            "Approval required for {}",
            approval["category"].as_str().unwrap_or("unknown")
        )),
    );
    object.insert("updated_at".into(), json!(now));
    result
}

pub fn resume_approved_run(run_state: &Value, approval: &Value) -> Result<Value, String> {
    if run_state["state"] != Value::String("waiting_approval".into()) {
        return Err("RUN_NOT_WAITING_APPROVAL".into());
    }
    if approval["status"] != Value::String("approved".into()) {
        return Err("APPROVAL_NOT_GRANTED".into());
    }
    if approval["run_id"] != run_state["run_id"] {
        return Err("APPROVAL_RUN_MISMATCH".into());
    }
    let mut result = run_state.clone();
    let object = result.as_object_mut().expect("run state must be an object");
    let sequence = run_state["sequence"]
        .as_u64()
        .expect("sequence must be u64");
    object.insert("state".into(), json!("running"));
    object.insert("sequence".into(), json!(sequence + 1));
    object.insert("reason_code".into(), json!("APPROVAL_GRANTED"));
    object.insert(
        "reason".into(),
        json!(format!(
            "Approval {} granted",
            approval["approval_id"].as_str().unwrap_or("unknown")
        )),
    );
    object.insert(
        "updated_at".into(),
        approval
            .get("decided_at")
            .cloned()
            .unwrap_or_else(|| run_state["updated_at"].clone()),
    );
    Ok(result)
}

pub fn apply_tool_timeout(run_state: &Value, tool_call_id: &str, now: &str) -> Value {
    let mut result = run_state.clone();
    let object = result.as_object_mut().expect("run state must be an object");
    let sequence = run_state["sequence"]
        .as_u64()
        .expect("sequence must be u64");
    object.insert("state".into(), json!("failed"));
    object.insert("sequence".into(), json!(sequence + 1));
    object.insert("reason_code".into(), json!("TOOL_TIMEOUT"));
    object.insert(
        "reason".into(),
        json!(format!("Tool call {tool_call_id} timed out")),
    );
    object.insert("updated_at".into(), json!(now));
    result
}

pub fn execute_cooperative_steps(max_steps: usize, cancel_after_step: usize) -> Value {
    let mut visited = Vec::new();
    for index in 0..max_steps {
        if index > cancel_after_step {
            return json!({"status": "cancelled", "steps": visited.len(), "visited": visited});
        }
        visited.push(index);
    }
    json!({"status": "completed", "steps": visited.len(), "visited": visited})
}

fn canonicalize(value: &Value) -> Value {
    match value {
        Value::Array(values) => Value::Array(values.iter().map(canonicalize).collect()),
        Value::Object(values) => {
            let mut keys: Vec<&String> = values.keys().collect();
            keys.sort();
            let mut output = Map::new();
            for key in keys {
                output.insert(key.clone(), canonicalize(&values[key]));
            }
            Value::Object(output)
        }
        _ => value.clone(),
    }
}

pub fn canonical_json(value: &Value) -> String {
    serde_json::to_string(&canonicalize(value)).expect("canonical JSON must serialize")
}

pub fn canonical_digest(value: &Value) -> String {
    let digest = Sha256::digest(canonical_json(value).as_bytes());
    format!("sha256:{digest:x}")
}

pub fn build_redacted_audit_event(input: &Value) -> Value {
    let sensitive = input["sensitive_arguments"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    json!({
        "protocol_version": PROTOCOL_VERSION,
        "event_id": input["event_id"].clone(),
        "timestamp": input["timestamp"].clone(),
        "trace_id": input["trace_id"].clone(),
        "run_id": input["run_id"].clone(),
        "tool_call_id": input["tool_call_id"].clone(),
        "event_type": "tool",
        "stage": "requested",
        "actor": {"type": "runtime", "id": input["actor_id"].clone()},
        "tool": input["tool"].clone(),
        "capabilities": input["required_capabilities"].clone(),
        "input_digest": canonical_digest(&input["args"]),
        "redacted_fields": sensitive,
        "attributes": {"redaction_count": input["sensitive_arguments"].as_array().map_or(0, Vec::len)}
    })
}

pub fn wrap_tool_output(tool_call_id: &str, output: &Value) -> Value {
    json!({
        "source": "tool",
        "tool_call_id": tool_call_id,
        "trust": "untrusted",
        "interpretation": "data",
        "data": output
    })
}

pub fn require_trusted_control_input(envelope: &Value) -> Result<Value, String> {
    if envelope["trust"] != Value::String("trusted".into()) {
        return Err("UNTRUSTED_TOOL_OUTPUT".into());
    }
    Ok(envelope["data"].clone())
}

#[derive(Debug, Clone)]
pub struct StepBudget {
    limit: usize,
    used: usize,
}

impl StepBudget {
    pub fn new(limit: usize) -> Result<Self, String> {
        if limit == 0 {
            return Err("INVALID_STEP_BUDGET".into());
        }
        Ok(Self { limit, used: 0 })
    }

    pub fn consume(&mut self) -> Value {
        if self.used >= self.limit {
            return json!({
                "allowed": false,
                "code": "STEP_BUDGET_EXCEEDED",
                "used": self.used,
                "limit": self.limit
            });
        }
        self.used += 1;
        json!({"allowed": true, "code": Value::Null, "used": self.used, "limit": self.limit})
    }
}

pub fn verify_mutation_precondition(expected: &str, current: &str) -> Value {
    if expected == current {
        json!({"allowed": true, "code": Value::Null})
    } else {
        json!({"allowed": false, "code": "STALE_PRECONDITION"})
    }
}

#[derive(Debug, Default)]
pub struct IdempotencyJournal {
    confirmed: HashMap<String, Value>,
}

impl IdempotencyJournal {
    pub fn execute<F>(&mut self, key: &str, effect: F) -> Result<Value, String>
    where
        F: FnOnce() -> Value,
    {
        if key.is_empty() {
            return Err("IDEMPOTENCY_KEY_REQUIRED".into());
        }
        if let Some(result) = self.confirmed.get(key) {
            return Ok(json!({"replayed": true, "result": result}));
        }
        let result = effect();
        self.confirmed.insert(key.to_string(), result.clone());
        Ok(json!({"replayed": false, "result": result}))
    }
}

pub fn enter_recovery(operation: &Value, now: &str) -> Result<Value, String> {
    if operation["status"] != Value::String("ambiguous".into()) {
        return Err("OPERATION_NOT_AMBIGUOUS".into());
    }
    let mut result = operation.clone();
    let object = result.as_object_mut().expect("operation must be an object");
    let attempt = operation["attempt"].as_u64().expect("attempt must be u64");
    object.insert("status".into(), json!("recovering"));
    object.insert("attempt".into(), json!(attempt + 1));
    object.insert("updated_at".into(), json!(now));
    object
        .entry("error")
        .or_insert_with(|| json!({"code": "REMOTE_RESULT_UNKNOWN", "retryable": true}));
    Ok(result)
}

pub fn validate_audit_correlation(events: &[Value]) -> Value {
    let Some(first) = events.first() else {
        return json!({"valid": false, "code": "AUDIT_SEQUENCE_EMPTY"});
    };
    let expected = json!({
        "trace_id": first["trace_id"].clone(),
        "run_id": first["run_id"].clone(),
        "tool_call_id": first["tool_call_id"].clone()
    });
    for event in events {
        if event["trace_id"] != expected["trace_id"]
            || event["run_id"] != expected["run_id"]
            || event["tool_call_id"] != expected["tool_call_id"]
        {
            return json!({
                "valid": false,
                "code": "AUDIT_CORRELATION_MISMATCH",
                "expected": expected,
                "event_id": event["event_id"].clone()
            });
        }
    }
    json!({
        "valid": true,
        "code": Value::Null,
        "trace_id": expected["trace_id"].clone(),
        "run_id": expected["run_id"].clone(),
        "tool_call_id": expected["tool_call_id"].clone()
    })
}
