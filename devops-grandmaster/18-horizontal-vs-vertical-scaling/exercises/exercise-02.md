# Exercise 02: Component Scaling Classification

**Type:** Guided
**Time:** 30 minutes
**Difficulty:** Easy-Medium

## Objective

Examine a multi-component application architecture and determine the correct scaling strategy for each component, identifying what must change before horizontal scaling is possible.

## Scenario

You inherit an e-commerce application with the following architecture:

```
                         ┌──────────────┐
                         │   Browser    │
                         └──────┬───────┘
                                │
                         ┌──────┴───────┐
                         │  App Server  │
                         │  (Node.js)   │
                         │  Port 3000   │
                         └──┬───┬───┬───┘
                            │   │   │
              ┌─────────────┘   │   └─────────────┐
              │                 │                 │
              v                 v                 v
     ┌────────────────┐ ┌──────────────┐ ┌───────────────┐
     │  PostgreSQL    │ │    Redis     │ │  Local Disk   │
     │  (primary)     │ │  (sessions)  │ │  (uploads)    │
     └────────────────┘ └──────────────┘ └───────────────┘
```

**Current behavior:**
- The app server stores user sessions in Redis (already externalized).
- File uploads are saved to the local filesystem of the app server.
- The app server has an in-memory LRU cache for product catalog lookups.
- The PostgreSQL database handles both reads and writes on a single instance.
- All components run on a single VM.

Traffic is growing. The app server's CPU hits 90% during peak hours. The database is I/O bound. Response times are climbing.

## Tasks

### Part A: Classify Each Component

For each component in the architecture, fill in the table:

| Component | Current Scaling | Can Scale Horizontally? | Blocker (if any) | Recommended Strategy |
|-----------|----------------|------------------------|-------------------|---------------------|
| App Server | | | | |
| PostgreSQL | | | | |
| Redis | | | | |
| Local Disk (uploads) | | | | |
| In-memory LRU cache | | | | |

<details>
<summary>Hint</summary>

For each component, ask: "If I run two copies of this, will it break?" If yes, what specifically breaks?

</details>

### Part B: Externalize the State

The app server cannot be horizontally scaled in its current state. List every piece of state that must be externalized before you can run multiple app server instances. For each piece of state, name the external service that should replace it.

<details>
<summary>Hint</summary>

There are three pieces of state stored locally on the app server: sessions, file uploads, and the product cache. Sessions are already in Redis -- good. What about the other two?

</details>

### Part C: Design the Target Architecture

Draw an ASCII diagram of the architecture *after* all externalization changes, with the app server running behind a load balancer with 3 replicas. Your diagram should show:

- The load balancer
- 3 app server instances
- All external state services
- Which components scale horizontally and which scale vertically

<details>
<summary>Hint</summary>

The load balancer sits between the browser and the app servers. The app servers should have no local state. The database remains a single primary (vertical scaling) but could have read replicas (horizontal for reads).

</details>

### Part D: Write the Docker Compose

Write a `docker-compose.yml` that implements your target architecture with:
- 3 app server replicas
- A PostgreSQL database
- A Redis instance
- An Nginx load balancer

Use resource limits to simulate the constraints.

<details>
<summary>Hint</summary>

Use `deploy.replicas: 3` for the app service. Use `deploy.resources.limits` to set CPU and memory constraints. The Nginx service needs a volume mount for its config file.

</details>

## Success Criteria

- [ ] You correctly identified which components can and cannot scale horizontally.
- [ ] You listed all state that must be externalized from the app server.
- [ ] Your target architecture diagram shows stateless app servers behind a load balancer.
- [ ] Your Docker Compose file defines all services with appropriate configuration.
- [ ] You explained why the database should scale vertically (not horizontally) as the first step.

## What You Should Understand After This Exercise

Horizontal scaling is not just "add more instances" -- it requires the application to be stateless. Every piece of state held in process memory, local disk, or local caches is a blocker. The pattern is always the same: externalize state to dedicated services (Redis, S3, databases), then scale the stateless compute tier horizontally. Databases and caches are typically scaled vertically first, with horizontal strategies (read replicas, sharding) added later when vertical limits are reached.
