# Solution 02: Implement a Graceful Shutdown Handler

## Part A: Write the Shutdown Handler

Here is the complete, working implementation:

```python
import signal
import sys
import threading
from flask import Flask, jsonify

app = Flask(__name__)

# State tracking
shutting_down = False
in_flight = 0
lock = threading.Lock()

@app.before_request
def before_request():
    global in_flight
    with lock:
        if shutting_down:
            return jsonify({"error": "Service is shutting down"}), 503
        in_flight += 1

@app.after_request
def after_request(response):
    global in_flight
    with lock:
        in_flight -= 1
    return response

@app.route('/health')
def health():
    if shutting_down:
        return jsonify({"status": "shutting_down"}), 503
    return jsonify({"status": "ok", "in_flight": in_flight}), 200

@app.route('/slow')
def slow():
    import time
    time.sleep(5)
    return jsonify({"result": "done"})

def graceful_shutdown(signum, frame):
    global shutting_down
    shutting_down = True
    print(f"[SHUTDOWN] Signal {signum} received. Starting graceful shutdown...")
    print(f"[SHUTDOWN] {in_flight} in-flight requests.")

    timeout = 30
    start = time.time()
    while in_flight > 0 and (time.time() - start) < timeout:
        print(f"[SHUTDOWN] Waiting for {in_flight} requests to complete...")
        time.sleep(1)

    if in_flight > 0:
        print(f"[SHUTDOWN] Timeout reached. {in_flight} requests did not complete.")
    else:
        print("[SHUTDOWN] All requests completed. Exiting cleanly.")

    sys.exit(0)

signal.signal(signal.SIGTERM, graceful_shutdown)

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5000, threaded=True)
```

### Why This Works

The handler coordinates three things:

1. **`shutting_down` flag** -- Set immediately when SIGTERM arrives. The
   `before_request` hook checks this flag and rejects new requests with 503.
   This prevents the application from starting new work while shutting down.

2. **`in_flight` counter** -- Incremented in `before_request`, decremented in
   `after_request`. This tracks how many requests are currently being processed.
   The shutdown handler waits for this counter to reach 0.

3. **Timeout loop** -- The handler polls `in_flight` every second. If all
   requests complete, it exits immediately. If the timeout expires, it exits
   anyway (better to drop a few requests than to hang forever and get SIGKILL).

The `threading.Lock()` is necessary because Flask with `threaded=True` handles
requests in multiple threads. Without the lock, concurrent modifications to
`in_flight` could cause race conditions (lost increments or decrements).

### Common Mistakes to Avoid

- **Not using a lock.** Without `threading.Lock()`, the `in_flight` counter
  can be corrupted by concurrent access. Two threads might read the same
  value, both increment it, and one increment is lost.
- **Not handling the timeout case.** If you wait forever for in-flight requests,
  the grace period expires and SIGKILL drops them anyway. Always have a timeout.
- **Returning 200 from the health endpoint during shutdown.** This causes the
  load balancer to keep sending traffic to a dying instance.

## Part B: Write the Docker Configuration

**Dockerfile:**
```dockerfile
FROM python:3.11-slim
WORKDIR /app
RUN pip install flask
COPY app.py .
CMD ["python", "-u", "app.py"]
```

**docker-compose.yml:**
```yaml
version: "3.8"

services:
  app:
    build: .
    ports:
      - "5000:5000"
    stop_grace_period: 20s
```

### Why This Works

The `stop_grace_period` is set to 20 seconds. Given a p99 latency of 8 seconds
and 2 seconds for cleanup, 20 seconds provides a 10-second safety margin. This
is long enough for the slowest requests but short enough that deployments do
not take forever.

The Dockerfile uses exec form `CMD ["python", "-u", "app.py"]` so that Python
is PID 1 and receives SIGTERM directly. The `-u` flag disables output buffering
so shutdown log messages appear immediately.

### Common Mistakes to Avoid

- **Using shell form CMD.** `CMD python app.py` runs through `/bin/sh`, which
  does not forward SIGTERM to the child process.
- **Setting stop_grace_period too low.** If set to 5 seconds, the 8-second
  p99 requests are killed before completing.
