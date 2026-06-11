# Exercise 01: Container Resource Metrics and cgroups

**Type:** Conceptual
**Difficulty:** Beginner
**Time:** 20-30 minutes

---

## Objective

Understand how Linux cgroups expose container resource metrics, identify the key metric families for CPU, memory, disk I/O, and network, and apply the USE method to reason about container resource monitoring.

---

## Background

Containers are not magic. They are regular Linux processes constrained by **cgroups** (control groups). Every resource metric you see in `docker stats`, cAdvisor, or Prometheus originates from cgroup files under `/sys/fs/cgroup/`. Understanding this foundation is critical because it explains what is measurable, what is not, and why some metrics behave the way they do.

---

## Part A: Explore cgroup files

On a Linux host with Docker installed, run a container and examine its cgroup resource files.

1. Start a container with explicit resource limits:

```bash
docker run -d --name cgroup-lab \
  --memory=256m \
  --cpus=0.5 \
  --memory-swap=256m \
  nginx:alpine
```

2. Find the container's cgroup directory. The path depends on your cgroup version:

```bash
# Find the container's full ID
CONTAINER_ID=$(docker inspect --format '{{.Id}}' cgroup-lab)
echo $CONTAINER_ID

# cgroup v1 (common on older systems)
ls /sys/fs/cgroup/memory/docker/${CONTAINER_ID}/

# cgroup v2 (common on newer systems)
ls /sys/fs/cgroup/system.slice/docker-${CONTAINER_ID}.scope/
```

3. Read the memory limit that Docker set:

```bash
# cgroup v1
cat /sys/fs/cgroup/memory/docker/${CONTAINER_ID}/memory.limit_in_bytes

# cgroup v2
cat /sys/fs/cgroup/system.slice/docker-${CONTAINER_ID}.scope/memory.max
```

4. Read the current memory usage:

```bash
# cgroup v1
cat /sys/fs/cgroup/memory/docker/${CONTAINER_ID}/memory.usage_in_bytes

# cgroup v2
cat /sys/fs/cgroup/system.slice/docker-${CONTAINER_ID}.scope/memory.current
```

### Questions

- What value do you see for the memory limit? Convert it to megabytes. Does it match the `256m` you specified?
- What value do you see for current memory usage? Why is it not zero (nginx is barely doing anything)?

---

## Part B: Map cgroup files to Prometheus metrics

cAdvisor reads cgroup files and exposes them as Prometheus metrics. For each cgroup file below, identify the corresponding Prometheus metric name.

| cgroup file (v1) | Prometheus metric name | What it measures |
|---|---|---|
| `cpuacct.usage` | ??? | Total CPU time consumed |
| `memory.usage_in_bytes` | ??? | Current memory consumption |
| `memory.limit_in_bytes` | ??? | Memory limit configured |
| `blkio.io_service_bytes_recursive` | ??? | Block I/O bytes |
| `cpuacct.usage_percpu` | ??? | Per-CPU-core usage |

### Your task

Fill in the `???` placeholders with the correct Prometheus metric names. Use the cAdvisor `/metrics` endpoint or documentation to find them.

Hint: cAdvisor metric names follow the pattern `container_<resource>_<measurement>_<unit>`.

---

## Part C: Apply the USE Method

The USE method (Utilization, Saturation, Errors) is a systematic approach to monitoring. For each resource below, identify which cAdvisor/Prometheus metric covers each USE dimension.

| Resource | Utilization | Saturation | Errors |
|----------|-------------|------------|--------|
| CPU | ??? | ??? | (none typically exposed) |
| Memory | ??? | ??? | OOM kill events |
| Disk I/O | ??? | ??? | I/O error counters |
| Network | ??? | ??? | Packet error/drop counters |

### Your task

Complete the table with real Prometheus metric names from cAdvisor.

---

## Part D: Metric types

Prometheus has four metric types. For each description below, name the metric type:

1. A value that only goes up (e.g., total bytes received): `???`
2. A value that can go up and down (e.g., current memory usage): `???`
3. A value sampled at configurable buckets (e.g., request duration distribution): `???`
4. A value that is a simple number snapshot (e.g., number of CPUs): `???`

---

## Part E: PromQL reasoning

Without running any queries, reason about what each PromQL expression computes:

1. `rate(container_cpu_usage_seconds_total{name="web"}[5m])`
   - What does this return? What unit?

2. `container_memory_usage_bytes{name="web"} / container_spec_memory_limit_bytes{name="web"} * 100`
   - What does this return? What unit?

3. `increase(container_restart_count{name="web"}[1h])`
   - What does this return? Under what circumstances would this be greater than 0?

4. `rate(container_network_receive_bytes_total{name="web"}[5m]) * 8`
   - Why multiply by 8? What unit does the result have?

---

## Success Criteria

- [ ] You can explain that containers use cgroups to enforce and measure resource limits.
- [ ] You can map at least 4 cgroup files to their Prometheus metric counterparts.
- [ ] You can complete the USE method table for all four resources with real metric names.
- [ ] You can name all four Prometheus metric types (counter, gauge, histogram, summary).
- [ ] You can interpret PromQL expressions that use `rate()`, `increase()`, and arithmetic operators.

---

## Hints

<details>
<summary>Hint 1: cgroup v1 vs v2 paths</summary>

cgroup v1 organizes resources by subsystem under `/sys/fs/cgroup/<subsystem>/docker/<id>/`.
cgroup v2 uses a unified hierarchy under `/sys/fs/cgroup/system.slice/docker-<id>.scope/`.
Most modern distros (Ubuntu 22.04+, Fedora 31+) default to cgroup v2.

</details>

<details>
<summary>Hint 2: Prometheus metric name pattern</summary>

cAdvisor exposes metrics with the prefix `container_`. The general pattern is:
`container_<resource>_<measurement>_<unit>`

Examples:
- `container_cpu_usage_seconds_total`
- `container_memory_usage_bytes`
- `container_fs_io_time_seconds_total`

</details>

<details>
<summary>Hint 3: Metric types</summary>

- **Counter**: Cumulative value that only goes up. Use `rate()` or `increase()` to get the per-second or per-interval value. Examples: bytes received, requests served, restarts.
- **Gauge**: Value that can go up or down. Use directly. Examples: current memory usage, temperature, active connections.
- **Histogram**: Samples observations in configurable buckets. Use `histogram_quantile()` for percentiles. Examples: request duration, response size.
- **Summary**: Pre-calculated quantiles on the client side. Similar to histogram but less flexible in aggregation.

</details>

<details>
<summary>Hint 4: Memory usage != RSS</summary>

`container_memory_usage_bytes` includes page cache (file-backed memory that the kernel can reclaim). The working set (`container_memory_working_set_bytes`) is a better indicator of memory pressure because it excludes reclaimable cache. The kernel uses the working set to decide when to OOM-kill.

</details>

<details>
<summary>Hint 5: rate() vs increase()</summary>

`rate()` returns per-second average rate over the time window.
`increase()` returns the total increase over the time window (equivalent to `rate() * <seconds in window>`).

For counters, `rate(metric[5m])` gives bytes/second, while `increase(metric[5m])` gives total bytes in the last 5 minutes.

</details>
