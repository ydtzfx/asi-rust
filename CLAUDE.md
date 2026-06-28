# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Collaboration Protocol

Follow the ["计算文明塑造者" protocol](memory://collaboration-protocol):

1. **Core judgment first** — direct conclusion before explanation
2. **Current optimal solution** — executable plan
3. **System decomposition** — architecture/modules/data flow
4. **Code/architecture/process** — complete deliverables
5. **Risks & boundaries** — explicitly stated
6. **Depositable assets** — reusable rules/templates/norms
7. **Next actions** — specific, actionable

Default priorities: solve current problem > systematize > long-term evolution.
For financial/security/automation tasks, always include risk controls, rollback, audit.
Actively question assumptions and propose better alternatives.

## Build & Test

```bash
# Build (debug)
cargo build

# Build (release)
cargo build --release

# Run all ~170 tests
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

ASI-Rust is a Cargo workspace with **20 crates**, two binaries (`asi-server`, `asi-cli`). See `docs/ULTIMATE_REPORT.md` for the full enterprise architecture.

### Crate dependency graph

```
asi-server (HTTP + routes + chat + enterprise integration)
  ├── asi-cortex      (unified brain: monitor/analyze/predict/decide/optimize)
  ├── asi-defense     (6-layer defense, threat detection, adaptive firewall)
  ├── asi-evolution   (prompt evo, knowledge, model picker, A/B test)
  ├── asi-automation  (watchdog, self-heal, auto-backup, recovery)
  ├── asi-devops      (PR review, auto-fix, merge gate, deploy verify)
  ├── asi-ai-sdk      (AI providers, ToolLoopAgent, AGI coordinator/planner/memory/reasoning/reflector)
  ├── asi-app         (Leptos SSR frontend — currently replaced by static/index.html)
  ├── asi-auth        (Clerk JWT verification middleware)
  ├── asi-db          (SQLite via sqlx, migrations, sessions)
  └── asi-lib         (15 modules: config, flags, cache, circuit_breaker, safe_path, prompt_guard...)

asi-cli               (enterprise CLI: status/deploy/monitor/backup/logs)
asi-loop              (closed-loop autonomous controller)
asi-mesh              (federated cognitive network)
asi-twin              (digital twin sandbox)
asi-business          (autonomous business: billing/capacity/SLA)
asi-events            (event-driven architecture: pub/sub bus, event store)
asi-compliance        (SOC2/ISO 27001 evidence collection + audit)
asi-analytics         (data lake, trend engine, BI insights)
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
2. Calls `provider.chat_stream()` (first iteration, token-by-token) or `provider.chat()` (subsequent tool-call iterations, `max_tokens=4096`)
3. Sends text content as `TextDelta` (streaming for first call), checks for `tool_calls`
4. If tool calls present: executes each via `ToolMap`, injects `Role::Tool` messages, loops
5. If no tool calls: sends `Done` with usage
6. `CancelToken` checks and `tx.send().is_err()` guard against orphaned tasks on client disconnect
7. LLM response cache (5-min TTL) avoids redundant calls for identical queries

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
| `crates/asi-server/src/startup.rs` | Enterprise runtime init (cortex, defense, evolution), watchdog, health loop, session cleanup |
| `crates/asi-server/src/enterprise.rs` | Cross-crate integration — initializes all 15+ subsystems at startup |

## Frontend

The frontend is `static/index.html` — a standalone chat UI served by `tower-http::ServeDir`. It connects to `/api/chat` via SSE, renders markdown, and displays tool calls/results. The Leptos SSR frontend (`asi-app`) is currently disabled due to a leptos_meta 0.7.8 rendering panic.

## Skills

| Command | Purpose |
|---------|---------|
| `/run` | Build and start the server |
| `/test` | Run test suite (all, single crate, single test) |
| `/deploy` | Full CI pipeline + deploy to production |
| `/review` | Comprehensive code review (security, perf, style) |
| `/migrate` | Database migration management + backup |
| `/monitor` | Health checks, metrics, logs, alerts |
| `/release` | Version bump, changelog, tag, push |

## Note on Rust edition

This project uses Rust **edition 2024** (`Cargo.toml` line 6). Ensure `rustc` ≥ 1.96.
