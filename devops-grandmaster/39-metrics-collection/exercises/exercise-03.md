# Exercise 03: Write PromQL Queries for Common Scenarios

**Type:** Independent
**Estimated time:** 40 minutes

## Objective

Write PromQL queries that answer real-world operational questions. You have
the Prometheus stack from Exercise 02 running. This exercise tests your
ability to translate operational questions into correct PromQL expressions.

## Prerequisites

- The Prometheus + Grafana + node_exporter stack from Exercise 02 is running
- Prometheus is accessible at `http://localhost:9090`
- Basic understanding of PromQL syntax from the module README

## Instructions

For each scenario below, write the PromQL query and test it in the
Prometheus UI. Record your query and the result you observed.

### Part A -- Infrastructure Queries

1. **CPU utilization percentage** per instance over the last 5 minutes.
   (Hint: subtract idle from 100.)

2. **Memory utilization percentage** per instance.
   (Hint: use `node_memory_MemAvailable_bytes` and
   `node_memory_MemTotal_bytes`.)

3. **Disk utilization percentage** for the root filesystem.
   (Hint: filter by `mountpoint="/"` and `fstype!="tmpfs"`.)

4. **System load average** over 1 minute, normalized by the number of CPUs.
   A value above 1.0 means the system is overloaded.
   (Hint: `node_load1` divided by the count of CPU cores.)

5. **Network bandwidth** in megabytes per second received on `eth0`.
   (Hint: use `rate()` and convert bytes to megabytes.)

### Part B -- Rate and Increase Queries

6. **Per-second rate of CPU seconds** consumed in system mode over the last
   5 minutes.

7. **Total bytes received** by the network interface over the last 1 hour.
   (Hint: use `increase()` not `rate()`.)

8. **Disk I/O utilization** -- what percentage of time the disk has been
   busy doing I/O over the last 5 minutes.
   (Hint: `rate(node_disk_io_time_seconds_total[5m]) * 100`.)

### Part C -- Aggregation Queries

9. **Average CPU utilization** across all instances (if you have only one
   instance, this is the same as the per-instance value).

10. **Top filesystem by disk usage** -- which mount point has the highest
    utilization percentage.

11. **Total network bytes received** across all network interfaces
    (aggregate all devices).

### Part D -- Alerting Expressions

For each scenario, write a PromQL expression that evaluates to `1` when
the condition is true (suitable for use in an alerting rule).

12. **CPU usage above 80%** for the last 5 minutes.

13. **Memory usage above 90%** for the last 5 minutes.

14. **Disk usage above 85%** for the root filesystem.

15. **A target is down** -- the `up` metric equals 0.

### Part E -- Bonus: Time-based Queries

16. **Compare current memory usage to 1 hour ago.** Is memory usage higher
    or lower than an hour ago?

17. **Predict disk full time.** Using `predict_linear()` on filesystem
    available bytes, estimate how many seconds until the disk is full.
    (Hint: `predict_linear(node_filesystem_avail_bytes{mountpoint="/"}[1h], 24*3600)`.)

## Success Criteria

- [ ] All queries return valid results in the Prometheus UI.
- [ ] CPU, memory, and disk utilization percentages are between 0 and 100.
- [ ] Rate queries use the `rate()` or `increase()` function with a time
      range vector.
- [ ] Aggregation queries use `sum`, `avg`, `topk`, or similar functions.
- [ ] Alerting expressions use comparison operators and evaluate to 1 or 0.
- [ ] You can explain the difference between `rate()` and `increase()`.
- [ ] You can explain the difference between `rate()` and `irate()`.

## Hints

<details>
<summary>Hint 1 -- CPU utilization formula</summary>
The CPU idle percentage is `avg(rate(node_cpu_seconds_total{mode="idle"}[5m]))`.
Subtract from 100 to get utilization. Use `by (instance)` if you want per
instance breakdown.
</details>

<details>
<summary>Hint 2 -- Memory percentage</summary>
Memory used percentage: `(1 - node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes) * 100`.
This gives the fraction of memory that is NOT available, expressed as a
percentage.
</details>

<details>
<summary>Hint 3 -- rate vs increase</summary>
`rate()` returns per-second rate. `increase()` returns total increase over
the time window. For "how many bytes in the last hour," use `increase(...[1h])`.
</details>

<details>
<summary>Hint 4 -- rate vs irate</summary>
`irate()` uses only the last two data points in the range, making it more
sensitive to spikes. `rate()` averages over the entire window, making it
better for dashboards and alerts.
</details>

<details>
<summary>Hint 5 -- Boolean alerts</summary>
To turn a comparison into 1/0 for alerting: `(your_expression > threshold) == 1`
or just use the comparison directly in the alert `expr` field. Prometheus
treats any result as "firing" if the expression returns any series.
</details>

<details>
<summary>Hint 6 -- Filtering tmpfs</summary>
Always add `fstype!="tmpfs"` when querying filesystem metrics. tmpfs is a
memory-backed filesystem and its size is misleading in disk usage dashboards.
</details>
