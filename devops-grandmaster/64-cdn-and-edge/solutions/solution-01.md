# Solution 01: CDN Architecture and Edge Computing

## Part A -- CDN Request Flow

### Complete Request Lifecycle (Tokyo User, Virginia Origin)

**Initial request (cache miss):**
```
User (Tokyo)        DNS Resolver      CDN Edge (Tokyo PoP)    Origin Shield (APAC)    Origin (Virginia)
     |                    |                    |                       |                       |
     |-- resolve -------> |                    |                       |                       |
     |   cdn.example.com  |                    |                       |                       |
     |                    |-- geo-route ------> |                       |                       |
     |                    |   (returns Tokyo    |                       |                       |
     |                    |    PoP IP)         |                       |                       |
     |<-- PoP IP ---------|                    |                       |                       |
     |                    |                    |                       |                       |
     |-- GET /page ------>|                    |                       |                       |
     |                    |                    |                       |                       |
     |                    |    CACHE MISS      |                       |                       |
     |                    |                    |                       |                       |
     |                    |    GET /page ----->|---------------------->|--------------------->|
     |                    |    (shield miss)   |  (shield cache miss)  |                       |
     |                    |                    |                       |                       |
     |                    |    200 + content <-|<----------------------|<---------------------|
     |                    |    (shield caches) |  (shield caches)      |                       |
     |                    |                    |                       |                       |
     |<-- 200 + content --|                    |                       |                       |
     |    (edge caches)   |                    |                       |                       |
```

**Subsequent request (cache hit):**
```
User (Tokyo)        CDN Edge (Tokyo PoP)
     |                    |
     |-- GET /page ------>|
     |                    |
     |    CACHE HIT       |
     |    (served from    |
     |     edge PoP)      |
     |                    |
     |<-- 200 + content --|
     |    (~2ms latency)  |
```

### Why This Works

The key insight is the two-tier caching hierarchy. The edge PoP serves most requests
from its local cache (sub-5ms). When the edge misses, the origin shield acts as a
request collapse point: if 100 edge PoPs miss simultaneously, the shield only forwards
one request to the origin. This reduces origin load by orders of magnitude.

## Part B -- Component Roles

| Component | Role | Failure Impact |
|-----------|------|----------------|
| Edge PoP (Cache Server) | Caches and serves content closest to users. Terminates TLS, compresses responses, and executes edge functions. Each PoP typically has hundreds of servers. | Users in that region are routed to the next-closest PoP. Latency increases but availability is maintained. CDN DNS detects PoP health and re-routes automatically. |
| Origin Shield | Centralized caching layer between edge PoPs and the origin. Collapses concurrent cache misses from many edges into a single origin request. Reduces origin load during cache warming or thundering herd events. | Edge PoPs make direct requests to the origin. Origin load increases significantly during cache misses, potentially causing latency spikes or origin overload. |
| Origin Server | The authoritative source of content. Runs application logic, queries databases, and generates dynamic responses. | No new content can be served. CDN continues serving cached content until TTLs expire. After TTLs expire, the CDN returns 5xx errors or stale content (if configured). |
| Global DNS / Anycast | Routes users to the nearest healthy PoP. Anycast announces the same IP from all PoPs; BGP naturally routes to the nearest. DNS-based routing resolves to different IPs per geography. | Users cannot reach any PoP. This is a catastrophic failure. Mitigate with multiple DNS providers, Anycast redundancy, and long TTLs on critical DNS records. |
| Edge Compute Runtime | Executes lightweight logic at the edge: A/B testing, header manipulation, authentication checks, geolocation-based routing, URL rewrites. | Requests that require edge logic fail or fall back to origin-side processing. Latency increases as the origin handles the logic. Non-critical functions (logging, A/B assignment) may be skipped. |

## Part C -- Edge Computing Models

