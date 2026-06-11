# Exercise 02: Round-Robin Implementation

**Type:** Guided
**Time:** 30 min
**Difficulty:** Easy-Medium

## Objective

Implement a basic round-robin load balancer, understand its behavior under different conditions, and identify its limitations.

## Scenario

You need to implement a simple round-robin load balancer in pseudocode (or any language you prefer). The load balancer sits in front of three backend servers and distributes incoming requests sequentially.

```
Architecture:

Client Requests → [Load Balancer] → Backend 1 (10.0.1.1)
                                   → Backend 2 (10.0.1.2)
                                   → Backend 3 (10.0.1.3)
```

## Tasks

### Part A: Basic Round-Robin Implementation

Write pseudocode for a round-robin load balancer that:
1. Maintains a list of backend servers
2. Cycles through them in order
3. Returns the next server for each request

```
function get_next_server(backends, state):
    # Your implementation here
```

<details>
<summary>Hint</summary>

You need a counter variable that increments with each request and wraps around using modulo arithmetic. The counter persists across requests (it is stateful).

</details>

### Part B: Trace Request Distribution

Given 3 backends and 12 incoming requests, trace which backend handles each request:

```
Backends: [Server-A, Server-B, Server-C]

Request  1 → Server: ____
Request  2 → Server: ____
Request  3 → Server: ____
Request  4 → Server: ____
Request  5 → Server: ____
Request  6 → Server: ____
Request  7 → Server: ____
Request  8 → Server: ____
Request  9 → Server: ____
Request 10 → Server: ____
Request 11 → Server: ____
Request 12 → Server: ____
```

<details>
<summary>Hint</summary>

Round-robin cycles: A, B, C, A, B, C, A, B, C, A, B, C. After 12 requests, each server handled exactly 4 requests (perfectly even distribution).

</details>

### Part C: Round-Robin Limitations

Now consider this scenario: Server-B is twice as powerful as Server-A and Server-C. It can handle 2x the requests per second. Answer these questions:

1. Does basic round-robin account for server capacity differences?
2. What happens to Server-A and Server-C under high load?
3. How would you modify the algorithm to handle unequal capacity?

<details>
<summary>Hint</summary>

Basic round-robin treats all servers equally. To handle unequal capacity, you need weighted round-robin. Assign weights proportional to capacity: Server-A=1, Server-B=2, Server-C=1. Then Server-B gets 2 out of every 4 requests.

</details>

### Part D: Weighted Round-Robin Implementation

Extend your implementation to support weighted round-robin:

```
Backends:
  Server-A: weight=1, capacity=100 req/s
  Server-B: weight=2, capacity=200 req/s
  Server-C: weight=1, capacity=100 req/s

function get_next_server_weighted(backends_with_weights, state):
    # Your implementation here
```

Trace the first 8 requests with weights [1, 2, 1]:

```
Request 1 → Server: ____
Request 2 → Server: ____
Request 3 → Server: ____
Request 4 → Server: ____
Request 5 → Server: ____
Request 6 → Server: ____
Request 7 → Server: ____
Request 8 → Server: ____
```

<details>
<summary>Hint</summary>

One approach: expand the server list based on weights. Weight [1, 2, 1] becomes [A, B, B, C]. Then apply basic round-robin to this expanded list. Another approach: track a "current weight" for each server and decrement it on each assignment.

</details>

## Success Criteria

- [ ] You can implement basic round-robin in pseudocode
- [ ] You can trace request distribution across backends
- [ ] You understand why round-robin fails with unequal server capacity
- [ ] You can implement weighted round-robin

## What You Should Understand After This Exercise

Round-robin is the simplest load balancing algorithm: it cycles through backends sequentially. It works well when all servers have equal capacity and requests have uniform duration. Its main limitation is that it is stateless about server health and capacity -- it sends requests to servers regardless of how busy they are. Weighted round-robin addresses the capacity problem but still ignores real-time load.
