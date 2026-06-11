# Exercise 01: Why Stateless Design Enables Horizontal Scaling

**Type:** Conceptual
**Time:** 20 minutes
**Difficulty:** Easy

## Objective

Explain why stateless application design is a prerequisite for effective horizontal scaling.
This exercise trains you to reason about the relationship between application state and scalability.

## Scenario

You are reviewing the architecture of a web application before it launches. The team plans
to scale horizontally by running multiple instances behind a load balancer. Here is the
current design:

```
┌─────────────┐
│   Client    │
└──────┬──────┘
       │
       v
┌──────────────┐
│ Load Balancer│ (round-robin)
└──┬───┬───┬───┘
   │   │   │
   v   v   v
┌─────┐ ┌─────┐ ┌─────┐
│App 1│ │App 2│ │App 3│
│     │ │     │ │     │
│sess │ │sess │ │sess │
│store│ │store│ │store│
└─────┘ └─────┘ └─────┘
```

Each application instance stores user sessions in its own local memory. The load
balancer distributes requests using round-robin.

## Tasks

### Part A: Identify the Failure Modes

Describe at least **three** concrete user-facing problems that will occur with this
architecture. For each problem, trace the exact request flow that causes it.

<details>
<summary>Hint</summary>

Think about what happens when:
- A user's second request lands on a different instance than their first
- An instance crashes or restarts
- The team tries to add or remove instances during deployment

</details>

### Part B: Quantify the Impact

Assume the application has 10,000 concurrent users and 3 instances. Each session
stores 5 KB of data. Calculate:

1. Total session memory across all instances
2. What percentage of users will experience a "lost session" on their next request
   (assuming round-robin distribution)
3. How much memory is wasted by duplicated session data if sessions were somehow
   replicated across all instances

<details>
<summary>Hint</summary>

For question 2, think about the probability that a user's next request goes to a
different instance than the one that holds their session. With round-robin and 3
instances, what fraction of requests go to a different instance?

</details>

### Part C: Design the Stateless Alternative

Redraw the architecture diagram so that:
- Any instance can handle any request
- No user-facing failures occur when an instance is added, removed, or crashes
- Session data survives instance restarts

Label every component and explain what each one stores.

<details>
<summary>Hint</summary>

There are two main approaches:
1. Externalize session state to a shared store (Redis, database)
2. Eliminate server-side sessions entirely (JWT tokens)

Your diagram should show where session data lives and how instances access it.

</details>

### Part D: The Trade-off Question

Stateless design is not free. List at least **three** costs or trade-offs of making
an application stateless. Consider operational complexity, latency, and failure modes.

<details>
<summary>Hint</summary>

Think about:
- What new infrastructure you need to manage
- What happens when the session store goes down
- Token size and network overhead
- The complexity of token revocation

</details>

## Success Criteria

- [ ] You can describe at least 3 failure modes of in-memory sessions behind a load balancer
- [ ] You can calculate the probability of session loss with round-robin distribution
- [ ] You can draw a stateless architecture with all components labeled
- [ ] You can list at least 3 trade-offs of stateless design
- [ ] You understand why "stateless" means "state lives somewhere other than the app instance"

## What You Should Understand After This Exercise

Horizontal scaling requires that any instance can handle any request. If an application
stores state in local memory, requests become dependent on which instance handles them.
This dependency is the fundamental barrier to scaling. Stateless design moves that state
out of the application instances -- either to a shared store or into the request itself.
