# Cheatsheet: Capacity Planning

## Load Testing Tools

| Tool | Language | Best For |
|------|----------|----------|
| k6 | JavaScript | Modern load testing |
| Locust | Python | Python-based testing |
| Apache Bench | CLI | Quick HTTP benchmarks |
| wrk | CLI | High-performance testing |

## k6 Example
```javascript
import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
  stages: [
    { duration: '2m', target: 100 },  // Ramp up
    { duration: '5m', target: 100 },  // Stay at 100
    { duration: '2m', target: 0 },    // Ramp down
  ],
  thresholds: {
    http_req_duration: ['p(95)<500'],  // 95% under 500ms
    http_req_failed: ['rate<0.01'],    // <1% errors
  },
};

export default function () {
  const res = http.get('http://test.k6.io');
  check(res, { 'status was 200': (r) => r.status === 200 });
  sleep(1);
}
```

## Bottleneck Analysis
```
1. Run load test
2. Identify bottleneck:
   - CPU bound → Scale up/out
   - Memory bound → Optimize/cache
   - I/O bound → Faster storage/async
   - Network bound → CDN/compress
3. Fix bottleneck
4. Repeat
```

## Capacity Formula
```
Required Capacity = Peak Users × Requests/User × Overhead Factor
```
