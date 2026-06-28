---
name: deploy
description: Run full CI pipeline locally, then deploy to production
disable-model-invocation: true
---

# Deploy to Production

Run the complete deployment pipeline.

## Steps
1. Format check: `cargo fmt --check`
2. Lint: `cargo clippy --all-targets -- -D warnings`
3. Test: `cargo test --workspace`
4. Build release: `cargo build --release`
5. If Vercel: `npx vercel deploy --prod`
6. If Docker: `docker compose -f docker-compose.prod.yml up -d --build`

## Pre-flight checks
- `.env` has production values (not test keys)
- `GRAFANA_PASSWORD` is set if using monitoring stack
- Git working tree is clean
