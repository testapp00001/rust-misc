# Solution 02: sysctl Tuning for High Throughput

## Part A: sysctl Configuration File

```bash
# /etc/sysctl.d/99-performance.conf
# Performance tuning for high-throughput servers

# === Memory Management ===
# Prefer keeping application data in RAM (10 = minimal swapping)
vm.swappiness = 10

# Reduce tendency to reclaim inode/dentry cache (50 = balanced)
vm.vfs_cache_pressure = 50

# Start writeback when 10% of memory is dirty
vm.dirty_ratio = 10

# Start background writeback when 5% of memory is dirty
vm.dirty_background_ratio = 5

# Minimum free memory (64MB) — kernel reclaims below this
vm.min_free_kbytes = 65536

# === TCP Connection Backlog ===
# Maximum listen backlog (pending connections)
net.core.somaxconn = 65535

# Maximum packets queued on INPUT side
net.core.netdev_max_backlog = 65536

# === TCP Buffer Sizes ===
# Receive buffer: min=4KB, default=85KB, max=16MB
net.ipv4.tcp_rmem = 4096 87380 16777216

# Send buffer: min=4KB, default=64KB, max=16MB
net.ipv4.tcp_wmem = 4096 65536 16777216

# Maximum buffer size (for explicit setsockopt calls)
net.core.rmem_max = 16777216
net.core.wmem_max = 16777216

# === TCP Connection Recycling ===
# Close FIN_WAIT_2 connections after 15 seconds (default: 60)
net.ipv4.tcp_fin_timeout = 15

# Allow reuse of TIME_WAIT sockets for new connections
net.ipv4.tcp_tw_reuse = 1

# Maximum TIME_WAIT buckets
net.ipv4.tcp_max_tw_buckets = 65536

# === TCP Keepalive ===
# Detect dead connections after 5 minutes (default: 2 hours)
net.ipv4.tcp_keepalive_time = 300

# Probe every 30 seconds
net.ipv4.tcp_keepalive_intvl = 30

# Give up after 5 probes
net.ipv4.tcp_keepalive_probes = 5

# === Local Port Range ===
# More outbound connection ports (1024-65535 = 64,511 ports)
net.ipv4.ip_local_port_range = 1024 65535

# === SYN Flood Protection ===
# Enable SYN cookies (protect against SYN flood attacks)
net.ipv4.tcp_syncookies = 1

# Maximum SYN backlog
net.ipv4.tcp_max_syn_backlog = 65535

# === Congestion Control ===
# Use BBR (better throughput on lossy networks)
net.ipv4.tcp_congestion_control = bbr

# Fair queuing (required for BBR)
net.core.default_qdisc = fq

# === TCP Optimizations ===
# Enable TCP Fast Open (client + server)
net.ipv4.tcp_fastopen = 3

# Do not slow start after idle (maintain throughput)
net.ipv4.tcp_slow_start_after_idle = 0

# Enable window scaling (support windows > 64KB)
net.ipv4.tcp_window_scaling = 1

# Enable selective acknowledgments (better loss recovery)
net.ipv4.tcp_sack = 1
```

**Why these values**: Each parameter is tuned for high-throughput server workloads. The values are not arbitrary -- they are based on industry best practices (Netflix, Cloudflare, AWS) and are safe for production use.

---

## Part B: Apply and Verify

```bash
# Apply all sysctl changes
sudo sysctl --system

# Verify specific parameters
sysctl net.core.somaxconn
# Expected output: net.core.somaxconn = 65535

sysctl net.ipv4.tcp_congestion_control
# Expected output: net.ipv4.tcp_congestion_control = bbr

sysctl vm.swappiness
# Expected output: vm.swappiness = 10

# Verify from proc filesystem
cat /proc/sys/net/core/somaxconn
# Expected output: 65535
```

**Why `sysctl --system`**: This command reads all configuration files in `/etc/sysctl.d/` and applies them. It is the recommended way to apply sysctl changes persistently.

---

## Part C: Measure the Impact

```bash
#!/bin/bash
# benchmark.sh — Before/after sysctl tuning benchmark

echo "=== Performance Benchmark ==="

# Start a simple HTTP server
python3 -m http.server 8080 &
SERVER_PID=$!
sleep 1

# Benchmark BEFORE tuning (with default settings)
echo ""
echo "--- BEFORE tuning (default sysctl) ---"
wrk -t4 -c100 -d30s http://localhost:8080/

# Apply sysctl changes
sudo sysctl -p /etc/sysctl.d/99-performance.conf

# Benchmark AFTER tuning
echo ""
echo "--- AFTER tuning (optimized sysctl) ---"
wrk -t4 -c100 -d30s http://localhost:8080/

# Cleanup
kill $SERVER_PID
```

**Expected improvement**:
- Throughput: 10-30% increase (more connections handled)
- P95 latency: 20-50% reduction (faster connection recycling)
- Connection capacity: 2-5x more concurrent connections

---

## Part D: Top 5 Impactful Parameters

### 1. net.core.somaxconn (128 -> 65535)
- **Impact**: Eliminates connection drops under burst traffic. The most impactful single change for connection-heavy workloads.
- **Why**: With the default of 128, any burst of more than 128 pending connections causes drops. 65535 handles virtually any burst.

### 2. net.ipv4.tcp_fin_timeout (60 -> 15)
- **Impact**: 4x faster recycling of closed connections. Frees ports and memory faster.
- **Why**: Short-lived HTTP connections close frequently. With 60s timeout, ports accumulate in TIME_WAIT. With 15s, they are recycled 4x faster.

### 3. vm.swappiness (60 -> 10)
- **Impact**: Eliminates latency spikes caused by swapping. Keeps application data in RAM.
- **Why**: Swapping to disk is 1000x slower than RAM access. A single swap event can cause a 10ms+ latency spike.

### 4. net.ipv4.tcp_tw_reuse (0 -> 1)
- **Impact**: Allows reuse of TIME_WAIT sockets for new outbound connections.
- **Why**: Without this, TIME_WAIT sockets consume ports until they expire. With reuse, the kernel can use them for new connections when safe.

### 5. net.ipv4.tcp_congestion_control (cubic -> bbr)
- **Impact**: Better throughput on lossy networks (10-25% improvement). More stable latency.
- **Why**: BBR (Bottleneck Bandwidth and Round-trip propagation time) models the network path instead of reacting to packet loss. It maintains high throughput without causing congestion.

---

## Common Mistakes
1. **Not testing before applying to production**: Some sysctl changes can cause issues in specific environments. Always test in staging first.
2. **Applying changes without verifying**: Use `sysctl -n <parameter>` to verify each change was applied correctly.
3. **Forgetting to load the BBR kernel module**: BBR requires the `tcp_bbr` kernel module. Load it with `modprobe tcp_bbr` before setting the congestion control.
4. **Setting dirty_ratio too low**: A very low `dirty_ratio` (e.g., 1%) causes frequent writeback, increasing I/O. Use 5-10%.

## Relevant README Sections
- [Linux Kernel Tuning (sysctl)](../README.md#linux-kernel-tuning-sysctl)
- [Network Stack](../README.md#network-stack)
- [Apply changes](../README.md#apply-changes)
