# Solution 02: Define SLIs and SLOs for a Web Application

## Part A -- Candidate SLIs

### 1. Browsing Products (search / catalog navigation)

| SLI | What | Where | How |
|-----|------|-------|-----|
| Search latency | Time from search request to results rendered | Client-side (Real User Monitoring) and API Gateway | Application metrics via `http_request_duration_seconds` histogram on `/api/search` |
| Search availability | Ratio of search requests returning 2xx vs. total | API Gateway | Prometheus counter `http_requests_total{path="/api/search"}` |
| Search result quality | Percentage of searches returning at least one result | Search Service (Elasticsearch) | Application-level metric or log analysis on zero-result queries |

### 2. Viewing a Product Page

| SLI | What | Where | How |
|-----|------|-------|-----|
| Page load time | Time from navigation start to largest contentful paint | Client-side (RUM via browser Performance API) | Synthetic monitoring (Lighthouse) and RUM beacons |
| Product API latency | Time for `/api/products/{id}` to return | API Gateway | Prometheus histogram on the Product Catalog Service |
| Product API availability | Ratio of successful product detail requests | API Gateway | `http_requests_total{path=~"/api/products/.*",status=~"2.."}` / total |

### 3. Placing an Order

| SLI | What | Where | How |
|-----|------|-------|-----|
| Checkout availability | Ratio of checkout attempts that complete successfully | API Gateway | Counter on `/api/orders` -- exclude 4xx (user input errors) from total |
| Checkout latency | Time from "Place Order" click to confirmation page | Client-side and API Gateway | End-to-end timer; server-side histogram on Order Service |
| Payment success rate | Ratio of payment authorizations that succeed | Payment Service | Counter on payment gateway calls; separate 3rd-party failures from internal errors |

### 4. Checking Order Status

| SLI | What | Where | How |
|-----|------|-------|-----|
| Order status API availability | Ratio of `/api/orders/{id}/status` returning 2xx | API Gateway | Prometheus counter with status labels |
| Order status API latency | Response time for order status lookup | API Gateway | Histogram on Order Service read path |
| Data freshness | Age of the most recent status update shown to the user | Order Service | Timestamp difference between last event processed and current time |

## Part B -- SLO Definitions

### Browsing Products

```
SLI: Ratio of search requests returning 2xx responses
Method: Prometheus counter ratio at API Gateway, 5-minute rolling window
SLO: 99.9% of search requests return successfully over a 30-day window
Rationale: Search is the entry point for revenue. Users who cannot search
cannot buy. However, search has a fallback (category navigation), so 99.9%
rather than 99.99% is appropriate.
```

```
SLI: 99th percentile search latency
Method: Prometheus histogram on /api/search, p99 computed over 5-minute window
SLO: 99% of search requests complete within 500ms over a 30-day window
Rationale: Users abandon searches after ~1 second. A 500ms p99 ensures the
vast majority of users get fast results while allowing headroom for complex
queries hitting Elasticsearch.
```

### Viewing a Product Page

```
SLI: Ratio of product detail API requests returning 2xx
Method: Prometheus counter ratio at API Gateway for /api/products/{id}
SLO: 99.95% of product page loads succeed over a 30-day window
Rationale: Product pages are the primary conversion funnel. A failed page
load directly loses a potential purchase. Higher bar than search because
there is no fallback -- the user clicked a specific product.
```

```
SLI: 99th percentile product page load time (client-side)
Method: Real User Monitoring beacon reporting Largest Contentful Paint
SLO: 95% of page loads have LCP under 2.5 seconds over a 30-day window
Rationale: Google's Core Web Vitals sets 2.5s as "good" LCP. Targeting the
95th percentile (rather than 99th) accounts for slow mobile networks without
forcing over-optimization for edge cases.
```

### Placing an Order

```
SLI: Ratio of checkout requests completing successfully (excluding 4xx)
Method: Prometheus counter on /api/orders at API Gateway; exclude status 4xx
SLO: 99.9% of checkout attempts succeed over a 30-day window
Rationale: Checkout is the highest-value action. Every failed checkout is
direct revenue loss. 99.9% allows ~43 minutes of monthly downtime, which is
achievable while leaving error budget for deployments.
```

