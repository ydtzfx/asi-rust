use axum::{Json, Router, http::StatusCode, routing::post};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;

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
// Rate limiter for the evolve endpoint (3 attempts per 5 minutes).
// ---------------------------------------------------------------------------

static EVOLVE_RATE_LIMITER: std::sync::LazyLock<asi_lib::rate_limit::SlidingWindowLimiter> =
    std::sync::LazyLock::new(asi_lib::rate_limit::SlidingWindowLimiter::new);

const EVOLVE_RATE_MAX: u32 = 3;
const EVOLVE_WINDOW_MS: u64 = 300_000; // 5 minutes
const EVOLVE_COMMAND_TIMEOUT: Duration = Duration::from_secs(300); // 5 min per command

// ---------------------------------------------------------------------------
// Handler
// ---------------------------------------------------------------------------

/// POST /api/evolve — triggers a self-evolution cycle.
///
/// Requires a `secret` in the JSON body that matches the `EVOLVE_SECRET`
/// environment variable.  Rate-limited to 3 attempts per 5 minutes.
/// On success, runs `cargo clippy` (lint) and `cargo test` in sequence
/// with a 5-minute timeout per command.
async fn evolve(
    Json(body): Json<EvolveRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // ---- Rate limit (brute-force protection) ----
    match EVOLVE_RATE_LIMITER.check("evolve", EVOLVE_RATE_MAX, EVOLVE_WINDOW_MS) {
        asi_lib::rate_limit::RateLimitResult::RetryAfter(ms) => {
            return Err((
                StatusCode::TOO_MANY_REQUESTS,
                Json(serde_json::json!({
                    "error": format!("Too many evolve attempts. Retry after {}s", ms / 1000)
                })),
            ));
        }
        asi_lib::rate_limit::RateLimitResult::Denied => {
            return Err((
                StatusCode::TOO_MANY_REQUESTS,
                Json(serde_json::json!({"error": "Too many evolve attempts"})),
            ));
        }
        asi_lib::rate_limit::RateLimitResult::Ok => {}
    }

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
    let lint_output = run_command_async(
        "cargo",
        &["clippy", "--all-targets", "--", "-D", "warnings"],
    )
    .await;

    // ---- Run tests (cargo test --workspace) ----
    let test_output = run_command_async("cargo", &["test", "--workspace"]).await;

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
    if let Err(e) = asi_db::queries::audit::insert_audit_log(
        asi_db::get_db(),
        "system",
        "evolve_cycle",
        &format!("Evolution cycle completed: {}", overall),
        Some(&detail.to_string()),
        None,
        None,
    )
    .await
    {
        asi_lib::logger::warn(
            "Failed to write evolve audit log",
            &[("error", &e.to_string())],
        );
    }

    Ok(Json(serde_json::json!(EvolveResponse {
        status: overall.to_string(),
        lint: lint_output,
        test: test_output,
    })))
}

/// Run a command asynchronously with a timeout, capturing stdout/stderr.
async fn run_command_async(program: &str, args: &[&str]) -> CommandOutput {
    let result = tokio::time::timeout(
        EVOLVE_COMMAND_TIMEOUT,
        tokio::process::Command::new(program).args(args).output(),
    )
    .await;

    match result {
        Ok(Ok(output)) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            CommandOutput {
                success: output.status.success(),
                stdout,
                stderr,
            }
        }
        Ok(Err(e)) => CommandOutput {
            success: false,
            stdout: String::new(),
            stderr: format!("Failed to execute {}: {}", program, e),
        },
        Err(_) => CommandOutput {
            success: false,
            stdout: String::new(),
            stderr: format!("{} timed out after {}s", program, EVOLVE_COMMAND_TIMEOUT.as_secs()),
        },
    }
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn routes() -> Router {
    Router::new().route("/evolve", post(evolve))
}
