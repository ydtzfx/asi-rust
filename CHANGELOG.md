# Changelog

## [0.1.0] — 2026-06-28

### Added
- Agent loop with streaming SSE output and tool calling (readFile, writeFile, listDirectory, runCommand)
- Clerk JWT authentication middleware with dev-mode bypass gate
- Ollama and DeepSeek AI provider adapters with automatic fallback
- 16-pattern prompt injection defense
- Global rate limiting (60 req/min) and chat-specific rate limiting (20/min)
- Concurrency limiter with saturating release
- Circuit breaker for provider failures
- CancelToken to stop agent execution on client disconnect
- RFC 7807 Problem Details error format
- OpenAPI 3.0 specification endpoint
- Prometheus metrics with Grafana dashboard
- Caddy TLS termination config
- Docker Compose production stack with monitoring
- GitHub Actions CI/CD (Ubuntu + Windows build/test/lint)
- Database migration tracking, backup/restore scripts
- k6 load testing script
- Fuzz tests for prompt injection detection
- Static chat UI with SSE streaming, tool call visualization

### Security
- Path containment with TOCTOU verification
- Command allowlist (git, cargo, npm, npx with subcommand restrictions)
- Message size limits (50 messages, 100KB)
- File read size limit (1MB)
- JWT validation (typ=JWT, alg=RS256)
- JWKS fetch with timeout
- Prompt injection defense (16 regex patterns)
- Rate limiting on all API endpoints
- Audit logging (flag-gated)
- Dev auth bypass requires two-factor gate
