# Exercise 05: Horizontal Scaling with Docker Compose

**Type:** Integration
**Time:** 45 minutes
**Difficulty:** Hard

## Objective

Implement a horizontally scaled application using Docker Compose with an Nginx load balancer, health checks, and the ability to verify that traffic is distributed across replicas. This exercise integrates concepts from [Module 07: Docker Compose](../../07-docker-compose/) with horizontal scaling from this module.

## Scenario

You have a stateless API service that needs to handle increased traffic. You will deploy it as multiple replicas behind an Nginx load balancer using Docker Compose. Each replica must report which instance handled the request so you can verify load balancing is working.

## Tasks

### Part A: Create the Application

Create a simple Python Flask application that returns the container's hostname on each request. This lets you verify which replica served each request.

Create a file called `app.py`:

```python
from flask import Flask, jsonify
import os
import socket
import time

app = Flask(__name__)

@app.route('/')
def index():
    return jsonify({
        "message": "Hello from horizontally scaled app",
        "instance": socket.gethostname(),
        "timestamp": time.time()
    })

@app.route('/health')
def health():
    return jsonify({"status": "healthy", "instance": socket.gethostname()})

@app.route('/heavy')
def heavy():
    """CPU-intensive endpoint to demonstrate scaling needs."""
    start = time.time()
    count = 0
    for num in range(2, 5000):
        if all(num % i != 0 for i in range(2, int(num**0.5) + 1)):
            count += 1
    elapsed = time.time() - start
    return jsonify({
        "primes": count,
        "elapsed_seconds": round(elapsed, 2),
        "instance": socket.gethostname()
    })

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5000)
```

Create a `Dockerfile` for the application.

<details>
<summary>Hint</summary>

Use `python:3.11-slim` as the base image. Install Flask with pip. Copy the app file and expose port 5000.

</details>

### Part B: Configure Nginx as a Load Balancer

Create an `nginx.conf` that:
1. Defines an upstream block with the app service.
2. Proxies all requests to the upstream.
3. Passes the `Host` and `X-Real-IP` headers.
4. Includes a health check endpoint that bypasses the upstream (Nginx responds directly to `/nginx-health`).

<details>
<summary>Hint</summary>

In the `http` block, define `upstream app_servers { server app:5000; }`. Docker Compose DNS resolves `app` to all replicas automatically. Use `proxy_set_header` to forward headers. Create a separate `location /nginx-health` that returns a 200 directly.

</details>

### Part C: Write the Docker Compose File

Create a `docker-compose.yml` with:
1. The `app` service with 4 replicas and resource limits (0.25 CPU, 128M memory per replica).
2. The `nginx` service with a port mapping to host port 8080.
3. A health check on the app service using the `/health` endpoint.
4. A health check on the nginx service using the `/nginx-health` endpoint.
5. A custom bridge network that both services share.

<details>
<summary>Hint 1</summary>

Use `deploy.replicas: 4` under the app service. Use `deploy.resources.limits` for CPU and memory. The nginx service needs a volume mount for `nginx.conf`.

</details>

<details>
<summary>Hint 2</summary>

Health checks use the `healthcheck` key with `test`, `interval`, `timeout`, and `retries`. For the app service: `test: ["CMD", "curl", "-f", "http://localhost:5000/health"]`.

</details>

### Part D: Verify Horizontal Scaling

After running `docker compose up -d`, perform these verification steps:

1. Send 12 requests to `http://localhost:8080/` and record which instance handles each request.
2. Verify that multiple different instances responded (not just one).
3. Send 4 concurrent requests to `/heavy` and measure the total time.
4. Compare: what would happen if you sent 4 concurrent requests to a *single* instance instead?

Write your findings as a table showing request number, instance name, and response time.

<details>
<summary>Hint</summary>

Use a bash loop: `for i in $(seq 1 12); do curl -s http://localhost:8080/ | python3 -m json.tool; done`. Look at the "instance" field in each response. For concurrent requests, use `&` to background them and `wait` to measure total time.

</details>

### Part E: Scale Up and Down

Demonstrate dynamic scaling:
1. Scale the app service to 8 replicas: `docker compose up -d --scale app=8`
2. Send 16 requests and verify that more instances are now responding.
3. Scale back down to 2 replicas.
4. Send 10 requests and verify that only 2 instances respond.
5. Observe: did any requests fail during scale-down? Why or why not?

<details>
<summary>Hint</summary>

Docker Compose handles the networking automatically -- new replicas are added to the DNS resolution for the service name. During scale-down, existing connections are allowed to complete before the container stops.

</details>

## Success Criteria

- [ ] Your Dockerfile builds the Flask application successfully.
- [ ] Nginx distributes requests across multiple app replicas.
- [ ] The verification step shows at least 2 different instance hostnames serving requests.
- [ ] Concurrent requests to `/heavy` complete faster with 4 replicas than with 1.
- [ ] You can scale up to 8 replicas and back down to 2 without errors.
- [ ] Your `docker-compose.yml` includes health checks for both services.

## What You Should Understand After This Exercise

Horizontal scaling with Docker Compose is the simplest way to run multiple replicas of a stateless service. The key pieces are: (1) `deploy.replicas` to run N identical containers, (2) an Nginx load balancer that distributes traffic via Docker DNS, (3) health checks to ensure only healthy replicas receive traffic, and (4) resource limits to prevent one replica from consuming all host resources. The application must be stateless -- each request is independent, and no state is stored in the container. This pattern scales to Kubernetes with minimal changes: replace `deploy.replicas` with a Deployment and add a Service for load balancing.
