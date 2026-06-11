# Exercise 01: Baseline Metrics Collection

**Type:** Conceptual
**Time:** 15 minutes
**Difficulty:** Easy

## Objective
Understand why baseline metrics are essential for capacity planning, identify the key metrics to collect, and write PromQL queries to extract them from a Prometheus monitoring system.

## Scenario
Your team needs to plan capacity for a product launch next month. The first question from leadership is: "How much traffic can our system handle today?" Nobody knows. The team has Prometheus and Grafana but has never used them for capacity planning. You need to establish baselines.

## Tasks

### Part A: Identify Key Baseline Metrics
List the eight most important metrics for capacity planning. For each metric, explain:
- What it measures
- Why it matters for capacity planning
- What "good" and "bad" values look like

<details>
<summary>Hint</summary>
Think about the four main resource categories: CPU, memory, network, and I/O. Within each category, identify the metrics that indicate how close you are to the limit. Also consider application-level metrics like request rate, latency, and error rate.
</details>

### Part B: Write PromQL Queries
Write a PromQL query for each of the following metrics:
1. Total HTTP requests per second across all pods
2. 95th percentile request latency
3. Average CPU utilization as a percentage
4. Average memory utilization as a percentage of the limit
5. Database connection count
6. Error rate (5xx responses as a fraction of total requests)

<details>
<summary>Hint</summary>
Use `rate()` for per-second rates, `histogram_quantile()` for percentiles, and division for percentages. The `container_cpu_usage_seconds_total` metric tracks CPU usage, and `container_spec_memory_limit_bytes` tracks memory limits.
</details>

### Part C: Determine Collection Duration and Frequency
Explain:
1. How long should you collect baseline metrics to capture normal variation? Why?
2. What sampling frequency should you use? Why?
3. What patterns should you look for in the collected data (daily cycles, weekly cycles, etc.)?

<details>
<summary>Hint</summary>
Traffic patterns vary by time of day (peak hours vs off-peak) and day of week (weekday vs weekend). Collecting for at least one full week captures both patterns. Sampling every 5 minutes gives 2,016 data points per week -- enough for statistical analysis without excessive storage.
</details>

## Success Criteria
- [ ] You can identify eight key metrics for capacity planning with good/bad thresholds.
- [ ] You can write correct PromQL queries for all six metric types.
- [ ] You can explain why one week of data is the minimum for reliable baselines.
- [ ] You can describe daily and weekly traffic patterns and why they matter.
- [ ] You can explain the difference between average and peak utilization.

## What You Should Understand After This Exercise
Capacity planning starts with measurement, not guessing. You need to know your current utilization across all resource dimensions (CPU, memory, network, I/O, connections) before you can forecast when you will hit limits. Baselines must capture normal variation -- daily peaks, weekly cycles, and occasional spikes -- to be useful for planning.
