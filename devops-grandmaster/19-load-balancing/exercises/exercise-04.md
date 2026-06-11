# Exercise 04: Compare Algorithms Under Load (Challenge)

## Objective

Measure and compare the performance of Round Robin, Least Connections, and IP
Hash load-balancing algorithms using a real load-testing tool.

## Prerequisites

- Docker and Docker Compose
- A load-testing tool: `hey` (https://github.com/rakyll/hey), `wrk`, or `k6`
- Basic command-line skills

## Background

Theory tells us which algorithm suits which scenario. This exercise makes you
*prove it* by running controlled experiments and collecting data.

## Instructions

### Step 1 -- Create the test environment

Create a project directory:

```
mkdir -p ~/lb-exercise-04 && cd ~/lb-exercise-04
```

Create a `docker-compose.yml` with:

- **Three backend containers** using `hashicorp/http-echo:0.2.3`, each
  responding with its own name.
- **One Nginx load balancer** on port `8080`.

Create an `nginx.conf` that defines `backend_pool` as an upstream.

### Step 2 -- Install a load-testing tool

Install `hey` (recommended):

```bash
go install github.com/rakyll/hey@latest
```

Or use `wrk`:

```bash
sudo apt-get install -y wrk
```

Or use any tool that can report latency percentiles and throughput.

### Step 3 -- Test Round Robin

Ensure the `upstream` block has no special directive (Round Robin is the
default). Start the environment and run:

```bash
# Using hey: 1000 total requests, 50 concurrent
hey -n 1000 -c 50 http://localhost:8080
```

Save the output to `results-round-robin.txt`.

### Step 4 -- Test Least Connections

Update `nginx.conf` to add `least_conn;` inside the `upstream` block. Reload
Nginx:

```bash
docker compose exec load-balancer nginx -s reload
```

Run the same load test. Save the output to `results-least-connections.txt`.

### Step 5 -- Test IP Hash

Update `nginx.conf` to use `ip_hash;` instead of `least_conn`. Reload Nginx
and run the load test. Save the output to `results-ip-hash.txt`.

### Step 6 -- Introduce latency variance

This is the key part. Update your backend containers so that:

- `backend-1` responds immediately (no delay).
- `backend-2` sleeps for 50ms before responding.
- `backend-3` sleeps for 200ms before responding.

To add a sleep, you can switch to a small custom HTTP server. Create a
`Dockerfile` for each backend or use `nginx` with `echo_sleep` (if using the
`ngx_http_echo_module`). Alternatively, write a simple Python or shell HTTP
server that sleeps.

**Simplest approach**: Use `python:3.12-slim` with a one-liner Python HTTP
server:

```dockerfile
FROM python:3.12-slim
CMD ["python", "-m", "http.server", "5678"]
```

Then add a wrapper script that sleeps before responding. Or use the following
one-liner for each backend container:

```yaml
backend-1:
  image: python:3.12-slim
  command: >
    python -c "
    from http.server import HTTPServer, BaseHTTPRequestHandler
    import time, os
    class H(BaseHTTPRequestHandler):
        def do_GET(self):
            time.sleep(float(os.environ.get('DELAY', '0')))
            self.send_response(200)
            self.end_headers()
            self.wfile.write(b'backend-1')
        def log_message(self, *a): pass
    HTTPServer(('0.0.0.0', 5678), H).serve_forever()
    "
  environment:
    DELAY: "0"
```

Repeat for `backend-2` (DELAY=0.05) and `backend-3` (DELAY=0.2).

Re-run all three load tests with the latency variance in place. Save the
results with a `-delayed` suffix.

### Step 7 -- Analyse and report

Create a file `analysis.md` with:

1. A table comparing the three algorithms across:
   - Average latency
   - p99 latency
   - Throughput (requests/second)
   - Distribution of requests across backends (if your tool reports this)
2. Which algorithm performed best *without* latency variance? Why?
3. Which algorithm performed best *with* latency variance? Why?
4. Which algorithm would you recommend for a production service where backends
   have heterogeneous response times?

## Success Criteria

- [ ] All six load-test runs (3 algorithms x 2 scenarios) completed and results
      saved.
- [ ] `analysis.md` contains a comparison table with real numbers.
- [ ] The analysis correctly explains *why* each algorithm behaved as it did.
- [ ] The final recommendation is justified by the data.

## Hints

<details>
<summary>Hint 1 -- Why latency variance matters</summary>

With identical backends, all algorithms perform similarly. When backends have
different response times, Least Connections naturally routes more requests to
faster backends because they free up connections sooner. Round Robin ignores
response time entirely.

</details>

<details>
<summary>Hint 2 -- Getting per-backend distribution</summary>

Since each backend returns a unique text response, you can count occurrences:

```bash
hey -n 1000 -c 50 http://localhost:8080 2>&1 | grep -c "backend-1"
```

Or pipe all responses to a file and count with `sort | uniq -c`.

</details>

<details>
<summary>Hint 3 -- Interpreting p99 latency</summary>

p99 latency means "99% of requests were faster than this value." A high p99
relative to the average indicates tail-latency problems -- a few slow requests
ruin the experience for some users. Least Connections typically has a lower p99
when backends vary in speed.

</details>
