# Exercise 01: The Container Shutdown Lifecycle

**Type:** Conceptual
**Time:** 20 minutes
**Difficulty:** Easy

## Objective

Understand exactly what happens when a container is stopped -- from the initial
signal through the grace period to forced termination. You will trace the shutdown
sequence step by step and explain why each phase matters for production reliability.

## Background

When you run `docker stop my_app`, the container does not simply vanish. Docker
follows a precise sequence of events. If you do not understand this sequence,
you cannot reason about dropped requests, corrupted data, or failed deployments.

```
The Docker Stop Sequence:

  docker stop my_app
        |
        v
  SIGTERM sent to PID 1
        |
        v
  Grace period (default: 10 seconds)
  Application can clean up here
        |
        v
  Still running?
        |
   YES  |  NO --> Container stops cleanly
        v
  SIGKILL sent to PID 1
        |
        v
  Process is destroyed immediately
  No cleanup possible
```

---

## Tasks

### Part A: Trace the Timeline

A container running a Flask web server receives `docker stop`. The server has
3 in-flight requests: one finishes in 2 seconds, one in 8 seconds, and one in
15 seconds. The default 10-second grace period applies.

Fill in the timeline below with what happens at each point in time:

```
T=0s:   ...
T=2s:   ...
T=8s:   ...
T=10s:  ...
T=15s:  (what happens to this request?)
```

<details>
<summary>Hint</summary>

Think about what signal is sent at T=0s, what the application should be doing
during the grace period, and what happens when the grace period expires.

</details>

### Part B: SIGTERM vs SIGKILL

Explain the difference between SIGTERM and SIGKILL by filling in this table:

| Aspect | SIGTERM | SIGKILL |
|--------|---------|---------|
| Can the process catch/handle it? | ? | ? |
| Can the process do cleanup? | ? | ? |
| What happens to open TCP connections? | ? | ? |
| What happens to in-flight requests? | ? | ? |
| What happens to buffered log data? | ? | ? |

<details>
<summary>Hint</summary>

SIGTERM is a polite request. SIGKILL is an order. Think about which one gives
the process a chance to run its signal handlers and which one bypasses them
entirely.

</details>

### Part C: Why PID 1 Matters

In a Docker container, the main process runs as PID 1. Explain:

1. Why does the signal go to PID 1 and not to all processes in the container?
2. What is special about PID 1 in Linux regarding signal handling?
3. If your Dockerfile uses `CMD shell_form` vs `CMD ["exec_form"]`, how does
   that affect which process receives SIGTERM?

<details>
<summary>Hint</summary>

Consider the difference between the shell form and exec form of CMD. In shell
form, `/bin/sh -c` becomes PID 1, and your application is a child process.
Signals sent to PID 1 may not be forwarded to child processes.

</details>

### Part D: Choosing the Right Grace Period

Your application has these characteristics:
- p99 request latency: 12 seconds
- Average database connection cleanup: 3 seconds
- Log buffer flush: 1 second

What `stop_grace_period` would you set in Docker Compose? Explain your reasoning.
What happens if you set it too low? What happens if you set it too high?

<details>
<summary>Hint</summary>

The grace period must be long enough for the slowest in-flight request to
complete plus cleanup time. But there are costs to setting it too high --
think about deployment speed and incident recovery.

</details>

---

## Success Criteria

- [ ] You can trace the complete shutdown timeline including what happens to each in-flight request
- [ ] You can explain at least 4 differences between SIGTERM and SIGKILL
- [ ] You understand why PID 1 matters and how shell form vs exec form affects signal delivery
- [ ] You can calculate an appropriate grace period given application characteristics
- [ ] You understand the trade-offs between too-short and too-long grace periods

## What You Should Understand After This Exercise

The container shutdown lifecycle is not a single event -- it is a multi-phase
sequence with specific timing constraints. SIGTERM gives the application a
window to finish work and clean up. SIGKILL is the last resort that destroys
everything. Understanding this sequence is the foundation for building
applications that survive restarts without dropping user requests.