| Model | Execution Location | Latency | Use Cases | Cold Start | Constraints |
|-------|--------------------|---------|-----------|------------|-------------|
| Edge Functions (e.g., CloudFront Functions) | Every edge PoP (400+ locations) | Sub-millisecond | URL rewrites, header manipulation, simple redirects, A/B test assignment, cache key normalization | None (always warm) | Max 10ms execution, 10KB code size, no network calls, read-only request/response access |
| Edge Workers (e.g., Lambda@Edge, Cloudflare Workers) | Regional edge caches (~13 locations for Lambda@Edge, 300+ for Cloudflare) | 1-50ms | Authentication, token validation, dynamic routing, API gateway logic, lightweight API responses | Cold start: 50-200ms for Lambda@Edge; near-zero for Cloudflare Workers | Max 5s execution (Lambda@Edge), 30s (Cloudflare), limited memory, can make network calls but adds latency |
| Regional Compute (e.g., Lambda in a region) | Specific AWS region (e.g., us-east-1) | 10-100ms (depends on user distance) | Full application logic, database queries, complex processing, ML inference | Cold start: 100ms-2s depending on runtime and package size | Full compute resources, database access, no CDN integration unless explicitly configured |

## Part D -- Scenario Matching

### Scenario 1: News website, changing articles, millions of reads/sec globally

**Best strategy: Edge caching with short TTLs and event-driven invalidation.**

Set a 5-minute TTL on article pages and use the CDN's purge API to invalidate specific
article paths when they are updated. The edge cache absorbs millions of reads while
the short TTL ensures eventual freshness. Event-driven purging (triggered by the CMS)
provides near-instant updates for breaking news without requiring all users to wait
for TTL expiry. The origin shield prevents thundering herd during popular breaking
news events.

### Scenario 2: E-commerce with personalized recommendations at edge

**Best strategy: Edge compute (Edge Workers) for personalization + cached catalog.**

Cache the product catalog aggressively at the edge (TTL of 1 hour). Use an Edge Worker
to intercept requests, check the user's authentication cookie, and fetch personalized
recommendations from a low-latency API (e.g., DynamoDB Global Tables or a
region-local recommendation service). The Edge Worker assembles the final response by
combining the cached catalog with the personalized data. This keeps the catalog cached
while delivering personalization at the edge.

### Scenario 3: Video streaming across continents

**Best strategy: CDN with range request support and multi-bitrate manifest serving.**

Use a CDN configured for large file delivery with:
- Chunked transfer encoding and range request support (so players can seek).
- Origin storage in S3 with Transfer Acceleration for ingest.
- Manifest files (HLS `.m3u8` or DASH `.mpd`) cached with short TTLs (30s) since
  they change when new segments are added.
- Video segments cached with long TTLs (7 days) since they are immutable once created.
- Multiple edge PoPs per continent ensure that popular content is served locally.

### Scenario 4: SaaS API, unique per-user responses, cannot be cached

**Best strategy: CDN for TLS termination, DDoS protection, and connection optimization.**

Even though responses are uncacheable, the CDN adds value by:
- Terminating TLS at the edge (reducing origin CPU for TLS handshakes).
- Absorbing volumetric DDoS attacks before they reach the origin.
- Maintaining persistent HTTP/2 connections to the origin, reducing connection overhead.
- Providing geographic routing to the nearest origin region.
- Applying WAF rules at the edge to block malicious requests.

Configure the behavior with `Cache-Control: no-store` and forward all headers to
the origin.

## Common Mistakes to Avoid

1. **Confusing edge functions with edge workers**: Edge functions (CloudFront Functions)
   cannot make network calls. If you need to call an API or database, use edge workers
   (Lambda@Edge). Choosing the wrong model forces a costly migration later.

2. **Ignoring origin shield**: Without an origin shield, 100 edge PoPs missing
   simultaneously means 100 requests to your origin. For expensive API calls or
   database queries, this can cause an outage.

3. **Over-caching at the edge for dynamic content**: Caching user-specific responses
   at the edge without proper `Vary` headers or cache key design can serve one user's
   data to another user. Always verify cache key isolation.

4. **Assuming CDN replaces application-level caching**: CDNs cache at the HTTP layer.
   Database query results, session data, and computed values still benefit from
   application-level caches (Redis, Memcached).

5. **Not planning for cache warming**: When you deploy a new version or purge the
   cache, all edge PoPs start cold. Plan for origin load spikes by deploying during
   low-traffic periods or using a cache warming strategy.

## Key Takeaway

A CDN is not just a caching layer -- it is a distributed compute platform that brings
your content, logic, and security policies closer to users. The architecture choices
(edge functions vs edge workers, origin shield configuration, cache key design) directly
impact latency, origin load, cost, and user experience. Design your CDN strategy
holistically: caching, compute, security, and observability are interconnected.
