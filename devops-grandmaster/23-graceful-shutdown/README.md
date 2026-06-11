# Module 23: Graceful Shutdown

> **Previous module (22):** Uptime commitment — SLA, SLO, SLI, and error budgets.
> **Limitation:** When you restart a container to deploy a new version, in-flight requests are dropped. Users see errors. This eats your error budget.
> **This module:** Graceful shutdown, connection draining, and zero-downtime restarts.

---

## 1. The Problem

You deploy version 2.0. Docker stops the old containers and starts new ones. But 50 users had active requests in progress when the container received the stop signal.

What happens to those requests?

```
User A: uploading a 10MB file... 75% complete...
User B: waiting for a complex database query...
User C: mid-transaction, money being transferred...

docker stop app_container

User A: CONNECTION RESET
User B: CONNECTION RESET
User C: CONNECTION RESET (money? who knows.)
```

Three users got errors. Three requests counted against your error budget. User C might have lost money or created a duplicate transaction.

This is not an edge case. Every deployment triggers this unless you handle it properly.

---

## 2. The Naive Way — Just Kill It

```bash
docker kill app_container
# or
docker stop --time 0 app_container
```

**What happens:**
1. Docker sends SIGKILL immediately
2. The process dies instantly
3. No cleanup, no finish-what-you-are-doing
4. All TCP connections are severed
5. All in-flight requests are lost
6. All in-memory state is lost

**Why it fails:**

- Every deployment causes user-facing errors
- Partial writes to databases (corrupted data)
- File uploads left incomplete
- Workers mid-processing lose their state
- Error budget is consumed by deployments, not real incidents

---

## 3. The Right Way — Graceful Shutdown

When Docker stops a container, it sends SIGTERM first. The application should catch this signal and clean up.

### The Docker Stop Sequence

```
docker stop app_container

1. Docker sends SIGTERM to the main process (PID 1)
2. Process has time to clean up (default: 10 seconds)
3. If process is still running after timeout → SIGKILL
4. Process is forcefully terminated
```

```
Timeline:
  0s    SIGTERM received
  0-10s Graceful shutdown period
        - Stop accepting new connections
        - Finish processing in-flight requests
        - Close database connections
        - Flush logs
        - Deregister from load balancer
  10s   If still running → SIGKILL (forced death)
```

### Graceful Shutdown in Python (Flask/Gunicorn)

```python
import signal
import sys
from flask import Flask

app = Flask(__name__)

# Track in-flight requests
in_flight = 0
shutting_down = False

@app.before_request
def before_request():
    global in_flight
    if shutting_down:
        from flask import jsonify
        return jsonify({"error": "Service is shutting down"}), 503
    in_flight += 1

@app.after_request
def after_request(response):
    global in_flight
    in_flight -= 1
    return response

def graceful_shutdown(signum, frame):
    global shutting_down
    shutting_down = True
    print(f"Received signal {signum}. Shutting down gracefully...")
    print(f"In-flight requests: {in_flight}")

    # Wait for in-flight requests to complete
    import time
    timeout = 30  # seconds
    start = time.time()
    while in_flight > 0 and (time.time() - start) < timeout:
        print(f"Waiting for {in_flight} requests to complete...")
        time.sleep(1)

    print("Shutdown complete.")
    sys.exit(0)

signal.signal(signal.SIGTERM, graceful_shutdown)
signal.signal(signal.SIGINT, graceful_shutdown)

@app.route('/health')
def health():
    return {"status": "ok"}

@app.route('/slow')
def slow():
    import time
    time.sleep(5)  # Simulate slow request
    return {"result": "done"}

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5000)
```

### Graceful Shutdown in Node.js (Express)

