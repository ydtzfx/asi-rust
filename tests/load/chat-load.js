// k6 load test for ASI chat API
// Usage: k6 run tests/load/chat-load.js
import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
    stages: [
        { duration: '30s', target: 5 },   // ramp up to 5 users
        { duration: '1m', target: 10 },   // ramp up to 10 users
        { duration: '30s', target: 0 },   // ramp down
    ],
    thresholds: {
        http_req_duration: ['p(95)<30000'], // 95% < 30s
        http_req_failed: ['rate<0.1'],      // < 10% errors
    },
};

const BASE_URL = 'http://localhost:3000';

export default function () {
    // Health check
    const health = http.get(`${BASE_URL}/api/health`);
    check(health, { 'health 200': (r) => r.status === 200 });

    // Ready check
    const ready = http.get(`${BASE_URL}/api/ready`);
    check(ready, { 'ready 200': (r) => r.status === 200 || r.status === 503 });

    // Metrics
    const metrics = http.get(`${BASE_URL}/api/metrics`);
    check(metrics, { 'metrics 200': (r) => r.status === 200 });

    // OpenAPI spec
    const openapi = http.get(`${BASE_URL}/api/openapi.json`, {
        headers: { 'X-User-ID': 'load-test' },
    });
    check(openapi, { 'openapi 200': (r) => r.status === 200 });

    sleep(1);
}
