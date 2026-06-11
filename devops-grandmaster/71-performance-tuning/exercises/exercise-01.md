# Exercise 01: Kernel Parameter Analysis

**Type:** Conceptual
**Time:** 15 minutes
**Difficulty:** Easy

## Objective
Understand the purpose of key Linux kernel parameters that affect server performance, identify which parameters are misconfigured by default, and explain why the defaults are inadequate for high-throughput workloads.

## Scenario
You have deployed a new web server with default Linux settings. Under load testing, the server handles only 5,000 concurrent connections before dropping connections. The server has 32GB of RAM and 16 CPU cores, but CPU utilization never exceeds 20%. Something is limiting throughput.

## Tasks

### Part A: Identify Limiting Parameters
The following kernel parameters have their default values. For each one, explain what it controls and why the default is too low for a high-throughput server:

```
net.core.somaxconn = 128
net.ipv4.tcp_fin_timeout = 60
fs.file-max = 65535
vm.swappiness = 60
net.ipv4.ip_local_port_range = 32768 60999
```

<details>
<summary>Hint</summary>
`somaxconn` limits the TCP listen backlog. `tcp_fin_timeout` controls how long TIME_WAIT sockets persist. `file-max` limits open file descriptors (sockets count as files). `swappiness` controls how aggressively the kernel swaps. `ip_local_port_range` limits outbound connections.
</details>

### Part B: Explain the Impact
For each parameter above, explain what happens when the limit is reached under high load:

<details>
<summary>Hint</summary>
When `somaxconn` is reached, new connections get dropped. When `file-max` is reached, the process cannot open new sockets. When `ip_local_port_range` is exhausted, outbound connections fail. When `swappiness` is high, the kernel swaps application memory, causing latency spikes.
</details>

### Part C: Recommend Optimized Values
For each parameter, recommend an optimized value for a high-throughput web server and explain why.

<details>
<summary>Hint</summary>
`somaxconn`: 65535 (handle burst connections). `tcp_fin_timeout`: 15 (faster connection recycling). `file-max`: 2097152 (support millions of connections). `swappiness`: 10 (keep application data in RAM). `ip_local_port_range`: 1024 65535 (more outbound connections).
</details>

## Success Criteria
- [ ] You can explain what each kernel parameter controls.
- [ ] You can describe the failure mode when each limit is reached.
- [ ] You can recommend optimized values with justification.
- [ ] You can explain why Linux defaults are designed for desktop use, not servers.
- [ ] You can identify which parameter is most likely causing the 5,000 connection limit.

## What You Should Understand After This Exercise
Linux kernel defaults are conservative -- designed for compatibility with desktop workloads. For high-throughput servers, these defaults become bottlenecks: connection limits, file descriptor limits, and aggressive swapping. Tuning these parameters can yield 2-10x improvements in throughput and connection capacity without changing application code.
