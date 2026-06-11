# Solution 04: Network Stack Optimization

## Part A: TCP Fast Open

```bash
# Enable TCP Fast Open for both client and server
net.ipv4.tcp_fastopen = 3
```

### What TFO Does

TCP Fast Open eliminates one round trip from connection establishment:

**Normal TCP (3 round trips):**
```
Client                     Server
  |--- SYN --------------->|     RTT 1
  |<-- SYN-ACK ------------|     RTT 2
  |--- ACK + Data -------->|     RTT 3
  |<-- Response -----------|     RTT 4 (first data)
```

**TCP Fast Open (2 round trips):**
```
Client                     Server
  |--- SYN + Data -------->|     RTT 1 (data in SYN)
  |<-- SYN-ACK + Data -----|     RTT 2 (response)
```

### Bitmask Values
- `0`: TFO disabled
- `1`: TFO enabled for client only (bit 1)
- `2`: TFO enabled for server only (bit 2)
- `3`: TFO enabled for both client and server (bit 1 + bit 2)

**Why 3**: `3 = 1 | 2` (binary: 11). This enables TFO for both outbound connections (client) and inbound connections (server).

---

## Part B: BBR Congestion Control

```bash
# Check available congestion control algorithms
sysctl net.ipv4.tcp_available_congestion_control
# Typical output: net.ipv4.tcp_available_congestion_control = cubic reno bbr

# Load BBR kernel module (if not already loaded)
sudo modprobe tcp_bbr

# Set BBR as the default congestion control
sudo sysctl -w net.ipv4.tcp_congestion_control=bbr

# Set fair queuing (required for BBR)
sudo sysctl -w net.core.default_qdisc=fq

# Verify BBR is active
sysctl net.ipv4.tcp_congestion_control
# Expected output: net.ipv4.tcp_congestion_control = bbr

# Persist across reboots
echo "net.ipv4.tcp_congestion_control = bbr" | sudo tee -a /etc/sysctl.d/99-performance.conf
echo "net.core.default_qdisc = fq" | sudo tee -a /etc/sysctl.d/99-performance.conf
```

**Why BBR is better**: Traditional congestion control (cubic) reacts to packet loss by reducing throughput. BBR models the network path (bandwidth and RTT) and maintains optimal throughput without causing congestion. On lossy networks, BBR can achieve 10-25% higher throughput than cubic.

---

## Part C: TCP Buffer Sizes

```bash
# /etc/sysctl.d/99-performance.conf

# TCP receive buffer: min=4KB, default=85KB, max=16MB
net.ipv4.tcp_rmem = 4096 87380 16777216

# TCP send buffer: min=4KB, default=64KB, max=16MB
net.ipv4.tcp_wmem = 4096 65536 16777216

# Maximum buffer size for explicit setsockopt() calls
net.core.rmem_max = 16777216
net.core.wmem_max = 16777216

# Enable window scaling (required for buffers > 64KB)
net.ipv4.tcp_window_scaling = 1

# Enable selective acknowledgments
net.ipv4.tcp_sack = 1
```

### Why min < default < max

The kernel auto-tunes buffer sizes between min and max:

- **min (4KB)**: Minimum buffer size. The kernel will not shrink below this.
- **default (64-85KB)**: Starting buffer size for new connections.
- **max (16MB)**: Maximum buffer size. The kernel can grow up to this for high-bandwidth connections.

**Why this pattern**: Small connections (low bandwidth, short duration) use small buffers (min). Large connections (high bandwidth, long duration) use large buffers (up to max). The kernel dynamically adjusts based on network conditions (BDP = bandwidth * RTT).

**Why window scaling is required**: Without window scaling, the TCP window is limited to 64KB (16-bit field). With scaling, the window can be up to 1GB. For high-bandwidth connections (1 Gbps+), a 64KB window limits throughput to `64KB / RTT`. At 100ms RTT, that is only 640 KB/s. With a 16MB window, throughput can reach 160 MB/s.

---

## Part D: Benchmark Network Performance

### TCP Throughput (iperf3)

```bash
# Start server
iperf3 -s &

# Test with 4 parallel streams for 30 seconds
iperf3 -c <server_ip> -t 30 -P 4

# Expected improvement with BBR:
# Before (cubic): ~800 Mbps on lossy network
# After (BBR): ~950 Mbps on lossy network
```

### HTTP Throughput (wrk)

```bash
# Before optimization
wrk -t4 -c400 -d30s http://server:8080/api/users

# After optimization (sysctl + BBR)
wrk -t4 -c400 -d30s http://server:8080/api/users

# Expected improvement:
# Before: 10,000 RPS, P95=50ms
# After:  13,000 RPS, P95=35ms
```

### Connection Latency (curl)

```bash
# Measure connection and TTFB latency
curl -w "\n\
  DNS:        %{time_namelookup}s\n\
  Connect:    %{time_connect}s\n\
  TLS:        %{time_appconnect}s\n\
  TTFB:       %{time_starttransfer}s\n\
  Total:      %{time_total}s\n" \
  -o /dev/null -s http://server:8080/api/users

# Expected improvement with TFO:
# Before: Connect=1.5ms, TTFB=3ms
# After:  Connect=0.5ms, TTFB=2ms (TFO saves 1 RTT)
```

---

## Common Mistakes
1. **Setting buffer sizes too large**: Very large buffers (256MB+) waste memory and can cause latency spikes (more data to buffer). 16MB is a good maximum.
2. **Not enabling window scaling**: Without `tcp_window_scaling=1`, the kernel ignores buffer sizes above 64KB.
3. **Using BBR without fq qdisc**: BBR requires the `fq` (fair queuing) qdisc. Without it, BBR does not function correctly.
4. **Not testing on the actual network**: BBR improvements are most visible on lossy or high-latency networks. On a local network with no loss, the difference is minimal.

## Relevant README Sections
- [Network Stack](../README.md#network-stack)
- [The Right Way](../README.md#the-right-way)
