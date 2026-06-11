# Exercise 01: Liveness vs Readiness vs Startup Probes

**Type:** Conceptual
**Time:** 20 minutes
**Difficulty:** Easy

## Objective

Explain the difference between liveness, readiness, and startup probes in
container orchestration. Articulate when to use each type, what happens when
each fails, and why conflating them causes production incidents.

## Background

A container showing "Up" in `docker ps` tells you the main process is
running. It does not tell you whether the application inside can serve
requests, has finished initializing, or is stuck in a deadlock. Health check
probes solve this by asking the application specific questions about its
state. There are three distinct questions, and each has different
consequences when the answer is "no."

---

## Tasks

### Part A: Define Each Probe Type

Answer each question in two or three sentences.

1. What is a **liveness probe**, and what action does the orchestrator take
   when it fails?
2. What is a **readiness probe**, and what action does the orchestrator take
   when it fails?
3. What is a **startup probe**, and what problem does it solve that the
   other two do not?
4. Can a container be "alive" but not "ready"? Give a concrete example.

<details>
<summary>Hint</summary>

- Liveness asks: "Are you stuck?" Readiness asks: "Can you handle requests?"
  Startup asks: "Have you finished starting?"
- When liveness fails, the container is restarted. When readiness fails,
  traffic is removed but the container keeps running.
- A slow-starting application might take 60 seconds to initialize. A liveness
  probe with a 30-second timeout would kill it before it ever starts.

</details>

### Part B: Match the Scenario to the Probe

For each scenario below, state which probe type (liveness, readiness, or
startup) is most appropriate and explain why.

1. A Java application takes 45 seconds to load its Spring context before it
   can serve any requests.
2. A web server has a known bug where it occasionally enters an infinite loop
   and stops responding to requests.
3. A service depends on a database. The database goes down for 30 seconds
   during a failover. The service itself is not broken.
4. A machine learning model server loads a 2 GB model into memory at startup.
   During loading, it cannot respond to HTTP requests.
5. A message queue consumer stops processing messages but the process is
   still running and responding to HTTP health checks.

<details>
<summary>Hint</summary>

- If the problem is "it has not finished starting yet," that is a startup
  probe concern.
- If the problem is "it is broken and needs to be restarted," that is a
  liveness probe concern.
- If the problem is "it is temporarily unable to handle work but will
  recover on its own," that is a readiness probe concern.
- Be careful with scenario 5 -- think about what "healthy" means for a
  worker that does not serve HTTP.

</details>

### Part C: Identify the Misconfiguration

For each configuration, explain what will go wrong in production.

**Configuration 1:**
```yaml
# A Java app with a 60-second startup time
livenessProbe:
  httpGet:
    path: /healthz
    port: 8080
  initialDelaySeconds: 10
  periodSeconds: 10
  failureThreshold: 3
# No startup probe, no readiness probe
```

**Configuration 2:**
```yaml
# Liveness probe checks the database connection
livenessProbe:
  httpGet:
    path: /ready
    port: 8080
  periodSeconds: 5
  failureThreshold: 3
# The /ready endpoint returns 503 if the database is unreachable
```

**Configuration 3:**
```yaml
# Readiness probe returns 200 unconditionally
readinessProbe:
  httpGet:
    path: /healthz
    port: 8080
  periodSeconds: 10
# The /healthz endpoint always returns 200, even during startup
```

<details>
<summary>Hint</summary>

- Think about what happens when a slow-starting app gets killed by its own
  liveness probe.
- Think about what happens when a transient database outage triggers a
  container restart loop.
- Think about what happens when traffic is sent to a container that has not
  finished initializing.

</details>

### Part D: Design a Probe Strategy

You are deploying a Python Flask application that:

- Takes 5-15 seconds to start (varies by load)
- Depends on PostgreSQL and Redis
- Occasionally enters a bad state where it stops responding (requires restart)
- Should not receive traffic until both PostgreSQL and Redis are connected

Write the probe configuration you would use (YAML or describe it). Specify
the endpoint paths, timing, and thresholds. Explain your choices.

<details>
<summary>Hint</summary>

- Use a startup probe to handle the variable startup time.
- Use a liveness probe that checks only the application itself (not
  dependencies).
- Use a readiness probe that checks dependencies.
- The liveness probe endpoint should be lightweight -- it should not call
  external services.

</details>

---

## Success Criteria

- [ ] You can explain the difference between liveness, readiness, and startup
      probes and what happens when each fails
- [ ] You can match a real-world scenario to the correct probe type
- [ ] You can identify misconfigured probes that would cause production issues
- [ ] You can design a probe strategy for an application with dependencies
      and variable startup time
- [ ] You understand why liveness probes should not check external dependencies

## What You Should Understand After This Exercise

The three probe types answer fundamentally different questions. Liveness
asks "should I restart this container?" Readiness asks "should I send traffic
to this container?" Startup asks "has this container finished initializing?"
Using the wrong probe for the wrong purpose leads to cascading failures --
for example, a liveness probe that checks a database will restart perfectly
healthy application containers every time the database blinks.
