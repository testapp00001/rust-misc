# Exercise 03: Implementing Backpressure and Rate Limiting

**Type:** Independent
**Time:** 30-45 minutes
**Difficulty:** Medium

## Objective

Implement backpressure mechanisms and rate limiting to protect your backend services from overload during traffic surges.

## Scenario

Your API gateway receives requests from three client types:

| Client | Normal RPS | Surge RPS | Priority |
|--------|-----------|-----------|----------|
| Mobile app | 2,000 | 20,000 | High (paying customers) |
| Partner API | 500 | 5,000 | Medium (contractual SLA: 5,000 RPS) |
| Web scraper | 10,000 | 50,000 | Low (no SLA, often abusive) |

Your backend can handle 15,000 RPS total. During a surge, you need to:
1. Protect the backend from overload
2. Prioritize mobile app traffic
3. Enforce partner SLA limits
4. Block or throttle scrapers

## Tasks

### Part A: Design the Rate Limiting Strategy

Design a rate limiting strategy that handles the surge scenario above. For each client type, specify:
- Rate limit (requests per second)
- Burst allowance
- Response when limit is exceeded (status code and body)
- Where the rate limit is enforced (edge, gateway, or application)

Write the strategy as a configuration that could be implemented in Nginx, Envoy, or an API gateway.

<details>
<summary>Hint 1</summary>

Use a tiered approach: global rate limit at the edge (protects backend), per-client rate limits at the gateway (enforces fairness), and per-endpoint limits at the application (protects specific resources). Use token bucket or sliding window algorithms.

</details>

### Part B: Implement Application-Level Backpressure

Write a Python or Go function that implements backpressure for a request handler. The function should:

1. Check current system load (queue depth, CPU, or connection count)
2. Accept requests when load is normal
3. Shed low-priority requests when load is elevated
4. Shed all non-critical requests when load is critical
5. Return appropriate HTTP status codes (429 for rate-limited, 503 for overloaded)

<details>
<summary>Hint 2</summary>

Define load levels (NORMAL, ELEVATED, HIGH, CRITICAL) based on queue depth or CPU. Map client priorities to load levels -- high-priority clients are accepted at higher load levels. Use 429 with `Retry-After` header for rate-limited requests, 503 for overloaded.

</details>

### Part C: Design Circuit Breaker Integration

Your system calls three downstream services: payment processor, inventory service, and notification service. During a surge, the payment processor becomes slow (5-second response times). Design a circuit breaker configuration that:

1. Detects the payment processor failure
2. Opens the circuit to prevent cascade failure
3. Returns a fast failure to clients instead of waiting 5 seconds
4. Tests recovery with half-open state
5. Closes the circuit when the payment processor recovers

Write the configuration and explain the state transitions.

<details>
<summary>Hint 3</summary>

A circuit breaker has three states: CLOSED (normal), OPEN (failing fast), HALF_OPEN (testing recovery). Configure: failure threshold (5 failures in 30 seconds), open timeout (30 seconds), half-open test count (3 requests). When OPEN, return a cached or default response immediately.

</details>

## Success Criteria

- [ ] Rate limiting strategy correctly prioritizes mobile > partner > scraper traffic
- [ ] Backpressure implementation has clear load-level thresholds and client-priority mapping
- [ ] Circuit breaker configuration prevents cascade failure from slow downstream services
- [ ] All responses use appropriate HTTP status codes (429, 503) with meaningful bodies
- [ ] The system degrades gracefully rather than crashing under extreme load

## What You Should Understand After This Exercise

Backpressure and rate limiting are not just about saying "no" to requests -- they are about making intelligent decisions under load. Prioritize high-value traffic, enforce SLAs, shed low-priority requests, and protect downstream services with circuit breakers. A system that degrades gracefully is always better than one that crashes catastrophically.
