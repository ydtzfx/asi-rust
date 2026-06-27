# ASI Rust

AI-powered multi-agent coding assistant platform (SaaS). Rust rewrite of the original Next.js ASI platform, built with performance, safety, and concurrency in mind.

## Architecture

The project is a Cargo workspace with six crates:

| Crate | Purpose |
|-------|---------|
| `asi-server` | Axum HTTP server — router, middleware stack, main entrypoint, all API routes |
| `asi-lib` | Core utilities — errors, flags, logger, config, safe-path, concurrency, rate-limit, cache, retry, telemetry, token tracking |
| `asi-ai-sdk` | AI provider abstraction — `Provider` trait with DeepSeek / Ollama adapters, `Tool` trait, `ToolLoopAgent`, SSE streaming |
| `asi-auth` | Clerk JWT verification middleware — validates session tokens, extracts user identity |
| `asi-db` | Database layer — SQLite via `rusqlite`, migration runner, schema definitions |
| `asi-app` | Leptos SSR frontend — dashboard chat UI, 5 public pages (landing, about, services, news, contact), Clerk auth bridge |

## Stack

| Concern | Technology |
|---------|-----------|
| Language | Rust (edition 2024) |
| Web framework | Axum 0.9 |
| Frontend | Leptos 0.7 (SSR) |
| Styling | Tailwind CSS 4 |
| Database | SQLite via rusqlite |
| Auth | Clerk JWT |
| AI | DeepSeek API (primary), Ollama (fallback) |
| Async runtime | Tokio |

## Setup

### Prerequisites

- Rust 1.96+ (edition 2024)
- Node.js 20+ (for Tailwind CLI)
- A Clerk account (for auth)

### Quick Start

```bash
# Clone and enter
git clone <repo-url> && cd asi-rust

# Copy environment config
cp .env.example .env
# Edit .env with your Clerk keys and API keys

# Build
cargo build --release

# Build CSS
npm install
npx @tailwindcss/cli -i styles/input.css -o static/styles.css --minify

# Run
./target/release/asi-server
```

The server starts on `http://localhost:3000`.

### Environment Variables

See `.env.example` for all supported variables. Key ones:

| Variable | Required | Default | Purpose |
|----------|----------|---------|---------|
| `CLERK_SECRET_KEY` | Yes | — | Clerk server-side auth |
| `NEXT_PUBLIC_CLERK_PUBLISHABLE_KEY` | Yes | — | Clerk client-side auth |
| `DATABASE_URL` | No | `asi.db` | SQLite database path |
| `DEEPSEEK_API_KEY` | No | — | DeepSeek API key (primary AI provider) |
| `DEEPSEEK_MODEL` | No | `deepseek-chat` | DeepSeek model name |
| `OLLAMA_BASE_URL` | No | `http://localhost:11434/v1` | Ollama endpoint |
| `OLLAMA_MODEL` | No | `gemma4:31b-cloud` | Ollama model |
| `EVOLVE_SECRET` | No | — | Secret for self-evolution API |

Feature flags are controlled by `FEATURE_*` env vars (e.g., `FEATURE_MULTI_AGENT=1`).

## Deploy

### Docker

```bash
docker build -t asi-rust .
docker run -p 3000:3000 --env-file .env asi-rust
```

### Vercel

The project includes a `vercel.json` for Vercel deployment. The build pipeline:

1. Installs Rust via `rustup`
2. Builds the binary with `cargo build --release`
3. Compiles Tailwind CSS
4. Serves static files from the `static/` directory

Note: Vercel supports Rust via Docker-based deployment or custom build steps. The `vercel.json` routes all requests to the static output — configure the server binary as a custom runtime for full functionality.

## API Routes

| Route | Method | Purpose |
|-------|--------|---------|
| `/api/chat` | POST | AI agent chat (SSE streaming) |
| `/api/health` | GET | Health check |
| `/api/ready` | GET | Readiness probe |
| `/api/flags` | GET/POST | Feature flags |
| `/api/sessions` | GET/POST | Session CRUD |
| `/api/metrics` | GET | Prometheus-style metrics |
| `/api/stats` | GET | Aggregate statistics |
| `/api/version` | GET | Version info |
| `/api/model` | GET/POST | Model registry |
| `/api/evolve` | GET | Self-evolution trigger |
| `/api/eval` | POST | Agent evaluation |
| `/api/feedback` | POST | User feedback submission |
| `/api/search` | POST | Codebase search |
| `/api/tools` | GET | Agent tool listing |
| `/api/docs` | GET | Agent documentation |

## Security

- Path containment for all agent file operations via `resolveSafePath()`
- Command execution via `exec_file` with strict allowlist, subcommand/flag filtering, and metacharacter rejection
- Clerk JWT verification on all protected routes
- Prompt injection defense (configurable via feature flag)
- Rate limiting on chat endpoint

## License

MIT
