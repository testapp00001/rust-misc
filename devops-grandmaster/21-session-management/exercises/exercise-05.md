# Exercise 05: Sticky Sessions vs Shared State vs Stateless -- A Comparison

**Type:** Integration
**Time:** 45 minutes
**Difficulty:** Medium

## Objective

Compare the three primary session management strategies -- sticky sessions, shared
state (Redis), and stateless tokens (JWT) -- across multiple dimensions. This exercise
teaches you to evaluate trade-offs systematically and choose the right approach for
a given situation.

## Scenario

You are a DevOps consultant. Five different clients have asked you to recommend a
session management strategy. Each client has different constraints.

**Client A: Internal Admin Tool**
- 50 users, all on the corporate network
- Single server, no plans to scale
- Developers are junior, limited DevOps experience
- Compliance requires audit logging of all actions

**Client B: E-Commerce Startup**
- 10,000 concurrent users, growing 50% per quarter
- Currently one server, planning to scale to 5-10 instances
- Microservices architecture with 3 services
- Budget is tight, team is small

**Client C: Financial Trading Platform**
- 1,000 concurrent users, but each session is critical
- Must never lose a session mid-transaction
- Strict regulatory requirements (session audit trail)
- High budget, dedicated operations team

**Client D: Social Media API**
- 1 million concurrent users
- Stateless REST API consumed by mobile apps
- No browser cookies (mobile clients only)
- Extreme horizontal scaling requirements

**Client E: Healthcare Portal**
- 5,000 concurrent users
- HIPAA compliance (strict data handling rules)
- Sessions contain sensitive patient data
- Must support session revocation within 30 seconds

## Tasks

### Part A: Build the Comparison Matrix

Fill in the following table for each of the three approaches. Rate each dimension
as Low, Medium, or High, and provide a one-sentence explanation.

```
| Dimension                  | Sticky Sessions | Redis Shared State | JWT Stateless |
|----------------------------|-----------------|--------------------|              |
| Horizontal scalability     |                 |                    |              |
| Failover resilience        |                 |                    |              |
| Implementation complexity  |                 |                    |              |
| Infrastructure cost        |                 |                    |              |
| Session data size limit    |                 |                    |              |
| Revocation speed           |                 |                    |              |
| Cross-service support      |                 |                    |              |
| Latency per request        |                 |                    |              |
| Data sovereignty control   |                 |                    |              |
| Audit trail capability     |                 |                    |              |
```

<details>
<summary>Hint</summary>

For "Session data size limit":
- Sticky sessions: limited by instance memory
- Redis: limited by Redis memory (but shared)
- JWT: limited by HTTP header size (typically 8KB max for cookies, more for
  Authorization headers, but larger tokens mean more bandwidth per request)

For "Revocation speed":
- Sticky sessions: instant (just drop the session from memory)
- Redis: instant (delete the key)
- JWT: only if you add a blacklist (adds shared state)

</details>

### Part B: Match Clients to Strategies

For each of the five clients, recommend **one** primary strategy. Justify your choice
with at least three specific reasons tied to their requirements. Also identify the
biggest risk of your recommendation and how to mitigate it.

<details>
<summary>Hint</summary>

Not every client needs the most scalable solution. Client A has 50 users on a single
server -- over-engineering their session management wastes time and adds complexity.

Client D has a mobile API -- cookies are not an option. What does that eliminate?

Client E needs fast revocation -- what does that eliminate?

</details>

### Part C: Identify the Hybrid Cases

For at least **two** clients, explain why a hybrid approach (combining two strategies)
might be better than a single strategy. Describe what data goes where and why.

<details>
<summary>Hint</summary>

Client B (e-commerce) might use JWT for authentication across microservices but Redis
for cart state. The cart is large and changes frequently -- putting it in a JWT token
would mean sending a large token on every request.

Client C (trading) might use Redis for transaction state but JWT for identity. If
Redis fails, the JWT still proves who the user is, and they can re-authenticate
without re-entering credentials.

</details>

### Part D: Design the Migration Path

Client B currently uses sticky sessions. They need to migrate to a better approach
without disrupting their 10,000 users. Write a step-by-step migration plan that:

1. Can be executed incrementally (not a big-bang cutover)
2. Has a rollback plan at each step
3. Does not require users to re-login during the migration
4. Can be completed within one week

<details>
<summary>Hint</summary>

Consider this phased approach:
- Phase 1: Add Redis alongside existing in-memory sessions (dual-write)
- Phase 2: Read from Redis, fall back to in-memory (dual-read)
- Phase 3: Read/write only from Redis
- Phase 4: Remove in-memory storage, remove sticky session configuration

At each phase, the old behavior still works if the new one fails.

</details>

## Success Criteria

- [ ] Your comparison matrix covers all 10 dimensions with ratings and explanations
- [ ] Each client recommendation has at least 3 specific justifications
- [ ] You identify the biggest risk for each recommendation
- [ ] At least 2 clients get a hybrid recommendation
- [ ] Your migration plan has at least 3 phases with rollback capability
- [ ] You can explain why "the best session strategy" does not exist -- only the best
  strategy for a specific set of constraints

## What You Should Understand After This Exercise

Session management is not a one-size-fits-all problem. The right strategy depends on
scale, compliance requirements, team expertise, budget, and architectural constraints.
A junior team should not be fighting with JWT refresh token rotation. A million-user
API cannot use sticky sessions. A compliance-heavy system needs audit trails that
pure JWT cannot provide. The mark of a skilled architect is choosing the simplest
solution that meets all requirements -- and knowing when to combine approaches.
