# Solution 04: Design Session Management for a Multi-Region Deployment

## Part A: Evaluate the Three Approaches

### 1. Sticky Sessions

**How it works:** The global load balancer assigns each user to a specific region and
ensures all their requests go to that region.

**Can it meet the requirements?**
- Cross-region session survival: NO. If a user travels from US to EU, their session is
  in the US region. The EU load balancer either sends them back to US (high latency) or
  creates a new session (lost state).
- Checkout state safety: PARTIALLY. Within a single region, yes. Across regions, no.
- GDPR compliance: YES. EU users stay in EU region.
- Region failover: NO. If the US region goes down, all US sessions are lost.
- Latency under 10ms: YES. Local instance memory is sub-millisecond.

### 2. Shared Redis (Single Global Cluster)

**How it works:** All regions connect to one Redis cluster (e.g., in US-East).

**Can it meet the requirements?**
- Cross-region session survival: YES. All regions read the same Redis.
- Checkout state safety: YES. Single source of truth.
- GDPR compliance: NO. EU user data is stored in US-East.
- Region failover: PARTIALLY. If US-East (where Redis lives) goes down, all regions lose sessions.
- Latency under 10ms: NO. EU to US-East round-trip is ~80ms. AP to US-East is ~180ms.

### 3. JWT Tokens

**How it works:** All session state is in the token. No server-side storage.

**Can it meet the requirements?**
- Cross-region session survival: YES. Token works everywhere.
- Checkout state safety: NO. Checkout state (cart, step 3 of 5) is too large and
  changes too frequently for a JWT token. Losing the token means losing checkout state.
- GDPR compliance: YES. No server-side storage of EU data (but the token is on the
  client, which raises questions about what counts as "storage").
- Region failover: YES. Tokens work in any region.
- Latency under 10ms: YES. Token verification is local CPU, sub-millisecond.

### Comparison Table

```
| Requirement              | Sticky Sessions | Shared Redis | JWT Tokens    |
|--------------------------|-----------------|--------------|---------------|
| Cross-region survival    | NO              | YES          | YES           |
| Checkout state safety    | PARTIAL         | YES          | NO            |
| GDPR compliance          | YES             | NO           | YES           |
| Region failover          | NO              | PARTIAL      | YES           |
| Latency < 10ms           | YES             | NO           | YES           |
| Revocation speed         | INSTANT         | INSTANT      | SLOW (TTL)    |
```

**Conclusion:** No single approach satisfies all requirements.

## Part B: Design a Hybrid Approach

### Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Global Load Balancer                      │
│              (latency-based routing)                         │
└──────┬──────────────────┬──────────────────┬────────────────┘
       │                  │                  │
       v                  v                  v
┌──────────────┐  ┌──────────────┐  ┌──────────────┐
│   US-East    │  │   EU-West    │  │   AP-Southeast│
│              │  │              │  │               │
│ ┌──────────┐ │  │ ┌──────────┐ │  │ ┌──────────┐  │
│ │App (JWT) │ │  │ │App (JWT) │ │  │ │App (JWT) │  │
│ │verify    │ │  │ │verify    │ │  │ │verify    │  │
│ └────┬─────┘ │  │ └────┬─────┘ │  │ └────┬─────┘  │
│      │       │  │      │       │  │      │        │
│ ┌────v─────┐ │  │ ┌────v─────┐ │  │ ┌────v─────┐  │
│ │Redis     │ │  │ │Redis     │ │  │ │Redis     │  │
│ │(sessions)│ │  │ │(sessions)│ │  │ │(sessions)│  │
│ │US data   │ │  │ │EU data   │ │  │ │AP data   │  │
│ └──────────┘ │  │ └──────────┘ │  │ └──────────┘  │
│              │  │              │  │               │
│ ┌──────────┐ │  │ ┌──────────┐ │  │ ┌──────────┐  │
│ │PostgreSQL│ │  │ │PostgreSQL│ │  │ │PostgreSQL│  │
│ │ primary  │ │  │ │replica   │ │  │ │replica   │  │
│ └──────────┘ │  │ └──────────┘ │  │ └──────────┘  │
└──────────────┘  └──────────────┘  └──────────────┘
```

### Data Distribution Strategy

**JWT Token (client-side, works in any region):**
- `user_id` -- unique identifier
- `role` -- user's permission level
- `home_region` -- user's registered region (for GDPR routing)
- `permissions` -- what the user can access
- `exp` -- short expiration (15 minutes)
- `jti` -- unique token ID for revocation

**Regional Redis (server-side, region-scoped):**
- Cart contents (changes frequently, may contain product data)
- Checkout state (step tracking, payment method selection)
- Temporary UI state (filters, search results)
- Session metadata (last activity, session start time)

**Database (server-side, globally replicated):**
- User profile (name, email, address)
- Order history
- Payment methods (tokenized)
- User preferences

### GDPR Compliance

The `home_region` claim in the JWT token determines where session data is stored:

```python
def get_redis_for_user(user):
    if user['home_region'] == 'eu-west':
        return redis_eu  # EU data stays in EU
    elif user['home_region'] == 'ap-southeast':
        return redis_ap
    else:
        return redis_us
