# Exercise 04: Network Stack Optimization

**Type:** Challenge
**Time:** 45 minutes
**Difficulty:** Medium-Hard

## Objective
Optimize the Linux network stack for high-throughput, low-latency communication by tuning TCP parameters, enabling advanced features like TCP Fast Open and BBR congestion control, and benchmarking the improvements.

## Scenario
Your API server handles 50,000 requests per second across 5,000 concurrent connections. Network latency is higher than expected (P99 of 100ms when it should be 20ms). TCP connection establishment takes 3 round trips. You need to optimize the network stack.

## Tasks

### Part A: Enable TCP Fast Open
Write the sysctl configuration to enable TCP Fast Open (TFO) for both client and server connections. Explain:
1. What TFO does
2. How many round trips it saves
3. What the `tcp_fastopen` bitmask values mean (1, 2, 3)

<details>
<summary>Hint</summary>
`net.ipv4.tcp_fastopen = 3` enables TFO for both client (bit 1) and server (bit 2). TFO saves one round trip by sending data in the SYN packet. Normal TCP: 3 round trips (SYN, SYN-ACK+data, ACK). TFO: 2 round trips (SYN+data, SYN-ACK+data).
</details>

### Part B: Configure BBR Congestion Control
Write the commands to:
1. Check which congestion control algorithms are available
2. Switch from cubic (default) to bbr
3. Set the queuing discipline to fq (required for BBR)
4. Verify BBR is active

<details>
<summary>Hint</summary>
Check available: `sysctl net.ipv4.tcp_available_congestion_control`. Switch: `sysctl -w net.ipv4.tcp_congestion_control=bbr`. Queue discipline: `sysctl -w net.core.default_qdisc=fq`. Verify: `sysctl net.ipv4.tcp_congestion_control`.
</details>

### Part C: Optimize TCP Buffer Sizes
Write the sysctl configuration for TCP buffer sizes that:
1. Sets minimum, default, and maximum receive buffer sizes
2. Sets minimum, default, and maximum send buffer sizes
3. Enables TCP window scaling
4. Enables selective acknowledgments (SACK)

Explain why the buffer sizes should follow the pattern: min < default < max.

<details>
<summary>Hint</summary>
`net.ipv4.tcp_rmem = 4096 87380 16777216` sets min=4KB, default=85KB, max=16MB. The kernel auto-tunes between min and max based on network conditions. `net.ipv4.tcp_window_scaling = 1` enables scaling beyond 64KB windows. `net.ipv4.tcp_sack = 1` enables selective ACKs.
</details>

### Part D: Benchmark Network Performance
Write a benchmark plan that tests:
1. TCP throughput using `iperf3`
2. HTTP throughput using `wrk` before and after optimization
3. Connection establishment time using `curl` with timing

For each test, explain what metric you are measuring and what improvement to expect.

<details>
<summary>Hint</summary>
iperf3: `iperf3 -c <server> -t 30 -P 4` measures raw TCP throughput. wrk: `wrk -t4 -c400 -d30s http://server/api` measures HTTP throughput. curl: `curl -w "connect: %{time_connect}s\nTTFB: %{time_starttransfer}s\n" -o /dev/null -s http://server/api` measures connection and TTFB latency.
</details>

## Success Criteria
- [ ] TCP Fast Open is enabled with the correct bitmask value.
- [ ] BBR congestion control is active and verified.
- [ ] TCP buffer sizes are configured with min/default/max values.
- [ ] The benchmark shows measurable improvement in throughput or latency.
- [ ] You can explain why each optimization improves network performance.

## What You Should Understand After This Exercise
The Linux network stack has many parameters that affect performance. TCP Fast Open reduces connection establishment latency. BBR congestion control provides better throughput on lossy networks. Proper buffer sizes allow the kernel to handle high-bandwidth connections. These optimizations work together to reduce latency and increase throughput.
