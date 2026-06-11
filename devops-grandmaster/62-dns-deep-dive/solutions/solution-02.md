# Solution 02: DNS Resolution Flow

## Part A: Trace the Resolution

```
Step 1: Client (192.168.1.100) asks its stub resolver to resolve "app.example.com"
        → Stub resolver forwards query to recursive resolver at 8.8.8.8

Step 2: Recursive resolver (8.8.8.8) checks its cache
        → Cache miss (first query), begins iterative resolution

Step 3: Recursive resolver queries a root server (e.g., a.root-servers.net)
        → Root server responds: "I don't know app.example.com, but here are
           the NS records for .com: a.gtld-servers.net, b.gtld-servers.net"

Step 4: Recursive resolver queries a .com TLD server (e.g., a.gtld-servers.net)
        → TLD server responds: "I don't know app.example.com, but here are
           the NS records for example.com: ns1.example.com (198.51.100.1),
           ns2.example.com (198.51.100.2)"

Step 5: Recursive resolver queries the authoritative server (ns1.example.com)
        → ns1.example.com responds: "app.example.com. 300 IN A 203.0.113.50"

Step 6: Recursive resolver returns the A record to the client's stub resolver
        → Client receives IP address 203.0.113.50

Step 7: Recursive resolver caches the answer with TTL=300
        → Next query for app.example.com will be served from cache for 300s
```

### Why This Flow Matters

The recursive resolver does all the heavy lifting. The client only makes
one query. The resolver makes up to three (root, TLD, authoritative) on
the client's behalf. This delegation model scales because each level only
needs to know about the next level down.

## Part B: Use dig to Trace Resolution

```
Block 1: Root hints
─────────────────────
.                       518400  IN  NS  a.root-servers.net.
.                       518400  IN  NS  b.root-servers.net.
;; Received 239 bytes from 8.8.8.8#53(8.8.8.8) in 12 ms
```
The resolver starts by loading root server hints. These NS records point
to the 13 root server groups. The 518400-second TTL means root NS records
are cached for 6 days.

```
Block 2: Root server referral
──────────────────────────────
com.                    172800  IN  NS  a.gtld-servers.net.
com.                    172800  IN  NS  b.gtld-servers.net.
;; Received 897 bytes from 198.41.0.4#53(a.root-servers.net) in 24 ms
```
The root server does not know `app.example.com` but knows who handles
`.com`. It refers the resolver to the .com TLD servers (managed by
Verisign). The response came from `198.41.0.4` which is `a.root-servers.net`.

```
Block 3: TLD server referral
─────────────────────────────
example.com.            172800  IN  NS  ns1.example.com.
example.com.            172800  IN  NS  ns2.example.com.
;; Received 215 bytes from 192.5.6.30#53(a.gtld-servers.net) in 30 ms
```
The .com TLD server knows the authoritative nameservers for `example.com`
but does not have the actual A record. It refers the resolver to
`ns1.example.com` and `ns2.example.com`.

```
Block 4: Authoritative answer
──────────────────────────────
app.example.com.        300     IN  A   203.0.113.50
;; Received 56 bytes from 198.51.100.1#53(ns1.example.com) in 8 ms
```
The authoritative server `ns1.example.com` (at 198.51.100.1) returns the
final answer: `app.example.com` resolves to `203.0.113.50` with a TTL of
300 seconds.

### Total Resolution Time

12ms + 24ms + 30ms + 8ms = 74ms for the full iterative chain. Subsequent
queries will be served from the recursive resolver's cache in ~0ms.

## Part C: Recursive vs Iterative Queries

| Scenario | Query Type | Explanation |
|----------|------------|-------------|
| Client asks its DNS server to resolve `example.com` | **Recursive** | Client expects a final answer, not a referral |
| Recursive resolver asks root server for `.com` NS records | **Iterative** | Resolver asks for the best referral, not a final answer |
| Recursive resolver asks TLD server for `example.com` NS records | **Iterative** | Same pattern: ask, get referred, ask next |
| Client receives the final A record | **Recursive (answer)** | The recursive resolver fulfilled its obligation |

### The Key Distinction

- **Recursive query:** "Resolve this fully and give me the final answer."
  The client's stub resolver makes recursive queries to its configured
  recursive resolver.

- **Iterative query:** "Give me the best answer you have, or tell me who
  to ask next." The recursive resolver makes iterative queries to root,
  TLD, and authoritative servers.

Most resolvers set the RD (Recursion Desired) flag in queries to their
upstream resolver. If the upstream resolver supports recursion (RA flag
set), it will perform the full resolution on the client's behalf.

### Common Mistakes to Avoid

- **Assuming the client contacts root servers.** The client only talks to
  its configured recursive resolver. All the iterative work happens
  between the resolver and the DNS hierarchy.
- **Confusing the RD and RA flags.** RD (Recursion Desired) is set by the
  client. RA (Recursion Available) is set by the resolver to indicate it
  supports recursive queries.
- **Forgetting about caching.** In practice, most queries are answered from
  cache. The full iterative resolution only happens on cache misses.
- **Thinking `dig +trace` shows real-time resolution.** `dig +trace`
  bypasses the configured recursive resolver and performs iterative
  resolution from the root. This is useful for debugging but is not how
  normal DNS resolution works.

## Key Takeaway

DNS resolution is a hierarchical delegation process. The recursive resolver
acts as an intermediary, performing iterative queries on the client's behalf.
Each level in the hierarchy (root, TLD, authoritative) refers the resolver
to the next level. Understanding this flow is essential for diagnosing DNS
failures -- you need to know which level in the chain is broken.