```javascript
const express = require('express');
const app = express();

let inFlight = 0;
let shuttingDown = false;

app.use((req, res, next) => {
    if (shuttingDown) {
        res.set('Connection', 'close');
        return res.status(503).json({ error: 'Service is shutting down' });
    }
    inFlight++;
    res.on('finish', () => { inFlight--; });
    next();
});

app.get('/slow', async (req, res) => {
    await new Promise(resolve => setTimeout(resolve, 5000));
    res.json({ result: 'done' });
});

app.get('/health', (req, res) => {
    res.json({ status: shuttingDown ? 'shutting_down' : 'ok' });
});

const server = app.listen(5000, () => {
    console.log('Server running on port 5000');
});

function gracefulShutdown(signal) {
    console.log(`Received ${signal}. Shutting down gracefully...`);
    shuttingDown = true;

    // Stop accepting new connections
    server.close(() => {
        console.log('All connections closed. Exiting.');
        process.exit(0);
    });

    // Force exit after timeout
    setTimeout(() => {
        console.log('Forcing exit after timeout.');
        process.exit(1);
    }, 30000);
}

process.on('SIGTERM', () => gracefulShutdown('SIGTERM'));
process.on('SIGINT', () => gracefulShutdown('SIGINT'));
```

### Graceful Shutdown in Go

```go
package main

import (
    "context"
    "fmt"
    "net/http"
    "os"
    "os/signal"
    "sync/atomic"
    "syscall"
    "time"
)

var (
    inFlight    int64
    shuttingDown int32
)

func main() {
    mux := http.NewServeMux()

    mux.HandleFunc("/slow", func(w http.ResponseWriter, r *http.Request) {
        if atomic.LoadInt32(&shuttingDown) == 1 {
            http.Error(w, `{"error":"shutting down"}`, http.StatusServiceUnavailable)
            return
        }
        atomic.AddInt64(&inFlight, 1)
        defer atomic.AddInt64(&inFlight, -1)

        time.Sleep(5 * time.Second)
        fmt.Fprintf(w, `{"result":"done"}`)
    })

    mux.HandleFunc("/health", func(w http.ResponseWriter, r *http.Request) {
        fmt.Fprintf(w, `{"status":"ok"}`)
    })

    server := &http.Server{
        Addr:    ":5000",
        Handler: mux,
    }

    // Start server in goroutine
    go func() {
        fmt.Println("Server running on port 5000")
        if err := server.ListenAndServe(); err != http.ErrServerClosed {
            fmt.Printf("Server error: %v\n", err)
        }
    }()

    // Wait for SIGTERM
    quit := make(chan os.Signal, 1)
    signal.Notify(quit, syscall.SIGTERM, syscall.SIGINT)
    <-quit

    fmt.Println("Received shutdown signal. Shutting down gracefully...")
    atomic.StoreInt32(&shuttingDown, 1)

    // Wait for in-flight requests
    timeout := 30 * time.Second
    ctx, cancel := context.WithTimeout(context.Background(), timeout)
    defer cancel()

    // server.Shutdown waits for in-flight requests to complete
    if err := server.Shutdown(ctx); err != nil {
        fmt.Printf("Shutdown error: %v\n", err)
    }

    fmt.Printf("Shutdown complete. %d in-flight requests were handled.\n",
        atomic.LoadInt64(&inFlight))
}
```

---

## 4. The Production Way

### The Full Zero-Downtime Deployment Sequence

```
Step 1: New container starts
┌──────────────┐  ┌──────────────┐
│  Old App (v1)│  │  New App (v2)│  ← Starting up
│  (serving)   │  │  (not ready) │
└──────────────┘  └──────────────┘

Step 2: New container passes health check
┌──────────────┐  ┌──────────────┐
│  Old App (v1)│  │  New App (v2)│  ← Now healthy
│  (serving)   │  │  (ready)     │
└──────────────┘  └──────────────┘

Step 3: Load balancer adds new container, starts draining old
┌──────────────┐  ┌──────────────┐
│  Old App (v1)│  │  New App (v2)│
│  (draining)  │  │  (serving)   │  ← LB routes new traffic here
│  3 requests  │  │              │
└──────────────┘  └──────────────┘

Step 4: Old container finishes in-flight requests
┌──────────────┐  ┌──────────────┐
│  Old App (v1)│  │  New App (v2)│
│  (done)      │  │  (serving)   │
└──────────────┘  └──────────────┘

Step 5: Old container removed
                              ┌──────────────┐
                              │  New App (v2)│
                              │  (serving)   │
                              └──────────────┘
```