```
SLI: 99th percentile end-to-end checkout latency
Method: Client-side timer from submit to confirmation; server-side histogram
SLO: 99% of checkouts complete within 5 seconds over a 30-day window
Rationale: Checkout involves synchronous payment authorization (Stripe/PayPal)
which adds 1-3 seconds of third-party latency. A 5-second p99 accounts for
this external dependency while keeping the user experience acceptable.
```

### Checking Order Status

```
SLI: Ratio of order status lookups returning 2xx
Method: Prometheus counter on /api/orders/{id}/status
SLO: 99.95% of order status requests succeed over a 30-day window
Rationale: Order status is a read-only operation against a well-indexed
database. It should be highly reliable. Users checking order status are
already customers -- keeping them informed reduces support tickets.
```

```
SLI: Freshness of order status data shown to user
Method: Compare timestamp of latest order event to query time, measured in
the Order Service
SLO: 99% of order status queries show data no more than 5 minutes old
Rationale: Users expect near-real-time updates. Asynchronous event processing
introduces some delay. 5 minutes is acceptable for shipping status; users
needing instant updates will refresh.
```

## Part C -- Criticality Classification

| SLO | Priority | Justification |
|-----|----------|---------------|
| Checkout availability (99.9%) | **Critical** | Direct revenue impact. Every failed checkout is a lost sale. Users encountering errors during checkout churn at the highest rate. This is the single most business-critical flow. |
| Checkout latency p99 < 5s | **Critical** | Slow checkouts cause users to abandon carts. Combined with availability, these two SLOs protect the revenue funnel. Payment provider latency is an external risk that must be monitored. |
| Product page availability (99.95%) | **High** | Product pages are in the conversion funnel. Failures here lose potential purchases, but users can browse other products. No external dependency risk beyond the database. |
| Product page LCP < 2.5s (p95) | **High** | Slow pages reduce conversion rates. However, this is a performance SLO, not an availability SLO -- users can still complete purchases on slow pages, they just have a worse experience. |
| Search availability (99.9%) | **High** | Search drives discovery. Without search, users must navigate categories manually, reducing conversion. But a search failure does not block users who know what they want. |
| Search latency p99 < 500ms | **Medium** | Latency affects engagement but not correctness. Users will wait for search results. Elasticsearch is an internal dependency with good reliability. |
| Order status availability (99.95%) | **Medium** | Read-only operation, low complexity. Users can check back later. Failure does not prevent them from using the product they already purchased. |
| Order status freshness < 5 min | **Medium** | Stale data is annoying but not harmful. Users can contact support if they need immediate updates. This SLO depends on async event processing, which has inherent delay. |

## Why This Solution Works

1. **User-centric measurement.** Every SLI answers "what does the user experience?" rather than tracking system internals like CPU or memory.

2. **Percentile-based latency.** Using p99 (or p95 for LCP) avoids the trap of averages that hide outliers. A single 30-second request would be invisible in an average but is captured by p99.

3. **Availability excludes user errors.** The checkout SLI excludes 4xx responses because a user submitting an invalid credit card number is not a system failure.

4. **External dependencies acknowledged.** The checkout latency SLO is set at 5 seconds specifically because Stripe/PayPal add 1-3 seconds. Setting it at 1 second would guarantee failure.

5. **Criticality reflects business impact.** Checkout SLOs are Critical because they directly map to revenue. Search and browse SLOs are High because they affect discovery. Status-check SLOs are Medium because they affect experience but not purchasing ability.

## Common Mistakes

1. **Using average latency instead of percentiles.** An average of 200ms could mask that 1% of users wait 10 seconds. Always use percentiles (p50, p95, p99).

2. **Including 4xx in availability calculations.** If a user sends a malformed request and gets a 400, that is not your system failing. Count only 5xx and network errors as failures.

3. **Setting the same SLO for every component.** A notification service does not need the same SLO as a checkout service. SLOs should reflect user impact, which varies by flow.

4. **Ignoring external dependencies in SLO targets.** If your payment flow depends on Stripe, your latency SLO must account for Stripe's latency. Setting a 200ms checkout SLO when Stripe alone takes 1-3 seconds guarantees you will always be in violation.

5. **Too many SLIs.** Eight SLIs (two per flow) is sufficient. Defining 20 SLIs creates noise and makes it hard to know which ones matter. Start with the minimum set that covers your critical user journeys.

6. **Defining SLIs you cannot measure.** If you do not have client-side RUM instrumentation, do not define a client-side LCP SLI. Start with what you can measure at the API Gateway and add instrumentation later.
