use axum::{
    Json, Router,
    http::StatusCode,
    routing::post,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::process::Command;

// ---------------------------------------------------------------------------
// Request / response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct EvolveRequest {
    /// The EVOLVE_SECRET must match the server's environment variable.
    secret: String,
}

#[derive(Debug, Serialize)]
struct EvolveResponse {
    status: String,
    lint: CommandOutput,
    test: CommandOutput,
}

#[derive(Debug, Serialize)]
struct CommandOutput {
    success: bool,
    stdout: String,
    stderr: String,
}

// ---------------------------------------------------------------------------
// Handler
// ---------------------------------------------------------------------------

/// POST /api/evolve — triggers a self-evolution cycle.
///
/// Requires a `secret` in the JSON body that matches the `EVOLVE_SECRET`
/// environment variable.  On success, runs `cargo clippy` (lint) and
/// `cargo test` in sequence and records the result.
async fn evolve(Json(body): Json<EvolveRequest>) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // ---- Authorisation: EVOLVE_SECRET ----
    let expected = std::env::var("EVOLVE_SECRET").unwrap_or_default();
    if expected.is_empty() {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "EVOLVE_SECRET is not configured on the server"})),
        ));
    }
    if body.secret != expected {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Invalid secret"})),
        ));
    }

    // ---- Run lint (cargo clippy --all-targets -- -D warnings) ----
    let lint_output = run_command("cargo", &["clippy", "--all-targets", "--", "-D", "warnings"]);

    // ---- Run tests (cargo test --workspace) ----
    let test_output = run_command("cargo", &["test", "--workspace"]);

    // ---- Build response ----
    let overall = if lint_output.success && test_output.success {
        "success"
    } else {
        "partial_failure"
    };

    // ---- Log to audit log ----
    let detail = serde_json::json!({
        "lint_errors": lint_output.stdout.lines().count(),
        "tests_passed": test_output.stdout.lines().filter(|l| l.contains("ok")).count(),
        "tests_failed": test_output.stdout.lines().filter(|l| l.contains("FAILED")).count(),
    });
    let _ = asi_db::queries::audit::insert_audit_log(
        asi_db::get_db(),
        "system",
        "evolve_cycle",
        &format!("Evolution cycle completed: {}", overall),
        Some(&detail.to_string()),
        None,
        None,
    )
    .await;

    Ok(Json(serde_json::json!(EvolveResponse {
        status: overall.to_string(),
        lint: lint_output,
        test: test_output,
    })))
}

/// Run a command synchronously and capture stdout/stderr.
fn run_command(program: &str, args: &[&str]) -> CommandOutput {
    match Command::new(program).args(args).output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            CommandOutput {
                success: output.status.success(),
                stdout,
                stderr,
            }
        }
        Err(e) => CommandOutput {
            success: false,
            stdout: String::new(),
            stderr: format!("Failed to execute {}: {}", program, e),
        },
    }
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn routes() -> Router {
    Router::new().route("/evolve", post(evolve))
}