### Deregistering from Load Balancer Before Shutdown

The application should stop receiving new requests before it starts shutting down.

**Pre-stop hook (Kubernetes style):**
```yaml
# In Kubernetes Pod spec
containers:
  - name: app
    image: myapp:2.0
    lifecycle:
      preStop:
        exec:
          # Deregister from LB, wait for connections to drain
          command: ["/bin/sh", "-c", "sleep 5"]
    terminationGracePeriodSeconds: 30
```

**Nginx upstream draining:**
```nginx
upstream app_backend {
    server app1:5000;
    server app2:5000;
    server app3:5000 down;  # Marked as draining
}
```

### Connection Draining

Connection draining means: stop sending new requests to this instance, but let existing requests finish.

```
Connection draining timeline:

0s  - Mark instance as "draining" in load balancer
      (no new requests will be sent to this instance)

0-30s - Existing requests complete naturally
        New requests go to other healthy instances

30s - All connections drained
      Send SIGTERM to application
      Application does final cleanup
      Container stops
```

**AWS ALB deregistration delay:**
```bash
# Configure deregistration delay (default: 300 seconds)
aws elbv2 modify-target-group-attributes \
  --target-group-arn arn:aws:elasticloadbalancing:... \
  --attributes Key=deregistration_delay.timeout_seconds,Value=30
```

**Nginx active health checks with drain:**
```nginx
upstream app_backend {
    server app1:5000 max_fails=1 fail_timeout=10s;
    server app2:5000 max_fails=1 fail_timeout=10s;
    server app3:5000 max_fails=1 fail_timeout=10s;
}
```

### Shutdown Timeout Configuration

```bash
# Docker: set stop timeout (default is 10 seconds)
docker stop --time 30 app_container

# docker-compose.yml
services:
  app:
    image: myapp:latest
    stop_grace_period: 30s  # Wait 30s before SIGKILL

# Kubernetes
spec:
  terminationGracePeriodSeconds: 60  # Wait 60s before SIGKILL
```

**Choosing the right timeout:**
```
Too short (5s):
  - In-flight requests are killed before completing
  - Defeats the purpose of graceful shutdown

Too long (300s):
  - Deployments take forever
  - Old instances stick around too long
  - During an incident, recovery is slow

Right (30-60s):
  - Enough time for most requests to complete
  - Fast enough for reasonable deployment speed
  - Match it to your p99 latency + buffer
```

### Health Check During Shutdown

The application should report unhealthy during shutdown so the load balancer stops sending traffic.

```python
@app.route('/health')
def health():
    if shutting_down:
        return jsonify({"status": "shutting_down"}), 503
    return jsonify({"status": "ok"}), 200

@app.route('/ready')
def ready():
    """Readiness probe: can this instance handle traffic?"""
    if shutting_down:
        return jsonify({"status": "not_ready"}), 503
    if not database_connected:
        return jsonify({"status": "not_ready"}), 503
    return jsonify({"status": "ready"}), 200
```

**Difference between liveness and readiness:**
```
Liveness:  "Is the process alive?"
           If failed → restart the container
           Always returns 200 during normal operation

Readiness: "Can this instance handle traffic?"
           If failed → remove from load balancer
           Returns 503 during startup and shutdown
```

---

## 5. Hands-On Lab

### Lab: Zero-Downtime Restart

**Step 1: Create an application with graceful shutdown**

