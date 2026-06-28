---
name: review
description: Comprehensive code review — security, performance, correctness, style
disable-model-invocation: true
---

# Code Review

Run a comprehensive review using multiple specialized agents.

## Review Dimensions
1. **Security**: Check for injection, auth bypass, path traversal, secrets
2. **Performance**: Clone costs, allocation patterns, async blocking
3. **Correctness**: Logic errors, edge cases, error handling gaps
4. **Style**: Idiomatic Rust, consistency with project conventions

## Commands
```bash
# Get the diff to review
git diff origin/master..HEAD

# Run clippy with strict settings
cargo clippy --all-targets -- -D warnings -W clippy::pedantic

# Check for unused dependencies
cargo udeps 2>/dev/null || echo "install: cargo install cargo-udeps"

# Run all tests
cargo test --workspace
```

## Focus Areas
- `crates/asi-ai-sdk/src/agent/tool_loop.rs` — agent correctness, cancel safety
- `crates/asi-server/src/routes/chat.rs` — auth, rate limiting, error handling
- `crates/asi-auth/src/middleware.rs` — JWT validation, dev-mode gate
- `crates/asi-lib/src/safe_path.rs` — path containment, TOCTOU
- `crates/asi-lib/src/prompt_guard.rs` — regex coverage, false positives
