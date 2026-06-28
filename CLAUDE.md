# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Test

```bash
# Build (debug)
cargo build

# Build (release)
cargo build --release

# Run all 107 tests
cargo test

# Run a single test or test module
cargo test -p asi-server --test api_integration
cargo test -p asi-lib test_nanoid_length
cargo test -p asi-server -- agent::tools::run_command

# Build Tailwind CSS (when styles/input.css changes)
npx @tailwindcss/cli -i styles/input.css -o static/styles.css --minify

# Run the server (reads .env for configuration)
cp .env.example .env   # first time only — edit with your keys
cargo build --release && ./target/release/asi-server
# Server starts on http://localhost:3000
```

## Architecture

ASI-Rust is a Cargo workspace with six crates serving a single binary (`asi-server`). Every crate is a library; `asi-server/src/main.rs` is the only binary entrypoint.

### Crate dependency graph

```
asi-server (binary + routes + agent tools)
  ├── asi-app      (Leptos SSR frontend — currently replaced by static/index.html)
  ├── asi-ai-sdk   (AI provider trait, ToolLoopAgent, streaming)
  ├── asi-auth     (Clerk JWT verification middleware)
  ├── asi-db       (SQLite via sqlx, migrations, sessions)
  └── asi-lib      (Config, flags, safe_path, rate_limit, concurrency, telemetry, prompt_guard, etc.)
```

### Request flow (chat)

```
Browser → POST /api/chat → auth middleware (asi-auth) → chat_handler (chat.rs)
  → build provider (DeepSeek or Ollama based on env)
  → ToolLoopAgent::execute() spawns tokio task
    → run_agent_loop(): provider.chat() → check for tool_calls →
       execute tools → feed results back → loop until done or max_steps
  → AgentEvents streamed as SSE to client
```

### Provider selection (`chat.rs:278-317`, `model_registry.rs:54-70`)

- `DEEPSEEK_API_KEY` set → DeepSeek primary, Ollama fallback
- `DEEPSEEK_API_KEY` not set → Ollama primary, uses `OLLAMA_MODEL` env var
- `OLLAMA_FALLBACK_MODEL` used as backup when `model-fallback` flag enabled
- `FallbackProvider` wrapper (`asi-ai-sdk/src/provider/fallback.rs`) retries with backup on transient errors

### Agent tool calling (`tool_loop.rs`)

The agent loop runs inside a `tokio::spawn` task. Each iteration:
1. Builds `ChatRequest` with conversation + tool definitions
2. Calls `provider.chat()` (non-streaming, `max_tokens=4096`)
3. Sends text content as `TextDelta`, checks for `tool_calls` in the response
4. If tool calls present: executes each via `ToolMap`, injects `Role::Tool` messages, loops
5. If no tool calls: sends `Done` with usage
6. `CancelToken` checks and `tx.send().is_err()` guard against orphaned tasks on client disconnect

### Auth (`asi-auth`)

- Middleware: `require_auth` verifies Clerk JWT from `Authorization` header or `__session` cookie
- Dev mode: when `CLERK_SECRET_KEY` starts with `sk_test_` **and** `ASI_DEV_AUTH_BYPASS=true`, the `X-User-ID` header is accepted as a fallback (two-factor gated)
- JWKS fetched from Clerk API with 1-hour cache, 10s timeout
- Validates JWT `typ=JWT` and `alg=RS256` before processing

### Config & feature flags (`asi-lib/src/config.rs`, `flags.rs`)

- `Config::from_env()` reads from `std::env::var` (no dotenv — set via shell or `export $(grep -v '^#' .env | xargs)`)
- `Config::validate()` logs warnings for production-risky settings (relative `DATABASE_URL`, missing `EVOLVE_SECRET`)
- Feature flags: `flag("name")` checks runtime override > `FEATURE_NAME` env var > default
- Defaults: `prompt-injection-defense=true`, all others `false`

### Security boundaries

- **Path containment**: `resolve_safe_path()` canonicalizes and verifies paths stay within project root; `verify_path_after_write()` closes TOCTOU window
- **Command allowlist**: `SAFE_COMMANDS` restricts to `git`, `npm`, `cargo`, `npx` (npx further restricted to known tools). Shell metacharacters blocked. `node` intentionally excluded.
- **Rate limiting**: `GlobalRateLimitLayer` (60 req/min per endpoint prefix) + chat-specific `SlidingWindowLimiter` (20/min per user)
- **Concurrency**: max 4 simultaneous agent executions via `ConcurrencyLimiter` (saturating release, no panic)
- **Prompt guard**: 16 regex patterns for injection detection
- **Message limits**: max 50 messages, 100KB total content per chat request
- **File read limit**: max 1MB per `readFile` call

### Database (`asi-db`)

- SQLite via `sqlx` 0.8 with WAL mode, foreign keys, busy_timeout=5s
- Pool size: defaults to 10, configurable via `DATABASE_POOL_SIZE`
- Migrations run at startup via `sqlx::migrate!`
- Global pool accessed via `asi_db::get_db()` (panics if called before `init_db`)

### Router layering order (`router.rs`)

Layers are applied bottom-to-top (innermost first):
1. Auth middleware (on protected routes only, not on `/api/health` or `/api/version`)
2. `GlobalRateLimitLayer` (60 req/min per endpoint)
3. `ResponseTimeLayer` (adds `x-response-time` header)
4. `TraceLayer` (HTTP tracing)
5. `CorsLayer` (permissive in dev)

## Key files

| File | Purpose |
|------|---------|
| `crates/asi-server/src/routes/chat.rs` | Chat handler — full pipeline from rate-limit to SSE streaming |
| `crates/asi-ai-sdk/src/agent/tool_loop.rs` | Agent core loop, CancelToken, UTF-8 safe truncation |
| `crates/asi-ai-sdk/src/provider/fallback.rs` | Primary+fallback provider wrapper |
| `crates/asi-ai-sdk/src/provider/ollama.rs` | Ollama OpenAI-compatible API adapter |
| `crates/asi-ai-sdk/src/provider/deepseek.rs` | DeepSeek API adapter |
| `crates/asi-auth/src/middleware.rs` | Auth middleware with dev-mode bypass |
| `crates/asi-auth/src/clerk.rs` | Clerk JWT verification + JWKS cache |
| `crates/asi-lib/src/safe_path.rs` | Path containment with TOCTOU protection |
| `crates/asi-lib/src/prompt_guard.rs` | 16-pattern prompt injection detection |
| `crates/asi-server/src/agent/tools/run_command.rs` | Command execution allowlist + metachar filter |
| `crates/asi-server/src/middleware.rs` | ResponseTimeLayer + GlobalRateLimitLayer |
| `crates/asi-server/src/startup.rs` | Background tasks: session cleanup, auto-evolve timer |

## Frontend

The frontend is `static/index.html` — a standalone chat UI served by `tower-http::ServeDir`. It connects to `/api/chat` via SSE, renders markdown, and displays tool calls/results. The Leptos SSR frontend (`asi-app`) is currently disabled due to a leptos_meta 0.7.8 rendering panic.

## Note on Rust edition

This project uses Rust **edition 2024** (`Cargo.toml` line 6). Ensure `rustc` ≥ 1.96.
