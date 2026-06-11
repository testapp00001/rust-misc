# Exercise 03: Cache Invalidation Strategies (Independent)

## Objective

Implement and compare multiple cache invalidation strategies for a CDN-cached web
application, evaluating the trade-offs between correctness, cost, and operational
complexity.

## Background

You run a news website where articles are published and updated frequently. The site
is fronted by a CDN with a 1-hour default TTL. When an editor publishes or updates an
article, users must see the new content within seconds, not minutes. You need to design
a cache invalidation strategy that balances freshness with cost and performance.

## Instructions

### Part A -- TTL-Based Invalidation

Design a TTL strategy for the following content types:

| Content Type | Update Frequency | Recommended TTL | Justification |
|-------------|------------------|-----------------|---------------|
| Breaking news articles | Every few minutes | | |
| Opinion columns | Once per day | | |
| Static assets (JS/CSS/images) | On deploy | | |
| API responses (user-specific) | Per request | | |
| Homepage | Every 5 minutes | | |

### Part B -- Purge-Based Invalidation

Write the implementation for an invalidation system that:

1. Exposes an internal API endpoint `POST /admin/invalidate` that accepts a JSON body:
   ```json
   {
     "paths": ["/articles/breaking-news-123", "/"],
     "type": "exact|prefix|wildcard"
   }
   ```
2. Calls the CloudFront `CreateInvalidation` API with the provided paths.
3. Returns the invalidation ID and status.
4. Rate-limits invalidation requests to avoid exceeding the CloudFront free tier (1000
   invalidation paths per month).

### Part C -- Versioned URL Strategy

Implement a versioned URL approach as an alternative to purging:

1. When an article is updated, increment a version number stored in the database.
2. Embed the version in the asset URL: `/articles/breaking-news-123?v=3`.
3. Update the HTML references to use the new versioned URL.
4. Explain why this eliminates the need for cache purging.

Write the code that generates versioned URLs in a template engine (any language or
pseudocode).

### Part D -- Stale-While-Revalidate

Implement the `stale-while-revalidate` pattern:

1. The CDN serves a cached response immediately, even if it is stale.
2. In the background, the CDN revalidates with the origin.
3. The next request gets the fresh response.

Write the `Cache-Control` header configuration and the origin-side logic that handles
revalidation requests (conditional GET with `If-None-Match` / `If-Modified-Since`).

### Part E -- Event-Driven Invalidation

Design an event-driven invalidation pipeline:

```
Editor updates article
    -> Application writes to database
    -> Publishes event to message queue (SNS/SQS/EventBridge)
    -> Invalidator service consumes event
    -> Calls CDN purge API or bumps version
```

Write the event schema and the invalidator service logic in the language of your choice.

## Success Criteria

- [ ] TTL table has appropriate values with clear justifications.
- [ ] Purge API implementation includes rate limiting and error handling.
- [ ] Versioned URL strategy eliminates the thundering herd problem on invalidation.
- [ ] Stale-while-revalidate implementation handles conditional requests correctly.
- [ ] Event-driven pipeline includes dead-letter queue handling for failed invalidations.

## Hints

<details>
<summary>Hint 1 -- CloudFront invalidation costs</summary>
The first 1,000 invalidation paths per month are free. After that, each path costs
$0.005. Using wildcard invalidations (e.g., `/articles/*`) counts as one path but
invalidates all matching objects. Prefer wildcards when invalidating many related
objects.
</details>

<details>
<summary>Hint 2 -- Versioned URLs are cache-friendly</summary>
When you use versioned URLs (`/app.js?v=abc123`), you can set extremely long TTLs
(e.g., 1 year) because the URL changes when the content changes. This is the pattern
used by Webpack, Vite, and other build tools that generate hashed filenames.
</details>

<details>
<summary>Hint 3 -- Stale-while-revalidate header format</summary>
```
Cache-Control: public, max-age=60, stale-while-revalidate=300
```
This means: serve from cache for 60 seconds. Between 60 and 360 seconds, serve the
stale response immediately but trigger a background revalidation. After 360 seconds,
do not serve stale -- wait for a fresh response.
</details>

<details>
<summary>Hint 4 -- Event schema design</summary>
The event should include enough context for the invalidator to act without querying
the database: article ID, list of affected paths, invalidation type (purge vs
version bump), and a timestamp for ordering. Use idempotency keys to handle
duplicate events.
</details>
