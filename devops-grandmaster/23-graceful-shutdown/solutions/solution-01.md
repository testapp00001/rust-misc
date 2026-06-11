# Solution 01: The Container Shutdown Lifecycle

## Part A: Trace the Timeline

Here is what happens at each point when `docker stop` is issued with 3 in-flight
requests (finishing at 2s, 8s, and 15s) and a 10-second grace period:

```
T=0s:   Docker sends SIGTERM to PID 1.
        The application's signal handler runs.
        The application sets shutting_down = True.
        New requests will now receive 503 responses.
        In-flight count: 3 requests still processing.

T=2s:   Request 1 completes (2-second request).
        In-flight count: 2 requests remaining.
        The application is still in its graceful shutdown loop.

T=8s:   Request 2 completes (8-second request).
        In-flight count: 1 request remaining.

T=10s:  The grace period expires. Docker sends SIGKILL to PID 1.
        The process is destroyed immediately.
        No signal handler runs. No cleanup occurs.

T=15s:  This request never completes. It was in-flight when SIGKILL
        arrived at T=10s. The TCP connection was severed. The client
        receives a "Connection reset by peer" error. Any partial work
        (database writes, file operations) is left in an incomplete state.
```

**Key insight:** The 15-second request is dropped because the grace period
was only 10 seconds. This is exactly why `stop_grace_period` must be
configured to match your application's actual request durations.

### Why This Works

The timeline shows the two-phase nature of container shutdown: SIGTERM gives
the application a chance to finish cleanly, and SIGKILL is the deadline. The
application must complete all work within the grace period or face forced
termination.

### Common Mistakes to Avoid

- **Assuming all requests complete.** They only complete if they finish
  before the grace period expires.
- **Confusing "request started" with "request must finish."** Docker does
  not track individual requests. It only knows the process is still running.

## Part B: SIGTERM vs SIGKILL

| Aspect | SIGTERM | SIGKILL |
|--------|---------|---------|
| Can the process catch/handle it? | Yes -- the process can register a signal handler | No -- the kernel terminates the process directly |
| Can the process do cleanup? | Yes -- the handler can close connections, flush logs, roll back transactions | No -- the process is destroyed immediately |
| What happens to open TCP connections? | The application can close them gracefully (FIN packet) | The kernel resets all connections (RST packet) |
| What happens to in-flight requests? | They can complete if the handler waits for them | They are severed mid-execution |
| What happens to buffered log data? | The handler can flush buffers to disk or stdout | All buffered data is lost |

### Why This Matters

SIGTERM is a *request* to shut down. SIGKILL is a *command* to die. The entire
concept of graceful shutdown exists in the gap between these two signals. If
your application does not handle SIGTERM, every `docker stop` behaves like
`docker kill`.

### Common Mistakes to Avoid

- **Thinking SIGKILL is rare.** It fires every time the grace period expires.
  If your shutdown takes longer than expected, SIGKILL is the result.
- **Ignoring SIGTERM in development.** Developers often test with Ctrl+C
  (SIGINT) and forget to handle SIGTERM. The container runtime sends SIGTERM,
  not SIGINT.

## Part C: Why PID 1 Matters

### 1. Why does the signal go to PID 1?

In Linux, signals are sent to specific processes by PID. Docker sends SIGTERM
to PID 1 because PID 1 is the main process of the container -- the process
specified in the Dockerfile's CMD or ENTRYPOINT. Docker has no knowledge of
child processes; it only communicates with PID 1.

### 2. What is special about PID 1 in Linux?

In normal Linux processes, if a signal is not handled, the default action
applies (e.g., SIGTERM's default is to terminate the process). But PID 1 has
a special exception: **signals without explicit handlers are ignored, not
acted upon.** This means if your application does not register a SIGTERM
handler, SIGTERM is silently ignored, and the grace period expires with
SIGKILL as the only recourse.

This is a Linux kernel behavior, not a Docker behavior. Docker cannot
override it.

### 3. Shell form vs exec form

**Shell form:** `CMD python app.py`
- Docker runs `/bin/sh -c "python app.py"`
- `/bin/sh` becomes PID 1
- `python app.py` is a child process (PID > 1)
- SIGTERM goes to `/bin/sh`, which does NOT forward it to child processes
- The child never receives SIGTERM, never shuts down gracefully
- Result: always SIGKILL after timeout

**Exec form:** `CMD ["python", "app.py"]`
- Docker runs `python app.py` directly
- `python app.py` becomes PID 1
- SIGTERM goes directly to your application
- If the application has a signal handler, it can shut down gracefully

### Why This Works

Understanding PID 1 behavior explains a common production bug: the application
works perfectly when tested locally (run directly, PID 1 is the shell, Ctrl+C
sends SIGINT to the foreground process group), but fails in containers (PID 1
is the application, and SIGTERM without a handler is ignored).

### Common Mistakes to Avoid

- **Using shell form CMD.** Always use exec form (`CMD ["..."]`) unless you
  have a specific reason not to.
- **Testing with SIGINT instead of SIGTERM.** In development, use
  `kill -TERM <pid>` to simulate what Docker does.

## Part D: Choosing the Right Grace Period

**Calculation:**

```
p99 request latency:              12 seconds
Database connection cleanup:       3 seconds
Log buffer flush:                  1 seconds
                                 ----------
Subtotal:                         16 seconds
Safety margin (25%):               4 seconds
                                 ----------
Recommended stop_grace_period:    20 seconds
```

**What happens if too low (e.g., 5 seconds):**
- In-flight requests at p99 latency are killed before completing
- Database connections may not close cleanly (potential connection leaks)
- Buffered logs are lost
- Users see errors during every deployment

**What happens if too high (e.g., 300 seconds):**
- Deployments take 5 minutes per instance (300s timeout worst case)
- During an incident where you need to roll back, recovery is slow
- Old instances linger, consuming resources
- If using Kubernetes, `terminationGracePeriodSeconds` of 300 means each pod
  deletion takes up to 5 minutes

**The right value** balances deployment speed against request completion
probability. Match it to your p99 latency plus cleanup time, plus a safety
margin. Monitor actual shutdown durations in production and adjust.

### Common Mistakes to Avoid

- **Using the default (10 seconds) without thinking.** Many applications have
  requests that take longer than 10 seconds.
- **Setting it to a very large number "just to be safe."** This makes
  deployments and incident recovery unacceptably slow.
- **Not measuring actual shutdown duration.** You cannot tune what you do
  not measure. Log how long shutdowns take and adjust accordingly.

## Key Takeaway

The container shutdown lifecycle is a two-signal sequence: SIGTERM (polite
request) followed by SIGKILL (forced termination). The gap between these
two signals is your only window to shut down cleanly. Everything about
graceful shutdown -- signal handlers, connection draining, preStop hooks,
health checks -- exists to make the best use of this window. If you
understand this timeline, you can reason about any shutdown-related bug.
