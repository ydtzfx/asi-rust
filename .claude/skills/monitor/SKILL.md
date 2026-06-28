---
name: monitor
description: Check server health, metrics, logs, and alert status
disable-model-invocation: true
---

# Server Monitoring

Quick health check and metrics inspection.

## Health endpoints
```bash
curl -s http://localhost:3000/api/health    # liveness
curl -s http://localhost:3000/api/ready     # DB + AI provider
curl -s http://localhost:3000/api/metrics   # Prometheus format
curl -s http://localhost:3000/api/stats -H 'X-User-ID: admin'  # aggregate
curl -s http://localhost:3000/health.html   # browser dashboard
```

## Key metrics to watch
- `asi_db_pool_active` — should be < pool size
- `asi_api_calls_total` — request rate
- `/api/ready` ai_provider field — if false, AI is down

## Alert rules (Prometheus)
- ASIServerDown: >1m unreachable
- ASIDatabaseDegraded: >2m health failing
- ASIHighErrorRate: >5% errors over 5m
- ASIRateLimitTriggered: >10 rate-limits in 5m

## Logs
```bash
# Check server logs
tail -f server.log
# Filter errors
grep ERROR server.log | tail -20
```
