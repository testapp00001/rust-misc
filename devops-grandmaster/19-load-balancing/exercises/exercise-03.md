# Exercise 03: Health-Check-Aware Load Balancing (Independent)

## Objective

Extend a load-balancer configuration so that unhealthy backends are
automatically removed from the pool and re-added when they recover.

## Background

In production, backends can crash, hang, or become overloaded. A load balancer
that blindly forwards traffic to dead servers degrades user experience. Active
health checks let the load balancer probe backends and route traffic only to
healthy ones.

## Instructions

### Step 1 -- Set up the base environment

Create a new project directory:

```
mkdir -p ~/lb-exercise-03 && cd ~/lb-exercise-03
```

Copy the `docker-compose.yml` and `nginx.conf` from Exercise 02 (or recreate
them). Make sure the three backends and the Nginx load balancer are running.

### Step 2 -- Add a health-check endpoint

The `hashicorp/http-echo` image always returns 200 OK. To simulate a failing
backend, you will use a **different approach**: create a small shell script
that the backend will serve. For this exercise, simply use Nginx's built-in
passive health check mechanism.

Update your `nginx.conf` so that each server in the `upstream` block has:

```nginx
server backend-1:5678 max_fails=2 fail_timeout=15s;
```

This tells Nginx: "If a backend fails 2 times within 15 seconds, mark it as
down for 15 seconds."

### Step 3 -- Test normal operation

```bash
docker compose up -d
for i in $(seq 1 12); do curl -s http://localhost:8080; done
```

You should see requests distributed across all three backends.

### Step 4 -- Simulate a backend failure

Stop one of the backend containers:

```bash
docker compose stop backend-2
```

Immediately send a burst of requests:

```bash
for i in $(seq 1 12); do curl -s http://localhost:8080; done
```

Observe that requests no longer reach `backend-2`. Nginx marks it as
unavailable after the configured number of failures.

### Step 5 -- Verify recovery

Restart the failed backend:

```bash
docker compose start backend-2
```

Wait for the `fail_timeout` period to elapse, then send another burst of
requests. Verify that `backend-2` is receiving traffic again.

### Step 6 -- Enhance with active health checks (optional)

If you are using Nginx Plus (or want to research the open-source alternative),
configure an active health check:

```nginx
server backend-1:5678 check;
```

For open-source Nginx, document in a short paragraph how you would implement
active health checks using a third-party module or a sidecar container.

## Deliverables

Write a short report (in a file called `report.md` or in your own notes) that
includes:

1. The full `nginx.conf` you used.
2. The output showing traffic shifting away from the stopped backend.
3. The output showing traffic returning to the recovered backend.
4. A 2-3 sentence explanation of the difference between passive and active
   health checks.

## Success Criteria

- [ ] `max_fails` and `fail_timeout` are configured on each upstream server.
- [ ] Stopping `backend-2` causes Nginx to stop sending traffic to it within a
      few requests.
- [ ] Restarting `backend-2` causes Nginx to resume sending traffic to it
      after the `fail_timeout` window.
- [ ] The report clearly distinguishes passive from active health checks.

## Hints

<details>
<summary>Hint 1 -- Passive health checks</summary>

Passive health checks (also called "circuit breaking" in Nginx) happen when
Nginx detects a failure on a real client request. It does not send separate
probe requests. The `max_fails` and `fail_timeout` parameters control when a
server is marked unavailable.

</details>

<details>
<summary>Hint 2 -- Active health checks</summary>

Active health checks send periodic HTTP requests to a dedicated endpoint (e.g.,
`/healthz`) on each backend, independent of client traffic. This requires Nginx
Plus or the `nginx_upstream_check_module`. The response status code determines
whether the backend is healthy.

</details>

<details>
<summary>Hint 3 -- Failures that Nginx counts</summary>

By default, Nginx counts errors (connection refused, timeout) and HTTP 502/503
responses as failures. You can change this with the `proxy_next_upstream`
directive.

</details>
