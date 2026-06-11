# Exercise 02: Progressive Load Testing

**Type:** Guided
**Time:** 30 minutes
**Difficulty:** Easy-Medium

## Objective
Design and run progressive load tests that systematically increase traffic to find the system's sustainable capacity and breaking point.

## Scenario
You have a web application running on Kubernetes with 3 replicas. Normal traffic is 500 RPS. You need to determine:
1. Can the system handle 2x traffic (1,000 RPS)?
2. Can the system handle 5x traffic (2,500 RPS)?
3. Can the system handle 10x traffic (5,000 RPS)?
4. What is the breaking point (the RPS at which error rate exceeds 5% or p95 latency exceeds 2 seconds)?

## Tasks

### Part A: Design the Load Test Stages
Write a k6 load test configuration with stages that progressively increase traffic. The test should:
1. Start with a warmup phase (100 VUs for 1 minute)
2. Test at 1x traffic (100 VUs for 3 minutes)
3. Test at 2x traffic (200 VUs for 3 minutes)
4. Test at 5x traffic (500 VUs for 3 minutes)
5. Test at 10x traffic (1000 VUs for 3 minutes)
6. Ramp down to 0 VUs over 1 minute

<details>
<summary>Hint</summary>
Use k6 `stages` array. Each stage has `duration` and `target` (number of virtual users). The warmup phase ensures the system is fully initialized before measurement begins.
</details>

### Part B: Define Thresholds for Acceptable Performance
Write k6 threshold definitions that enforce:
1. Error rate must be below 5% at each traffic level
2. 95th percentile latency must be below 2,000ms
3. 99th percentile latency must be below 5,000ms

<details>
<summary>Hint</summary>
Use the `thresholds` object in k6 options. The key is the metric name (e.g., `http_req_failed`, `http_req_duration`), and the value is an array of threshold expressions like `['p(95)<2000']`.
</details>

### Part C: Create a Realistic Traffic Mix
Write a k6 `default` function that simulates realistic user behavior:
1. 60% of requests: GET /api/products (browse products)
2. 20% of requests: GET /api/products/{id} (view product detail)
3. 10% of requests: POST /api/cart (add to cart)
4. 10% of requests: POST /api/orders (checkout)

Each request should have appropriate headers and payloads.

<details>
<summary>Hint</summary>
Use `Math.random()` to select the endpoint based on weights. Create a cumulative probability distribution: if random < 0.6, browse; if < 0.8, detail; if < 0.9, cart; else checkout.
</details>

### Part D: Write the Results Analysis Script
Write a bash script that:
1. Runs the k6 load test and exports results to JSON
2. Parses the JSON to extract RPS, p95 latency, and error rate at each traffic level
3. Determines the sustainable capacity (highest traffic level where error rate < 5% and p95 < 2s)
4. Prints a summary table

<details>
<summary>Hint</summary>
Use `k6 run --summary-export=results.json` to export results. Use `jq` to parse the JSON: `.metrics.http_reqs.values.rate` for RPS, `.metrics.http_req_duration.values["p(95)"]` for p95, `.metrics.http_req_failed.values.rate` for error rate.
</details>

## Success Criteria
- [ ] The k6 configuration has stages for 1x, 2x, 5x, and 10x traffic.
- [ ] Thresholds enforce error rate and latency limits at each level.
- [ ] The traffic mix simulates realistic user behavior with weighted endpoints.
- [ ] The analysis script extracts and compares metrics across traffic levels.
- [ ] The script identifies the sustainable capacity and breaking point.

## What You Should Understand After This Exercise
Progressive load testing reveals how your system behaves under increasing load. The sustainable capacity is the highest traffic level where performance remains acceptable. The breaking point is where the system fails. The gap between current traffic and sustainable capacity is your headroom. This data drives all capacity planning decisions.