```

When a US user travels to EU-West, their session data is read from US-East Redis
(higher latency, but GDPR does not apply to US users). When an EU user travels to
US-East, their session data stays in EU-West Redis (higher latency, but GDPR compliant).

### Region Failover

```
Normal operation:
  User → Global LB → Home Region → JWT verify + Redis session → Response

Region failover:
  User → Global LB → Nearest Available Region → JWT verify (works anywhere)
       → Redis session (from home region, higher latency)
       → If home region Redis is down: session lost, but JWT identity persists
       → User re-authenticates (JWT proves identity) but loses cart/checkout state
```

Failover graceful degradation:
1. JWT authentication works in any region (identity is in the token)
2. If the home region's Redis is down, the user loses their cart and checkout state
3. The user can still browse products and add new items to a fresh cart
4. When the home region recovers, sessions resume normally

## Part C: Handle the Edge Cases

### Scenario 1: US User Starts Checkout, Flows to Europe

```
1. User in US-East adds items to cart (stored in US-East Redis)
2. User flies to Europe, connects to EU-West
3. EU-West verifies JWT token (valid, works anywhere)
4. EU-West checks session for cart data
5. EU-West reads from US-East Redis (cross-region, ~80ms latency)
6. Cart and checkout state are retrieved
7. User continues checkout step 3
8. Updated checkout state is written back to US-East Redis
```

**Trade-off:** The cross-region Redis read adds ~80ms latency. For a checkout flow
(this is not a latency-sensitive API call), this is acceptable. If it were unacceptable,
you could replicate Redis data across regions, but this introduces consistency challenges.

### Scenario 2: EU-West Region Goes Down for 30 Minutes

```
1. Global LB detects EU-West is unhealthy
2. EU traffic is routed to US-East (next closest)
3. EU users' JWT tokens are verified in US-East (works fine)
4. EU users' session data is in EU-West Redis (unreachable)
5. Session lookup fails → users are treated as authenticated but sessionless
6. Users can browse, but cart is empty and checkout state is lost
7. When EU-West recovers, sessions are available again
```

**Impact:** EU users lose their cart for 30 minutes. They are still authenticated
(JWT works) but must rebuild their cart. This is degraded but not catastrophic.

**Mitigation:** For critical checkout flows, write checkout state to the database
(not just Redis) as a fallback. The database has a replica in each region.

### Scenario 3: JWT Token Expires Mid-Checkout

```
1. User is on step 3 of checkout
2. Access token expires (15 minutes have passed)
3. Next API call returns 401
4. Client-side code detects 401
5. Client sends refresh token to /refresh endpoint
6. New access token is issued
7. Client retries the step 3 request with new token
8. Checkout continues seamlessly
```

**Key insight:** The refresh token endpoint must be available in the failover region.
Since refresh tokens are verified with a separate secret and do not require Redis,
this works in any region.

**Client-side handling:**
```javascript
async function apiCall(url, options) {
    let response = await fetch(url, options);
    if (response.status === 401) {
        // Try to refresh
        const refreshed = await refreshAccessToken();
        if (refreshed) {
            // Retry with new token
            options.headers['Authorization'] = `Bearer ${newAccessToken}`;
            response = await fetch(url, options);
        }
    }
    return response;
}
```

### Scenario 4: AP-Southeast Redis Loses Data

```
1. AP-Southeast Redis crashes without persistence
2. All sessions stored in AP Redis are lost
3. AP users' next request: JWT verified (works), session lookup fails
4. Users are treated as authenticated but sessionless
5. Cart is empty, checkout state is lost
6. Users must re-add items to cart
```

**Impact:** AP users (10% of total) lose their cart. They do not lose their identity
(JWT still works). They do not need to re-enter credentials.

**Mitigation:**
- Enable Redis persistence (RDB snapshots every 5 minutes)
- For critical data (checkout step 3+), also write to the database
- Accept that some session data loss is a trade-off of the regional design

### Scenario 5: Attacker Steals a JWT Token

```
1. Attacker intercepts a user's JWT token (e.g., XSS, network sniffing)
2. Attacker sends requests with the stolen token
3. Server verifies the signature (valid) and processes requests
4. Attacker has full access to the user's account for 15 minutes
```

**Why this is dangerous:** JWT tokens are bearer tokens. Anyone who has the token
can use it. There is no binding to the client's IP, device, or browser.

**Mitigations:**
- Short token lifetime (15 minutes) limits the attack window
- Use HTTPS everywhere to prevent network sniffing
- Store tokens in httpOnly cookies (not localStorage) to prevent XSS theft
- Implement anomaly detection (sudden region change, unusual activity patterns)
- For high-security operations (payment, password change), require re-authentication

## Part D: Operational Runbooks

### Runbook 1: Adding a New Region (South America)

```
1. Provision infrastructure in SA-East (Sao Paulo):
   - Application instances (3 minimum)
   - Redis cluster
   - PostgreSQL read replica

