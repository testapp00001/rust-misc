# Exercise 02: sysctl Tuning for High Throughput

**Type:** Guided
**Time:** 30 minutes
**Difficulty:** Easy-Medium

## Objective
Create a sysctl configuration file that optimizes Linux kernel parameters for high-throughput server workloads, apply the changes, and measure the impact.

## Scenario
You have a web server running Nginx with default kernel settings. Under load testing with `wrk`, the server handles 10,000 requests/second with P95 latency of 50ms. You need to tune kernel parameters to increase throughput and reduce latency.

## Tasks

### Part A: Create the sysctl Configuration File
Write a sysctl configuration file `/etc/sysctl.d/99-performance.conf` that sets the following parameters to optimized values:

1. Memory management: `vm.swappiness`, `vm.vfs_cache_pressure`, `vm.dirty_ratio`, `vm.dirty_background_ratio`
2. TCP connection backlog: `net.core.somaxconn`, `net.core.netdev_max_backlog`
3. TCP buffer sizes: `net.core.rmem_max`, `net.core.wmem_max`, `net.ipv4.tcp_rmem`, `net.ipv4.tcp_wmem`
4. TCP connection recycling: `net.ipv4.tcp_fin_timeout`, `net.ipv4.tcp_tw_reuse`
5. TCP keepalive: `net.ipv4.tcp_keepalive_time`, `net.ipv4.tcp_keepalive_intvl`, `net.ipv4.tcp_keepalive_probes`
6. Local port range: `net.ipv4.ip_local_port_range`
7. SYN flood protection: `net.ipv4.tcp_syncookies`, `net.ipv4.tcp_max_syn_backlog`
8. Congestion control: `net.ipv4.tcp_congestion_control`, `net.core.default_qdisc`

<details>
<summary>Hint</summary>
For memory: set swappiness to 10, vfs_cache_pressure to 50, dirty_ratio to 10, dirty_background_ratio to 5. For TCP backlog: set somaxconn to 65535. For buffers: set rmem_max and wmem_max to 16777216 (16MB). For connection recycling: set tcp_fin_timeout to 15, tcp_tw_reuse to 1. For congestion: set tcp_congestion_control to bbr, default_qdisc to fq.
</details>

### Part B: Apply and Verify Changes
Write the commands to:
1. Apply all sysctl changes from the configuration file
2. Verify that `net.core.somaxconn` is set to 65535
3. Verify that `net.ipv4.tcp_congestion_control` is set to `bbr`

<details>
<summary>Hint</summary>
Use `sudo sysctl --system` to apply all changes. Use `sysctl net.core.somaxconn` to verify. Use `sysctl net.ipv4.tcp_congestion_control` to verify congestion control.
</details>

### Part C: Measure the Impact
Write a load testing plan that:
1. Benchmarks the server BEFORE tuning (default settings)
2. Applies the sysctl changes
3. Benchmarks the server AFTER tuning
4. Compares throughput (RPS) and P95 latency

Use `wrk` or `hey` for the benchmark.

<details>
<summary>Hint</summary>
Use `wrk -t4 -c100 -d30s http://localhost:8080/` for each benchmark. Run it twice: once with defaults, once after tuning. Compare the requests/sec and latency values in the output.
</details>

### Part D: Explain Each Parameter's Impact
For the top 5 parameters that have the biggest impact on throughput, explain:
- What the default value is
- What the tuned value is
- Why the change improves performance

<details>
<summary>Hint</summary>
The biggest impacts typically come from: somaxconn (connection backlog), tcp_fin_timeout (connection recycling), swappiness (memory management), tcp_tw_reuse (socket reuse), and tcp_congestion_control (BBR vs cubic).
</details>

## Success Criteria
- [ ] The sysctl configuration file contains all required parameters with optimized values.
- [ ] The changes are applied correctly and verified.
- [ ] The load test shows measurable improvement in throughput or latency.
- [ ] You can explain why each parameter was changed and its impact.
- [ ] The configuration file is in the correct location (`/etc/sysctl.d/`).

## What You Should Understand After This Exercise
sysctl tuning is a low-risk, high-reward optimization. The changes are applied at runtime and persist across reboots when placed in `/etc/sysctl.d/`. The biggest gains come from increasing connection limits, reducing connection recycling time, and switching to the BBR congestion control algorithm.
