# Exercise 01: Load Balancing Algorithms (Conceptual)

## Objective

Build a solid mental model of the most common load-balancing algorithms so you
can choose the right one for a given workload.

## Background

A load balancer distributes incoming requests across a pool of backend servers.
The *algorithm* it uses determines which server receives each request. Different
algorithms suit different traffic patterns and backend capabilities.

## Instructions

### Part A -- Algorithm Descriptions

For each algorithm below, write 2-3 sentences describing how it works and one
scenario where it is a good fit.

1. **Round Robin**
2. **Weighted Round Robin**
3. **Least Connections**
4. **IP Hash**
5. **Random**
6. **Least Response Time**

### Part B -- Scenario Matching

Read each scenario and state which algorithm you would choose. Justify your
answer in one or two sentences.

| # | Scenario |
|---|----------|
| 1 | All servers have identical hardware; traffic is uniform. |
| 2 | Servers have different CPU/RAM specs (e.g., 4-core and 16-core machines). |
| 3 | A stateful application where a user must hit the same backend for the duration of a session (and you cannot use sticky sessions at the cookie level). |
| 4 | Backends vary in response time because they depend on external APIs with unpredictable latency. |
| 5 | A simple service with no state; you want the easiest algorithm that works. |
| 6 | Long-lived connections (e.g., WebSocket) are common and you want to avoid overloading a single server. |

### Part C -- Trade-off Analysis

Answer each question in 3-5 sentences.

1. Why might *Least Connections* perform poorly if backend servers have very
   different processing capacities?
2. What happens to *IP Hash* when a large corporate network funnels many users
   through a single NAT IP?
3. Under what conditions does *Random* outperform *Round Robin*?

## Success Criteria

- [ ] Each of the six algorithms has a correct description and a fitting scenario.
- [ ] All six scenario-matching questions are answered with a justified algorithm choice.
- [ ] Trade-off answers demonstrate understanding of edge cases and failure modes.
- [ ] Answers are written in your own words (not copy-pasted from documentation).

## Hints

<details>
<summary>Hint 1 -- Round Robin vs Weighted Round Robin</summary>

Round Robin ignores server capacity. Weighted Round Robin lets you assign a
higher proportion of requests to more powerful machines by giving them a larger
weight value.

</details>

<details>
<summary>Hint 2 -- IP Hash and NAT</summary>

When many users share one public IP, IP Hash treats them as a single client.
This can cause severe imbalance if the hash function maps that IP to one
backend.

</details>

<details>
<summary>Hint 3 -- Least Connections weakness</summary>

Least Connections tracks active connections, not server capacity. A slow 4-core
machine with 10 connections may be more overloaded than a fast 16-core machine
with 10 connections. Capacity-aware variants add a weight to address this.

</details>