2. Configure replication:
   - PostgreSQL: set up streaming replication from US-East primary
   - Redis: configure cross-region replication if needed

3. Configure the global load balancer:
   - Add SA-East as a target
   - Set latency-based routing weights
   - Health checks for the new region

4. Deploy the application:
   - Use the same Docker image as other regions
   - Configure JWT_SECRET (must be the same across all regions)
   - Configure REDIS_HOST to point to SA-East Redis
   - Configure DATABASE_URL to point to SA-East replica

5. Test:
   - Verify JWT tokens issued in US-East work in SA-East
   - Verify session lookup routes to the correct region
   - Verify GDPR routing for EU users (should still go to EU Redis)

6. Monitor:
   - Latency metrics for the new region
   - Error rates
   - Redis hit rates
   - Cross-region traffic patterns
```

### Runbook 2: Rotating the JWT Signing Key

```
Phase 1: Dual-key support (duration: 1 week)
  1. Generate a new signing key (JWT_SECRET_NEW)
  2. Deploy both keys to all regions:
     - JWT_SECRET (old) for verification
     - JWT_SECRET_NEW for signing new tokens
  3. All new tokens are signed with JWT_SECRET_NEW
  4. All tokens (old and new) are verified:
     - Try JWT_SECRET_NEW first
     - Fall back to JWT_SECRET if verification fails
  5. Monitor for verification failures

Phase 2: Remove old key (after old tokens expire)
  1. Wait for the longest token lifetime to pass (7 days for refresh tokens)
  2. Remove JWT_SECRET from all regions
  3. Only JWT_SECRET_NEW is active
  4. Monitor for any 401 spikes (indicating old tokens still in circulation)

Rollback plan:
  - If 401 rates spike, re-add JWT_SECRET as a fallback verification key
  - Investigate which tokens are failing (check logs for token age)
```

**Why this works:** During the transition period, both keys are valid. New tokens use
the new key, but old tokens signed with the old key are still accepted. No user is
logged out. After the transition period (7 days, the refresh token lifetime), all
old tokens have expired naturally.

### Runbook 3: Redis Cluster Failover Within a Region

```
1. Detect failure:
   - Redis health check fails
   - Application logs show Redis connection errors
   - Alert fires (monitoring should catch this within 30 seconds)

2. Automatic failover (if using Redis Sentinel or Cluster):
   - Redis Sentinel promotes a replica to primary
   - Application reconnects to the new primary
   - Session data is preserved (replicas had the data)
   - Expected downtime: 10-30 seconds

3. If automatic failover fails:
   - Manually promote a replica:
     redis-cli -h redis-replica SLAVEOF NO ONE
   - Update application configuration to point to the new primary
   - Restart application instances if they cannot reconnect automatically

4. If all Redis nodes are lost:
   - Start a new Redis instance
   - All sessions are lost
   - Users must re-authenticate
   - JWT tokens still work (identity is preserved)

5. Post-incident:
   - Investigate root cause of Redis failure
   - Review Redis persistence configuration
   - Consider increasing replica count
   - Update runbook if any steps were unclear
```

### Runbook 4: GDPR Data Deletion Request

```
1. Identify all data for the user:
   - Database: user profile, order history, payment methods
   - Redis (EU-West): current session data, cart, checkout state
   - JWT tokens: cannot be deleted from clients (fundamental limitation)

2. Delete from database:
   DELETE FROM users WHERE id = <user_id>;
   DELETE FROM orders WHERE user_id = <user_id>;
   DELETE FROM payment_methods WHERE user_id = <user_id>;

3. Delete from Redis:
   # Find all session keys for this user
   # This requires a secondary index (user_id → session_ids)
   SCAN for sessions belonging to the user
   DELETE each session key

4. Handle JWT tokens:
   - Add user_id to a permanent blacklist in Redis
   - All future JWT verification checks this blacklist
   - Existing tokens will be rejected on next request
   - Tokens on client devices are "dead" (server rejects them)

5. Verify deletion:
   - Query database for any remaining user data
   - Check Redis for any remaining session data
   - Test that the user's JWT tokens are rejected

6. Document:
   - Record the deletion request and completion date
   - This is a legal requirement under GDPR Article 17
```

**The JWT problem:** You cannot delete data from a JWT token that is stored on a
client's device. The solution is server-side rejection: maintain a blacklist of
deleted user IDs and reject any token containing a blacklisted user_id. This adds
one Redis lookup per request, partially defeating statelessness, but it is the
only way to comply with GDPR deletion requirements.

## Key Takeaway

Multi-region session management forces you to confront the fundamental trade-offs of
every approach. No single strategy satisfies all requirements. The hybrid approach --
JWT for identity, regional Redis for session state, database for persistence -- is
the most common production solution. The architect's job is to decide which trade-offs
are acceptable for the specific business requirements.
