# Exercise 01: CDN Architecture and Edge Computing (Conceptual)

## Objective

Understand the fundamental architecture of Content Delivery Networks -- how points of
presence (PoPs), request routing, caching layers, and edge computing work together to
reduce latency and improve availability.

## Background

A CDN is a globally distributed network of servers that caches content close to end
users. Instead of every request traveling to a single origin server, the CDN serves
cached responses from the nearest edge location. Modern CDNs also support edge
computing, where application logic runs at the edge rather than at the origin.

## Instructions

### Part A -- CDN Request Flow

Draw a sequence diagram (ASCII or tool of your choice) showing the complete lifecycle
of a request from a user in Tokyo to a website whose origin is in Virginia. Include:

1. The initial DNS resolution step (how the CDN's DNS routes the user to the nearest
   PoP).
2. The edge server receiving the request and checking its local cache.
3. The cache-miss path: edge server -> origin shield -> origin server.
4. The cache-hit path on a subsequent request.

### Part B -- Component Roles

For each component below, describe its role in 2-3 sentences and explain what happens
when it fails.

| Component | Role | Failure Impact |
|-----------|------|----------------|
| Edge PoP (Cache Server) | | |
| Origin Shield | | |
| Origin Server | | |
| Global DNS / Anycast | | |
| Edge Compute Runtime | | |

### Part C -- Edge Computing Models

Compare the three edge computing models and fill in the table:

| Model | Execution Location | Latency | Use Cases | Cold Start | Constraints |
|-------|--------------------|---------|-----------|------------|-------------|
| Edge Functions (e.g., CloudFront Functions) | | | | | |
| Edge Workers (e.g., Lambda@Edge, Cloudflare Workers) | | | | | |
| Regional Compute (e.g., Lambda in a region) | | | | | |

### Part D -- Scenario Matching

For each scenario, select the best CDN strategy and justify your choice in 2-3
sentences.

1. A news website with articles that change every few minutes but receive millions of
   reads per second globally.
2. An e-commerce site that needs to serve personalized product recommendations at the
   edge while keeping the catalog cached.
3. A video streaming platform that must deliver large files with minimal buffering
   across continents.
4. A SaaS API where each response is unique per user and cannot be cached.

## Success Criteria

- [ ] Sequence diagram correctly shows DNS-based routing, cache-hit, and cache-miss
      paths.
- [ ] Component table accurately describes roles and failure impacts.
- [ ] Edge computing model comparison reflects real constraints of each approach.
- [ ] Scenario justifications reference specific CDN features.

## Hints

<details>
<summary>Hint 1 -- DNS-based routing</summary>
CDN providers use either Anycast (same IP announced from all PoPs) or DNS-based
routing (return different IPs based on the requester's geographic location). When a
user resolves the CDN hostname, the authoritative DNS server returns the IP of the
closest healthy PoP.
</details>

<details>
<summary>Hint 2 -- Origin Shield purpose</summary>
An origin shield is an additional caching layer between edge PoPs and the origin. Its
purpose is to collapse concurrent cache misses from many edge servers into a single
request to the origin. This is sometimes called "request collapsing" or "thundering
herd protection."
</details>

<details>
<summary>Hint 3 -- Edge Functions vs Edge Workers</summary>
Edge Functions (like CloudFront Functions) run at every PoP, have strict execution
limits (sub-millisecond), and can only perform simple transformations. Edge Workers
(like Lambda@Edge) run at regional edge caches, have higher compute limits (up to
seconds), and can make external network calls. The tradeoff is latency (closer to
user) vs capability (more compute power).
</details>

<details>
<summary>Hint 4 -- Uncacheable API responses</summary>
Even when responses cannot be cached, a CDN can still add value through TLS
termination at the edge (reducing origin CPU), DDoS protection, request routing, and
connection reuse (HTTP/2 multiplexing to origin).
</details>
