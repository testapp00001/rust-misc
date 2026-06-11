# Exercise 05: Complete Surge-Handling Architecture

**Type:** Integration
**Time:** 45-60 minutes
**Difficulty:** Hard

## Objective

Design and implement a complete traffic surge handling system that combines pre-warming, queue-based architecture, rate limiting, backpressure, and graceful degradation for a real-world scenario.

## Scenario

You are the platform engineer for a ticketing company. A major concert goes on sale Saturday at 10:00 AM. Expected traffic profile:

```
09:00 - 10:00 AM:  Normal traffic (5,000 RPS) + users queuing in virtual waiting room
10:00:00 AM:       SURGE to 200,000 RPS (users clicking "Buy" simultaneously)
10:00 - 10:15 AM:  200,000 RPS sustained (tickets selling out)
10:15 - 10:30 AM:  Traffic drops to 50,000 RPS (remaining users browsing)
10:30 AM onward:   Normal traffic (5,000 RPS)
```

Constraints:
- 500 tickets available
- Each purchase requires: seat selection, payment processing (3-second API call), confirmation email
- Payment API limit: 100 requests/second
- Database can handle 500 writes/second
- Must be fair: first-come-first-served
- Must not double-sell tickets

## Tasks

### Part A: Design the Complete Architecture

Design a multi-layer architecture that handles this scenario. Draw ASCII art showing all components and their interactions. Your architecture must include:

1. **Edge layer**: CDN, WAF, rate limiting
2. **Waiting room**: Virtual queue for users before the sale
3. **Ingestion layer**: Accepting purchase attempts
4. **Queue layer**: Buffering purchase requests
5. **Processing layer**: Fulfilling orders at sustainable rate
6. **Data layer**: Inventory management with consistency guarantees

<details>
<summary>Hint 1</summary>

The waiting room is critical -- it prevents 200,000 users from hitting your API simultaneously. Users wait in the virtual room and are admitted at a controlled rate. The ingestion layer validates requests and checks inventory atomically (Redis DECR for ticket count). The queue buffers validated purchase attempts. Workers process payments at 100/second.

</details>

### Part B: Implement the Pre-Warming Script

Write a pre-warming script that runs at 9:00 AM (1 hour before the sale). It should:

1. Scale application deployments to handle 200,000 RPS ingestion
2. Pre-provision cluster nodes
3. Warm Redis cache with event data (seat map, pricing)
4. Warm CDN cache with static assets
5. Verify all services are healthy
6. Report readiness status

<details>
<summary>Hint 2</summary>

At 200,000 RPS with lightweight ingestion, you need approximately 100-200 pods (each handling 1,000-2,000 RPS for a simple validate-and-queue operation). Pre-provision nodes to avoid the 4-minute Cluster Autoscaler delay. Use `kubectl wait --for=condition=available` to verify readiness.

</details>

### Part C: Implement Inventory Consistency

The most critical part of the system is preventing double-sells. Design an inventory management system that:

1. Uses Redis atomic operations to check and decrement ticket count
2. Prevents overselling even under 200,000 RPS concurrent access
3. Handles the case where a payment fails (return the ticket to inventory)
4. Provides an accurate count of remaining tickets

Write the Redis commands and explain why they are atomic.

<details>
<summary>Hint 3</summary>

Use `WATCH` + `MULTI` + `EXEC` for optimistic locking, or use a Lua script for atomic check-and-decrement. The Lua script approach is simpler: check if count > 0, decrement if yes, return success/failure. This is atomic because Redis executes Lua scripts atomically -- no other command runs during execution.

</details>

### Part D: Design the Monitoring Dashboard

Design a monitoring dashboard that the operations team uses during the sale. Specify:

1. Key metrics to display (with PromQL queries)
2. Alert thresholds for critical conditions
3. Runbook links for each alert
4. Real-time inventory count display

<details>
<summary>Hint 4</summary>

Key metrics: requests/second (by endpoint), queue depth, payment success rate, inventory remaining, error rate, p99 latency, pod count. Alert on: queue depth > 80% capacity, payment failure rate > 5%, inventory < 50 tickets, HPA at max replicas.

</details>

## Success Criteria

- [ ] Architecture diagram shows all 6 layers with clear data flow
- [ ] Pre-warming script scales infrastructure and verifies readiness before the sale
- [ ] Inventory management prevents double-selling using atomic Redis operations
- [ ] Rate limiting admits users from the waiting room at a controlled rate
- [ ] Monitoring dashboard has actionable alerts with runbook links
- [ ] The system handles 200,000 RPS without dropping purchase requests

## What You Should Understand After This Exercise

Handling extreme traffic surges requires combining every strategy from this module: pre-warming for known events, queues for buffering, rate limiting for fairness, backpressure for protection, and graceful degradation for resilience. The hardest problem is not handling the traffic -- it is maintaining data consistency (no double-sells) under extreme concurrency. Atomic operations and careful queue design are the keys to correctness.
