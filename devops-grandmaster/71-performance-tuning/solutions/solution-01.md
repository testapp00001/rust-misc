# Solution 01: Kernel Parameter Analysis

## Part A: Identify Limiting Parameters

### net.core.somaxconn = 128
- **What it controls**: Maximum length of the TCP listen backlog queue. When a server calls `listen()`, this is the maximum number of pending connections.
- **Why the default is too low**: 128 is adequate for a desktop machine with a few connections. A web server under load may have thousands of connections arriving simultaneously. When the backlog is full, new connections are dropped silently (the client sees a connection timeout).

### net.ipv4.tcp_fin_timeout = 60
- **What it controls**: How long (in seconds) a socket stays in FIN_WAIT_2 state after the connection is closed.
- **Why the default is too low**: 60 seconds means closed connections hold resources for 1 minute. Under high connection churn (short-lived HTTP connections), this accumulates thousands of half-closed sockets, consuming memory and port numbers.

### fs.file-max = 65535
- **What it controls**: System-wide maximum number of open file descriptors. Every open file, socket, and pipe consumes one file descriptor.
- **Why the default is too low**: 65,535 is insufficient for a server handling tens of thousands of concurrent connections. Each connection uses one file descriptor. At 65,535, the system cannot accept new connections.

### vm.swappiness = 60
- **What it controls**: How aggressively the kernel swaps application memory to disk. Range: 0 (never swap if memory available) to 100 (swap aggressively).
- **Why the default is too high**: 60 is designed for desktop use where swapping is acceptable. For servers, swapping causes latency spikes (disk I/O is 1000x slower than RAM). A value of 10 keeps application data in RAM.

### net.ipv4.ip_local_port_range = 32768 60999
- **What it controls**: Range of ephemeral ports available for outbound connections.
- **Why the default is too small**: 32768-60999 = 28,231 ports. A server making outbound connections to a database or API may exhaust this range under load. Expanding to 1024-65535 provides 64,511 ports.

---

## Part B: Explain the Impact

### somaxconn reached
- **Impact**: New TCP connections get dropped. The client receives a connection timeout. The server log shows no errors (the drop happens in the kernel, not the application).
- **Symptoms**: Intermittent connection failures under load. Works fine at low traffic, fails at high traffic.

### tcp_fin_timeout too long
- **Impact**: Sockets accumulate in TIME_WAIT state. Eventually, the system runs out of available ports for new outbound connections.
- **Symptoms**: "Cannot assign requested address" errors. High `TIME_WAIT` count in `ss -s` output.

### file-max reached
- **Impact**: The process cannot open new files or sockets. The `accept()` system call fails with EMFILE (too many open files).
- **Symptoms**: Server stops accepting connections. Application logs show "too many open files" errors.

### swappiness too high
- **Impact**: The kernel swaps application memory to disk even when RAM is available. This causes latency spikes as swapped pages are read back from disk.
- **Symptoms**: Intermittent latency spikes. `vmstat` shows non-zero `si` (swap in) and `so` (swap out) values.

### ip_local_port_range too small
- **Impact**: Outbound connections fail when all ephemeral ports are in use. This affects connections to databases, APIs, and external services.
- **Symptoms**: "Cannot assign requested address" errors when connecting to backend services.

---

## Part C: Recommended Optimized Values

| Parameter | Default | Optimized | Reason |
|-----------|---------|-----------|--------|
| net.core.somaxconn | 128 | 65535 | Handle burst connection arrivals without dropping |
| net.ipv4.tcp_fin_timeout | 60 | 15 | Recycle closed connections 4x faster |
| fs.file-max | 65535 | 2097152 | Support millions of concurrent connections |
| vm.swappiness | 60 | 10 | Keep application data in RAM, minimize swapping |
| net.ipv4.ip_local_port_range | 32768 60999 | 1024 65535 | 2.3x more outbound connection ports |

---

## Common Mistakes
1. **Setting swappiness to 0**: A value of 0 does not mean "never swap." It means "only swap to avoid OOM." Use 10 instead, which allows minimal swapping for background processes.
2. **Not increasing file-max and limits together**: Increasing `fs.file-max` without increasing per-user limits does not help. Both must be changed.
3. **Setting somaxconn without application support**: The application must also set its listen backlog high. `somaxconn` is the kernel limit; the application's `listen(backlog)` must also be increased.

## Relevant README Sections
- [Linux Kernel Tuning (sysctl)](../README.md#linux-kernel-tuning-sysctl)
- [The Naive Way](../README.md#the-naive-way)
- [The Right Way](../README.md#the-right-way)
