# Solution 01: Container Resource Metrics and cgroups

---

## Part A: cgroup file exploration

### Finding the cgroup directory

For cgroup v2 (Ubuntu 22.04+, Fedora 31+):

```bash
CONTAINER_ID=$(docker inspect --format '{{.Id}}' cgroup-lab)
ls /sys/fs/cgroup/system.slice/docker-${CONTAINER_ID}.scope/
```

For cgroup v1:

```bash
ls /sys/fs/cgroup/memory/docker/${CONTAINER_ID}/
```

### Reading the memory limit

cgroup v2:
```bash
cat /sys/fs/cgroup/system.slice/docker-${CONTAINER_ID}.scope/memory.max
# Output: 268435456 (256 * 1024 * 1024 = 256MB)
```

cgroup v1:
```bash
cat /sys/fs/cgroup/memory/docker/${CONTAINER_ID}/memory.limit_in_bytes
# Output: 268435456
```

### Answers to questions

**Q: What value do you see for the memory limit? Convert it to megabytes. Does it match the 256m you specified?**

A: The value is 268435456 bytes, which equals exactly 256 MB (268435456 / 1024 / 1024 = 256). Yes, it matches the `--memory=256m` flag. Docker translates the human-readable `256m` into the exact byte value for the cgroup.

**Q: What value do you see for current memory usage? Why is it not zero?**

A: The value is typically 2-5 MB even though nginx is idle. This is because the nginx process itself needs memory for its executable code, shared libraries, stack, heap, and internal data structures. A running process always consumes some baseline memory even when handling zero requests.

---

## Part B: cgroup files to Prometheus metrics mapping

| cgroup file (v1) | Prometheus metric name | What it measures |
|---|---|---|
| `cpuacct.usage` | `container_cpu_usage_seconds_total` | Total CPU time consumed (nanoseconds in cgroup, seconds in Prometheus) |
| `memory.usage_in_bytes` | `container_memory_usage_bytes` | Current memory consumption |
| `memory.limit_in_bytes` | `container_spec_memory_limit_bytes` | Memory limit configured |
| `blkio.io_service_bytes_recursive` | `container_fs_io_time_seconds_total` | Block I/O time |
| `cpuacct.usage_percpu` | `container_cpu_per_cpu_usage_seconds_total` | Per-CPU-core usage |

Additional commonly used metrics:
- `memory.usage_in_bytes` -> `container_memory_usage_bytes`
- `memory.stat` (rss field) -> `container_memory_rss`
- `memory.stat` (cache field) -> `container_memory_cache`
- `memory.memsw.usage_in_bytes` -> `container_memory_swap`

---

## Part C: USE Method completed table

| Resource | Utilization | Saturation | Errors |
|----------|-------------|------------|--------|
| CPU | `container_cpu_usage_seconds_total` (use `rate()` for percentage) | `container_cpu_cfs_throttled_seconds_total` (time spent throttled) | (not typically exposed via cgroups) |
| Memory | `container_memory_usage_bytes` / `container_spec_memory_limit_bytes` | `container_memory_working_set_bytes` (closer to OOM threshold) | OOM kill events via `dmesg` or container restart count |
| Disk I/O | `container_fs_io_time_seconds_total` (use `rate()` for utilization ratio) | `container_fs_io_current` (number of I/Os in progress) | `container_fs_io_time_seconds_total` errors |
| Network | `container_network_receive_bytes_total` / `container_network_transmit_bytes_total` | `container_network_transmit_packets_dropped_total` | `container_network_receive_errors_total` / `container_network_transmit_errors_total` |

### Explanation

**CPU Saturation:** When a container hits its CPU limit, the kernel throttles it. `container_cpu_cfs_throttled_seconds_total` measures how much time the container spent throttled. If `rate()` of this metric is high, the container needs more CPU.

**Memory Saturation:** The working set (`container_memory_working_set_bytes`) is closer to the OOM threshold than raw usage because it excludes reclaimable page cache. When the working set approaches the limit, the container is at risk of OOM kill.

**Disk I/O Saturation:** `container_fs_io_current` shows how many I/O operations are currently in flight. If this is consistently high, the disk subsystem is saturated.

**Network Errors:** Packet drops and errors indicate network saturation or misconfiguration. A non-zero rate of drops means the container cannot send or receive fast enough.

---

## Part D: Metric types

1. A value that only goes up (e.g., total bytes received): **Counter**
2. A value that can go up and down (e.g., current memory usage): **Gauge**
3. A value sampled at configurable buckets (e.g., request duration distribution): **Histogram**
4. A value that is a simple number snapshot (e.g., number of CPUs): **Gauge** (or Summary for pre-calculated quantiles)

Note: Summary is the fourth type, but it is less commonly used. It provides pre-calculated quantiles on the client side. Histograms are preferred because they allow server-side aggregation.

---

## Part E: PromQL reasoning

### 1. `rate(container_cpu_usage_seconds_total{name="web"}[5m])`

**What it returns:** The average per-second rate of CPU usage over the last 5 minutes.
**Unit:** CPU cores (0.0 to N, where N is the number of available cores).
**Example:** If the value is `0.35`, the container is using 35% of one CPU core. If the value is `1.5`, the container is using 1.5 cores worth of CPU.

To get a percentage, multiply by 100: `rate(...) * 100` gives percent of a single core.

### 2. `container_memory_usage_bytes{name="web"} / container_spec_memory_limit_bytes{name="web"} * 100`

**What it returns:** The container's current memory usage as a percentage of its configured memory limit.
**Unit:** Percent (0-100).
**Example:** If the value is `75.5`, the container is using 75.5% of its memory limit.
**Edge case:** If no memory limit is set, `container_spec_memory_limit_bytes` is 0, and the expression evaluates to `+Inf`. Always filter with `container_spec_memory_limit_bytes > 0`.

### 3. `increase(container_restart_count{name="web"}[1h])`

**What it returns:** The total number of times the container restarted in the last hour.
**When it is greater than 0:** The container was restarted (by Docker, by OOM kill, by a health check failure, or by an explicit `docker restart`). A value of 3 means the container restarted 3 times in the last hour, which is a strong signal of instability.

### 4. `rate(container_network_receive_bytes_total{name="web"}[5m]) * 8`

**Why multiply by 8:** The raw metric is in bytes. Multiplying by 8 converts bytes to bits (1 byte = 8 bits).
**Unit:** Bits per second (bps).
**Reason:** Network throughput is conventionally measured in bits per second (bps, Mbps, Gbps), not bytes per second. The `rate()` already gives bytes/second, so `* 8` converts to bits/second.

---

## Key Takeaways

1. **Containers are cgroups.** Every container metric comes from cgroup virtual files under `/sys/fs/cgroup/`. cAdvisor reads these files and exposes them as Prometheus metrics.

2. **USE method is systematic.** For every resource (CPU, memory, disk, network), check Utilization (how busy), Saturation (how much waiting), and Errors (how many failures). This framework ensures you do not miss critical metrics.

3. **Metric types matter.** Counters need `rate()`, gauges are used directly, histograms need `histogram_quantile()`. Using the wrong function on the wrong type produces meaningless results.

4. **Working set > raw usage.** For memory alerting, use `container_memory_working_set_bytes` instead of `container_memory_usage_bytes`. The working set excludes reclaimable cache and is what the kernel uses for OOM decisions.