**app.py:**
```python
from flask import Flask, jsonify
import signal
import sys
import time
import threading

app = Flask(__name__)

in_flight = 0
shutting_down = False
lock = threading.Lock()

@app.before_request
def before_request():
    global in_flight
    with lock:
        if shutting_down:
            return jsonify({"error": "Service shutting down"}), 503
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
    """Simulates a slow request (5 seconds)."""
    time.sleep(5)
    return jsonify({"result": "completed", "in_flight": in_flight})

@app.route('/fast')
def fast():
    return jsonify({"result": "done"})

def graceful_shutdown(signum, frame):
    global shutting_down
    print(f"\n[SHUTDOWN] Signal {signum} received. Starting graceful shutdown...")
    shutting_down = True

    print(f"[SHUTDOWN] {in_flight} in-flight requests. Waiting...")
    timeout = 30
    start = time.time()
    while in_flight > 0 and (time.time() - start) < timeout:
        time.sleep(1)
        print(f"[SHUTDOWN] Still waiting: {in_flight} requests remaining")

    print(f"[SHUTDOWN] Complete. Exiting.")
    sys.exit(0)

signal.signal(signal.SIGTERM, graceful_shutdown)

if __name__ == '__main__':
    print("[STARTUP] App running on port 5000")
    app.run(host='0.0.0.0', port=5000, threaded=True)
```

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
    stop_grace_period: 30s
```

**Step 2: Test graceful shutdown**

```bash
# Terminal 1: Start the app
docker compose up

# Terminal 2: Send a slow request
curl http://localhost:5000/slow &

# Terminal 2: Immediately stop the container (gracefully)
docker compose stop app

# Watch Terminal 1: You should see:
# [SHUTDOWN] Signal 15 received. Starting graceful shutdown...
# [SHUTDOWN] 1 in-flight requests. Waiting...
# 172.18.0.1 - - "GET /slow HTTP/1.1" 200 -
# [SHUTDOWN] Complete. Exiting.

# The slow request COMPLETED before the container stopped
```

**Step 3: Test without graceful shutdown**

```bash
# Start the app
docker compose up -d

# Send a slow request
curl http://localhost:5000/slow &
SLOW_PID=$!

# Kill it immediately
docker kill $(docker compose ps -q app)

# The curl request gets: curl: (56) Recv failure: Connection reset by peer
```

**Step 4: Simulate zero-downtime deployment**

**deploy.sh:**
```bash
#!/bin/bash
set -e

echo "=== Zero-Downtime Deployment ==="

# Step 1: Start new instance alongside old
echo "[1/5] Starting new instance..."
docker compose up -d --scale app=2 --no-recreate

sleep 3

# Step 2: Check new instance is healthy
echo "[2/5] Checking new instance health..."
# (In production, the load balancer does this automatically)

# Step 3: Stop old instance (gracefully)
echo "[3/5] Stopping old instance..."
OLD_CONTAINER=$(docker compose ps -q app | head -1)
docker stop --time 30 $OLD_CONTAINER

# Step 4: Verify
echo "[4/5] Verifying deployment..."
curl -s http://localhost:5000/health

echo "[5/5] Deployment complete!"
```

---

## 6. Limitation

Graceful shutdown handles restarts on a single machine. Your container stops cleanly, finishes in-flight requests, and the new version starts up. Zero downtime.

But all of this still runs on one machine. You have one Docker host, one load balancer, one set of containers. If the machine itself fails — hardware fault, kernel panic, network outage — everything goes down.

You need to run containers across multiple machines. You need automatic scheduling, self-healing, service discovery, and configuration management across a cluster of machines.

This is the boundary between running containers and orchestrating them.

---

## 7. Next Topic

**Phase 4: Kubernetes** — True multi-machine orchestration. Module 24 covers when to use Kubernetes and when it is overkill. [Go to Module 24: When to Use Kubernetes →](../24-when-to-use-kubernetes/README.md)
