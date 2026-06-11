# Solution 01: Why Stateless Design Enables Horizontal Scaling

## Part A: Identify the Failure Modes

### Failure 1: Session Loss on Instance Switch

A user logs in. Their request is handled by App 1, which creates a session in its
local memory. The user makes a second request. The load balancer routes it to App 2.
App 2 does not have the session in its memory. The user is told they are not logged in.

```
Request 1: User → LB → App 1 → Creates session "abc" → Returns cookie
Request 2: User → LB → App 2 → Looks up "abc" → Not found → 401
```

With round-robin and 3 instances, the probability of hitting a different instance on
the next request is **2/3 (67%)**. Two out of every three requests after login will fail.

### Failure 2: Session Loss on Instance Crash

App 1 holds 3,333 user sessions. The server runs out of memory and the process crashes.
All 3,333 users lose their sessions simultaneously. They must all re-authenticate. If
this happens during a checkout flow, users lose their cart and checkout progress.

### Failure 3: Scaling Paradox

The ops team adds App 4 to handle increased traffic. The new instance starts with zero
sessions. The load balancer routes some traffic to it. Those requests fail because the
new instance has no session data. Adding capacity actually makes things worse temporarily.

### Failure 4: Deployment Disruption

During a rolling deployment, App 1 is restarted to deploy new code. All sessions on
App 1 are lost. Users assigned to App 1 (roughly 1/3 of all users) are logged out.
The ops team cannot drain App 1 gracefully because the session state cannot be
transferred to another instance.

## Part B: Quantify the Impact

### 1. Total Session Memory

```
10,000 users * 5 KB per session = 50,000 KB = ~49 MB total
```

Split across 3 instances: ~16 MB per instance (assuming even distribution).

### 2. Percentage of "Lost Session" Requests

With round-robin and 3 instances, each request has a 1/3 chance of hitting the same
instance that holds the session, and a 2/3 chance of hitting a different one.

```
P(same instance) = 1/3 = 33%
P(different instance) = 2/3 = 67%
```

**67% of all requests after login will fail.** This is not an edge case -- it is the
normal behavior.

### 3. Wasted Memory with Replication

If sessions were replicated to all 3 instances:

```
Each instance stores all 10,000 sessions: 10,000 * 5 KB = 49 MB per instance
Total across 3 instances: 147 MB
Useful data: 49 MB
Wasted (duplicate): 98 MB (67% of total)
```

This is the cost of replication -- 2/3 of memory is redundant copies.

## Part C: Design the Stateless Alternative

### Option 1: External Session Store (Redis)

```
┌─────────────┐
│   Client    │
└──────┬──────┘
       │ (cookie: session_id=abc)
       v
┌──────────────┐
│ Load Balancer│ (round-robin, no sticky needed)
└──┬───┬───┬───┘
   │   │   │
   v   v   v
┌─────┐ ┌─────┐ ┌─────┐
│App 1│ │App 2│ │App 3│
│     │ │     │ │     │
│(no  │ │(no  │ │(no  │
│state)│ │state)│ │state)│
└──┬──┘ └──┬──┘ └──┬──┘
   │       │       │
   └───────┼───────┘
           v
     ┌──────────┐
     │  Redis   │
     │          │
     │ abc →    │
     │ {user:42,│
     │  role:   │
     │  admin}  │
     └──────────┘
```

- **App instances:** Stateless. They hold no session data. Any instance can handle any request.
- **Redis:** Stores all sessions. Every app instance reads/writes here.
- **Client:** Holds a session ID cookie that identifies the session in Redis.

### Option 2: JWT Tokens (No Server-Side State)

```
┌─────────────────┐
│     Client      │
│ stores JWT token│
└───────┬─────────┘
        │ (Authorization: Bearer eyJhbG...)
        v
┌──────────────┐
│ Load Balancer│ (round-robin)
└──┬───┬───┬───┘
   │   │   │
   v   v   v
┌─────┐ ┌─────┐ ┌─────┐
│App 1│ │App 2│ │App 3│
│     │ │     │ │     │
│verify│ │verify│ │verify│
│token │ │token │ │token │
└─────┘ └─────┘ └─────┘
```

- **Client:** Stores the JWT token. Contains all session data (user_id, role, expiration).
- **App instances:** Verify the token signature using a shared secret. No lookup needed.
- **No external store:** No Redis, no database query for session data.

## Part D: The Trade-off Question

### Trade-off 1: New Infrastructure Dependency (Redis approach)

Redis becomes a critical dependency. If Redis goes down, all sessions are lost. You
need to operate, monitor, and scale Redis in addition to your application. This adds
operational complexity and cost.

### Trade-off 2: Token Revocation Difficulty (JWT approach)

JWT tokens are valid until they expire. If a user's account is compromised and you
need to revoke their access immediately, you cannot -- the token is still valid. You
must either wait for expiration or add a blacklist (which reintroduces shared state).

### Trade-off 3: Increased Latency (Redis approach)

Every request now requires a round-trip to Redis to fetch session data. This adds
1-5ms of latency per request. For high-traffic applications, this can be significant
and requires careful Redis sizing and monitoring.

### Trade-off 4: Token Size and Bandwidth (JWT approach)

JWT tokens carry all session data in every request. As you add more claims, the token
grows. A 2 KB token sent on every request adds up: at 1,000 requests/second, that is
2 MB/second of bandwidth just for tokens.

### Trade-off 5: Complexity of Token Refresh (JWT approach)

Short-lived access tokens require a refresh token flow. This adds significant complexity:
token storage on the client, refresh token rotation, handling concurrent refresh requests,
and revoking refresh tokens. This is a common source of bugs.

### Why This Works

Stateless design works because it eliminates the coupling between a specific server
instance and a user's session. By moving state out of the application instance --
either to a shared store or into the client's token -- you ensure that any instance
can handle any request. This is the fundamental prerequisite for horizontal scaling.

### Common Mistakes

- **Assuming stateless means "no state anywhere."** State still exists -- it is just
  not in the app instance. In Redis, state is centralized. In JWT, state is distributed
  to clients. Both are "stateless" from the server's perspective.
- **Choosing JWT because it is "more modern."** JWT solves specific problems (cross-service
  auth, no shared store). If you do not have those problems, Redis sessions are simpler
  and more controllable.
- **Ignoring the revocation problem.** Teams adopt JWT and then realize they need
  immediate logout. They bolt on a Redis blacklist, reintroducing the dependency they
  tried to avoid. Plan for revocation from the start.
