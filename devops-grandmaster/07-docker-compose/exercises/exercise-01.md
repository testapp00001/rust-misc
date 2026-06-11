# Exercise 01: Compose vs. Commands -- The Real Tradeoffs

**Type:** Conceptual
**Time:** 15 minutes
**Difficulty:** Easy

## Objective

Understand *why* Docker Compose exists by analyzing the concrete failures of the
manual `docker run` approach. This exercise trains you to think declaratively
instead of imperatively.

## Scenario

Your team runs a three-service application: a PostgreSQL database, a Redis cache,
and a Node.js API server. A junior colleague wrote this deployment script:

```bash
#!/bin/bash

docker network create myapp || true

docker run -d --name postgres \
  -e POSTGRES_PASSWORD=secret \
  -v pgdata:/var/lib/postgresql/data \
  postgres:16-alpine

sleep 3

docker run -d --name redis redis:7-alpine

docker run -d --name api \
  -e DATABASE_URL=postgresql://postgres:secret@postgres:5432/mydb \
  -e REDIS_URL=redis://redis:6379 \
  -p 3000:3000 \
  --network myapp \
  myapp-api:1.0

echo "All services started!"
```

The same colleague then wrote this Compose file as an alternative:

```yaml
services:
  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_PASSWORD: secret
    volumes:
      - pgdata:/var/lib/postgresql/data

  redis:
    image: redis:7-alpine

  api:
    build: ./api
    ports:
      - "3000:3000"
    environment:
      DATABASE_URL: postgresql://postgres:secret@postgres:5432/mydb
      REDIS_URL: redis://redis:6379
    depends_on:
      - postgres
      - redis

volumes:
  pgdata:
```

## Tasks

### Part A: Failure Modes of the Script

List at least **five** ways the bash script can fail or produce unexpected behavior
that the Compose file handles correctly. For each failure, explain:
- What goes wrong
- Under what conditions it happens
- How Compose avoids or solves it

<details>
<summary>Hint</summary>

Think about:
- What happens if you run the script twice?
- What happens if PostgreSQL takes longer than 3 seconds to start?
- What happens if the API container crashes at 3 AM?
- How do you see logs from all three services at once?
- How do you stop everything cleanly?
- What happens on a teammate's Windows machine?

</details>

### Part B: The Declarative Advantage

The bash script is **imperative** (it says *how* to do each step). The Compose file
is **declarative** (it says *what* the desired state should be). Explain three concrete
advantages of the declarative approach using the scenario above.

<details>
<summary>Hint</summary>

Consider:
- What happens when you add a fourth service to each approach?
- How does each approach handle "what is currently running?"
- What does each approach document about the system?

</details>

### Part C: What the Script Gets Right

The Compose file is not strictly better in every way. Identify at least **two**
things the bash script approach makes more visible or explicit than the Compose file.
This is about honest trade-off analysis, not declaring a winner.

<details>
<summary>Hint</summary>

Think about:
- Startup ordering and timing
- What a new team member sees when reading each file
- Debugging when something goes wrong

</details>

### Part D: The Missing Pieces

The Compose file above still has problems. Identify at least **three** things it is
missing that a production deployment would need. Reference the "Production Way"
section of the module README.

<details>
<summary>Hint</summary>

Look at:
- What `depends_on` actually waits for
- What happens when containers restart
- Network isolation
- Resource constraints

</details>

## Success Criteria

- [ ] You can list at least 5 failure modes of the manual script approach
- [ ] You can explain the declarative vs. imperative distinction with concrete examples
- [ ] You can identify honest trade-offs where the script approach has advantages
- [ ] You can spot at least 3 production gaps in the basic Compose file
- [ ] You understand that Compose is not "magic" -- it is a declarative layer over the same Docker API

## What You Should Understand After This Exercise

Docker Compose is not just a convenience wrapper. It solves real operational problems:
idempotent startup, clean shutdown, log aggregation, and declarative state management.
But it is not complete out of the box -- you still need health checks, restart policies,
and resource limits for production use.