- **Forgetting the `-u` flag.** Without it, Python buffers stdout and you may
  not see shutdown log messages.

## Part C: Test Your Implementation

```bash
# Terminal 1: Start the application
docker compose up

# Terminal 2: Test graceful shutdown
# Send a slow request in the background
curl http://localhost:5000/slow &
CURL_PID=$!

# Give the request time to start processing
sleep 1

# Stop the container gracefully
docker compose stop app

# Check if the curl request completed successfully
wait $CURL_PID
echo "Exit code: $?"
# Expected: Exit code 0 (request completed)

# Terminal 1 should show:
# [SHUTDOWN] Signal 15 received. Starting graceful shutdown...
# [SHUTDOWN] 1 in-flight requests.
# [SHUTDOWN] Waiting for 1 requests to complete...
# (after ~4 more seconds)
# [SHUTDOWN] All requests completed. Exiting cleanly.
```

To verify the contrast with forced shutdown:

```bash
# Start the app again
docker compose up -d

# Send a slow request
curl http://localhost:5000/slow &
CURL_PID=$!
sleep 1

# Kill the container immediately (no grace period)
docker kill $(docker compose ps -q app)

wait $CURL_PID
echo "Exit code: $?"
# Expected: curl error (56) Recv failure: Connection reset by peer
```

### Why This Works

The test demonstrates the core value of graceful shutdown: the slow request
completes successfully with `docker stop` but fails with `docker kill`. The
background curl process and `wait` allow you to capture the result of a
request that spans the shutdown period.

### Common Mistakes to Avoid

- **Not waiting for the request to start before stopping.** If you stop the
  container before the request arrives, there are no in-flight requests to drain.
- **Testing only the happy path.** Always test both graceful and forced shutdown.

## Part D: The Health Endpoint During Shutdown

The `/health` endpoint should return 503 during shutdown because:

1. **The load balancer uses health checks to decide where to send traffic.**
   If the health check returns 200, the load balancer considers this instance
   healthy and continues routing new requests to it.

2. **During shutdown, the instance cannot reliably handle new requests.**
   The `before_request` hook rejects them with 503, but the load balancer
   does not know this until it gets a 503 from the health check.

3. **Returning 503 from the health check causes the load balancer to remove
   this instance from its routing pool.** New traffic goes to other healthy
   instances. This is exactly what you want during shutdown.

If the health endpoint continued returning 200:

```
Timeline with health returning 200 during shutdown:

T=0s:   SIGTERM received, shutting_down = True
T=0s:   Health check returns 200 (load balancer thinks: "healthy!")
T=1s:   Load balancer sends new request to this instance
T=1s:   before_request returns 503 (user gets an error)
T=2s:   Load balancer sends another new request
T=2s:   before_request returns 503 (another user gets an error)
...     This continues until the load balancer's health check fails
```

With the health endpoint returning 503:

```
Timeline with health returning 503 during shutdown:

T=0s:   SIGTERM received, shutting_down = True
T=0s:   Health check returns 503 (load balancer thinks: "unhealthy!")
T=0s:   Load balancer removes this instance from routing pool
T=1s:   New requests go to other instances (no errors)
T=0-20s: In-flight requests complete normally
T=20s:  Process exits cleanly
```

### Common Mistakes to Avoid

- **Not updating the health endpoint.** This is the most common graceful
  shutdown bug. The application catches SIGTERM and waits for in-flight
  requests, but the health endpoint still returns 200, so the load balancer
  keeps sending traffic.
- **Returning 503 from the liveness probe.** The liveness probe should always
  return 200 during shutdown. If it returns 503, Kubernetes might restart the
  pod, which defeats the purpose of graceful shutdown.

## Key Takeaway

A graceful shutdown handler is a coordination mechanism between three
components: the signal (SIGTERM from the container runtime), the application
(in-flight request tracking and cleanup), and the infrastructure (load
balancer health checks). All three must work together. The handler catches
the signal, stops new work, drains existing work, and reports its status
through the health endpoint. Missing any one of these causes dropped requests.
