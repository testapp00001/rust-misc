# Exercise 04: Design Session Management for a Multi-Region Deployment

**Type:** Challenge
**Time:** 60 minutes
**Difficulty:** Hard

## Objective

Design a session management strategy for a web application deployed across three
geographic regions. This exercise forces you to confront the real-world complexity
of session management when latency, data sovereignty, and fault tolerance matter.

## Scenario

You are the architect for an e-commerce platform. The system is being deployed to:

```
Region: US-East (Virginia)
  - Primary database (PostgreSQL)
  - Redis cluster
  - 5 application instances

Region: EU-West (Frankfurt)
  - Read replica of PostgreSQL
  - Redis cluster
  - 4 application instances

Region: AP-Southeast (Singapore)
  - Read replica of PostgreSQL
  - Redis cluster
  - 3 application instances
```

A global load balancer (like AWS Route 53 or Cloudflare) routes users to the nearest
region based on latency.

**User behavior:**
- 70% of users are in the US, 20% in Europe, 10% in Asia
- Users sometimes travel between regions (a US user flies to Europe)
- The average session lasts 30 minutes
- The checkout process has 5 steps and must not lose state mid-flow
- GDPR requires that EU user data stays in EU-West

**Requirements:**
1. A user's session must survive if they switch regions mid-session
2. Checkout state must not be lost under any circumstances
3. EU user session data must not leave the EU region
4. If a region goes down, users in that region should be able to continue (possibly
   with degraded experience) in another region
5. Session lookup latency must be under 10ms for 99% of requests

## Tasks

### Part A: Evaluate the Three Approaches

For each approach below, describe how it would work in this multi-region scenario,
and identify which requirements it can and cannot meet:

1. **Sticky sessions** -- route each user to the same region/app instance
2. **Shared Redis store** -- all regions connect to a single Redis cluster
3. **JWT tokens** -- all session state in the token, no server-side storage

Create a comparison table.

<details>
<summary>Hint</summary>

For each approach, ask:
- Where does session data physically live?
- What happens when a user switches regions?
- What happens when a region goes down?
- Does it meet the GDPR requirement?
- What is the lookup latency?

</details>

### Part B: Design a Hybrid Approach

No single approach satisfies all requirements. Design a hybrid strategy that combines
elements of multiple approaches. Your design should specify:

1. What data goes in the JWT token
2. What data goes in the regional Redis store
3. What data goes in the primary database
4. How a region failover works
5. How GDPR compliance is maintained

Draw an architecture diagram showing data flow for:
- A normal request (user in their home region)
- A cross-region request (user traveling)
- A region failover scenario

<details>
<summary>Hint</summary>

Consider this split:
- JWT: identity, role, permissions (small, rarely changes, no GDPR issues)
- Regional Redis: cart contents, checkout state, temporary data (large, changes often,
  may contain PII)
- Database: order history, user profile (persistent, replicated)

For GDPR: tag sessions with the user's home region. EU sessions only go to EU Redis.
For failover: JWT tokens work in any region. For Redis data, you need either cross-
region replication (latency cost) or graceful degradation (lose cart, keep identity).

</details>

### Part C: Handle the Edge Cases

For each scenario below, describe exactly what happens with your hybrid design:

1. A US user starts checkout, flies to Europe, and submits step 3 of checkout
2. The EU-West region goes down completely for 30 minutes
3. A user's JWT token expires mid-checkout
4. The Redis cluster in AP-Southeast loses data (crash without persistence)
5. An attacker steals a user's JWT token

<details>
<summary>Hint</summary>

For scenario 1: The JWT token is valid in any region. But the checkout state is in
US-East Redis. Either the EU region must be able to read from US-East Redis (latency),
or the checkout state must be replicated. What is the trade-off?

For scenario 4: If Redis loses data, sessions stored only in Redis are lost. Users
must re-authenticate. But if you kept identity in JWT, they can log back in without
re-entering credentials. How does this affect the checkout flow?

</details>

### Part D: Write the Operational Runbook

Write a runbook (step-by-step procedure) for each of these operational events:

1. **Adding a new region** (e.g., South America)
2. **Rotating the JWT signing key** without logging everyone out
3. **Handling a Redis cluster failover** within a single region
4. **Responding to a GDPR data deletion request**

<details>
<summary>Hint</summary>

For key rotation: Use two keys during transition. The old key verifies existing tokens.
The new key signs new tokens. After the old tokens expire, remove the old key.

For GDPR deletion: You need to find and delete all session data for a specific user.
If session data is only in JWT tokens, you cannot delete it from client-side tokens.
This is a real problem. How do you solve it?

</details>

## Success Criteria

- [ ] You can explain why no single session approach satisfies all requirements
- [ ] Your hybrid design clearly specifies where each type of data lives
- [ ] Your design handles at least 4 of the 5 edge cases correctly
- [ ] Your GDPR solution ensures EU data never leaves the EU region
- [ ] Your runbook for JWT key rotation avoids a service disruption
- [ ] Your architecture diagram shows clear data flow for normal, travel, and failover scenarios

## What You Should Understand After This Exercise

Real-world session management is never as simple as "just use JWT" or "just use Redis."
Every approach has trade-offs. The architect's job is to understand those trade-offs
and choose the right combination for the specific requirements. Multi-region deployments
amplify every trade-off because latency, data sovereignty, and fault tolerance all
interact with session management decisions.
