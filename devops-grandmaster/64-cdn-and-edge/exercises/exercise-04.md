# Exercise 04: Multi-Region Edge Architecture (Challenge)

## Objective

Design a multi-region edge architecture that serves content from the nearest origin
when the CDN cache misses, using geographic routing, origin shields, and latency-based
failover to achieve sub-100ms p95 latency globally.

## Background

Your company operates a global e-commerce platform with users in North America, Europe,
and Asia-Pacific. The current single-origin architecture in `us-east-1` causes 300ms+
latency for APAC users on cache misses. You need to deploy origins in multiple regions
and ensure that edge cache misses are served from the nearest healthy origin.

## Instructions

### Part A -- Multi-Region Origin Design

Design the origin architecture:

```
Regions:
  - Primary: us-east-1 (North America)
  - Secondary: eu-west-1 (Europe)
  - Tertiary: ap-southeast-1 (Asia-Pacific)

Requirements:
  - Each region has its own application servers and database read replica.
  - Writes are routed to the primary region only.
  - Reads can be served from any region.
  - Data replication lag must be accounted for in the caching strategy.
```

Draw an architecture diagram showing:
1. User traffic entering via CDN edge locations.
2. CDN routing to the nearest origin group.
3. Database replication topology.
4. Write-path routing to the primary region.

### Part B -- Origin Shield Configuration

Configure an origin shield per region to reduce origin load:

| Shield Region | Protects Origins In | Purpose |
|---------------|---------------------|---------|
| us-east-1 | North America, South America | |
| eu-west-1 | Europe, Africa | |
| ap-southeast-1 | Asia, Oceania | |

Write the CloudFront configuration (or Terraform) that assigns each origin group to
its corresponding shield region.

### Part C -- Latency-Based Failover

Implement a health-check and failover mechanism:

1. Each origin has a health check endpoint: `GET /health` returns 200 if healthy.
2. If the nearest origin is unhealthy, the CDN should failover to the next closest
   healthy origin.
3. Write the Route 53 health check and failover routing configuration (or equivalent
   CloudFront origin group configuration).

Design the health check logic:

```
/health endpoint should:
  - Verify database connectivity (read replica is reachable and not too far behind)
  - Verify application server is accepting requests
  - Return 503 if any check fails
```

### Part D -- Cache Coherence Across Regions

When a product price is updated in `us-east-1`, users in all regions must see the
updated price within 30 seconds. Design the invalidation strategy:

1. How do you propagate invalidation to all CDN edge locations?
2. How do you handle the database replication lag (a user in APAC might query a
   read replica that has not yet received the update)?
3. What `Cache-Control` directives balance freshness with origin load?

### Part E -- Edge Function for Geographic Routing

Write an edge function (CloudFront Functions or Cloudflare Workers) that:

1. Reads the `CloudFront-Viewer-Country` header.
2. Adds a `X-Origin-Region` header to the origin request, set to the preferred
   region based on the viewer's country.
3. Logs the routing decision for observability.
4. Handles the case where the preferred region is not configured (falls back to
   primary).

## Success Criteria

- [ ] Architecture diagram shows all three regions with correct data flow.
- [ ] Origin shield configuration reduces origin requests by collapsing concurrent
      cache misses.
- [ ] Failover correctly redirects traffic when a region is unhealthy (testable by
      taking down one origin).
- [ ] Cache invalidation propagates to all regions within 30 seconds.
- [ ] Edge function correctly routes based on viewer geography.

## Hints

<details>
<summary>Hint 1 -- Origin groups in CloudFront</summary>
CloudFront supports origin groups with primary and secondary origins. When the
primary origin returns specific failure codes (5xx, 4xx), CloudFront automatically
retries with the secondary origin. This is simpler than Route 53 failover for
CDN-specific scenarios.

```hcl
origin_group {
  origin_id = "multi-region-group"
  failover_criteria {
    status_codes = [500, 502, 503, 504]
  }
  member {
    origin_id = "us-east-1"
  }
  member {
    origin_id = "eu-west-1"
  }
}
```
</details>

<details>
<summary>Hint 2 -- Replication lag and cache TTL</summary>
If your read replicas have up to 5 seconds of replication lag, set your CDN cache TTL
to at least 10 seconds for dynamic content. This ensures that by the time the cache
expires and a new request reaches the origin, the replica has caught up. For price
updates, use event-driven invalidation with a short delay to account for lag.
</details>

<details>
<summary>Hint 3 -- Edge function header parsing</summary>
```javascript
function handler(event) {
    var request = event.request;
    var country = request.headers['cloudfront-viewer-country']
        ? request.headers['cloudfront-viewer-country'].value
        : 'US';

    var regionMap = {
        'US': 'us-east-1', 'CA': 'us-east-1', 'MX': 'us-east-1',
        'GB': 'eu-west-1', 'DE': 'eu-west-1', 'FR': 'eu-west-1',
        'JP': 'ap-southeast-1', 'AU': 'ap-southeast-1', 'SG': 'ap-southeast-1'
    };

    request.headers['x-origin-region'] = { value: regionMap[country] || 'us-east-1' };
    return request;
}
```
</details>

<details>
<summary>Hint 4 -- Global invalidation strategy</summary>
CloudFront invalidations apply globally to all edge locations by default -- you do
not need to target specific regions. The challenge is not CDN-level invalidation but
database-level consistency: ensure your invalidation event includes a timestamp, and
the origin handler waits for the read replica to catch up before serving a fresh
response (or uses a "read-your-writes" pattern by querying the primary for recently
updated data).
</details>
