# Solution 04: Packet Analysis with Wireshark

## Part A: Connection Analysis

### 1. Round-Trip Time (RTT)

```
Packet 1 (SYN):     T=0.000s  Client → Server
Packet 2 (SYN-ACK): T=0.025s  Server → Client

RTT = 0.025s - 0.000s = 25ms
```

This is a 25ms round-trip time, which is typical for a connection crossing
a geographic distance (e.g., US to EU would be ~80-120ms; within a region
would be 1-10ms; 25ms suggests moderate distance).

### 2. Handshake Issues

The handshake itself is clean:
- SYN sent, SYN-ACK received, ACK sent -- all within 25ms
- No retransmissions during handshake
- No RST or error flags

**No issues with the handshake.**

### 3. Client's Initial Window Size

```
Packet 1 (SYN): win 65535
```

The client advertises a receive window of 65535 bytes (64KB). This tells
the server: "You can send up to 64KB before waiting for my acknowledgment."

Modern TCP stacks also use **window scaling** (TCP option) to support
larger windows. The actual window may be `win 65535 * scale_factor`.

## Part B: Data Transfer Analysis

### 1. Initial Congestion Window

```
Packet 5 (HTTP 200 OK):  T=0.050  Server → Client (HTTP headers)
Packet 6 (TCP segment):   T=0.050  Server → Client (1460 bytes)
Packet 7 (TCP segment):   T=0.051  Server → Client (1460 bytes)
Packet 8 (TCP segment):   T=0.052  Server → Client (1460 bytes)
Packet 9 (Dup ACK):       T=0.550  Client → Server
```

The server sends **3 segments** immediately after the HTTP response (packets
6-8). This is the **initial congestion window (cwnd)** of 3 segments.

```
Initial cwnd = 3 segments × 1460 bytes = 4,380 bytes
```

This is typical for TCP Cubic (Linux default). Some modern stacks use
cwnd=10 (Google's BBR or tuned Cubic).

### 2. What Happens at Packet 9 (TCP Dup ACK)?

```
Packet 9: [TCP Dup ACK] ACK 4381
```

A **duplicate ACK** means the client received packets out of order.
The client expected packet starting at sequence 4381 but received
something else (likely a later segment). The client sends a duplicate
ACK to inform the server.

```
Expected sequence: 4381 (after 3 segments: 1001 + 1460 + 1460 + 1460 = 4381)
Client received:   A later segment (e.g., sequence 5841)
Client sends:      Dup ACK saying "I need 4381" (selective ACK)
```

This indicates **packet loss** -- one of the segments (likely packet 7
or 8) was lost in transit.

### 3. Why Does Packet 10 Say "Retransmission"?

```
Packet 10: [TCP Retransmission] 1460 bytes  T=1.050
```

The server did not receive an ACK for the lost segment within the
**retransmission timeout (RTO)**. The RTO is typically 200ms-1s.

```
Timeline:
  T=0.051: Segment sent (packet 7 or 8)
  T=0.550: Client sends Dup ACK (500ms later)
  T=1.050: Server retransmits (500ms RTO after Dup ACK)

The server waited ~1 second for an ACK, then retransmitted.
```

### 4. Bytes Transferred in First Second

```
Packets 6-8: 3 segments × 1460 bytes = 4,380 bytes
Packet 5 (HTTP headers): ~200 bytes (estimated)
Packet 10 (retransmission): 1460 bytes (duplicate, does not count as new data)

New data in first second: ~4,580 bytes
```

This is **extremely low throughput** for a 1MB file.

## Part C: Performance Diagnosis

### 1. Effective Throughput

```
File size: 1 MB = 1,048,576 bytes
Transfer time: 30 seconds
Effective throughput: 1,048,576 / 30 = 34,953 bytes/sec ≈ 280 kbps
```

Expected throughput for a 25ms RTT link:

```
Bandwidth-Delay Product (BDP) = Bandwidth × RTT
If link capacity is 10 Mbps:
  BDP = 1,250,000 bytes/sec × 0.025 sec = 31,250 bytes
  This means the pipe can hold 31,250 bytes in transit

With cwnd=3 segments (4,380 bytes):
  Utilization = 4,380 / 31,250 = 14%
  The link is 86% idle!
```

### 2. What Is Causing Low Throughput?

The problem is **TCP congestion control combined with packet loss**:

```
Normal TCP behavior:
  cwnd starts at 3 segments
  After ACK: cwnd grows (slow start)
  After loss: cwnd drops to 1, then slow start again

With packet loss:
  T=0.050:  Send 3 segments (cwnd=3)
  T=0.550:  Dup ACK received (loss detected)
  T=1.050:  Retransmit lost segment
  T=1.075:  ACK received (recovery)
  T=1.076:  cwnd reset to 1 (or 2 with fast recovery)
  T=1.076:  Send 1 segment
  ...       Slowly grow cwnd again

Each loss event resets cwnd, causing the connection to spend most
of its time in slow start, never reaching full throughput.
```

### 3. TCP Parameters to Tune

| Parameter | Current | Recommended | Effect |
|-----------|---------|-------------|--------|
| `tcp_congestion_control` | cubic | bbr | BBR is loss-tolerant, maintains high throughput even with packet loss |
| `tcp_slow_start_after_idle` | 1 (on) | 0 (off) | Do not reset cwnd after idle periods |
| `tcp_mtu_probing` | 0 (off) | 1 (on) | Discover optimal MSS to avoid fragmentation |
| `net.ipv4.tcp_window_scaling` | 1 (on) | 1 (on) | Already enabled (good) |
| `net.core.rmem_max` | 212992 | 16777216 | Increase max receive buffer for high BDP links |

```bash
# Apply recommended settings
sysctl -w net.ipv4.tcp_congestion_control=bbr
sysctl -w net.ipv4.tcp_slow_start_after_idle=0
sysctl -w net.ipv4.tcp_mtu_probing=1
```

### Why BBR Helps

Traditional congestion control (Cubic) interprets packet loss as congestion
and reduces the sending rate. BBR (Bottleneck Bandwidth and Round-trip
propagation time) measures the actual bandwidth and RTT, and maintains
high throughput even when packet loss occurs. For lossy links, BBR can
achieve 2-10x higher throughput than Cubic.

## Part D: Packet Capture Commands

### 1. Capture HTTP traffic with headers

```bash
# Using tshark (Wireshark CLI)
tshark -i eth0 -f "tcp port 80" -Y "http" -V

# Using tcpdump (show ASCII content)
tcpdump -i eth0 -A -s 0 'tcp port 80' | grep -E "^(GET|POST|HTTP|Host:|Content-Type:)"

# Capture to file for Wireshark analysis
tcpdump -i eth0 -w http.pcap 'tcp port 80'
```

### 2. Show TCP window sizes

```bash
tshark -i eth0 -f "tcp" -T fields -e tcp.srcport -e tcp.dstport -e tcp.window_size
```

### 3. Filter for retransmissions

```bash
tshark -i eth0 -f "tcp" -Y "tcp.analysis.retransmission"

# Or with tcpdump (less precise)
tcpdump -i eth0 'tcp[tcpflags] & (tcp-syn|tcp-fin|tcp-rst) == 0' -n
# Then look for duplicate sequence numbers
```

### 4. Capture first 100 packets on port 80

```bash
tcpdump -i eth0 -c 100 -w first100.pcap 'tcp port 80'

# Or with tshark
tshark -i eth0 -c 100 -f "tcp port 80" -w first100.pcap
```

### Common Mistakes to Avoid

- **Confusing retransmissions with duplicate ACKs.** A retransmission is
  the sender re-sending data. A duplicate ACK is the receiver reporting
  out-of-order data. Both indicate packet loss, but from different
  perspectives.
- **Not capturing at the right point.** Capture at the server to see what
  the server sends, at the client to see what the client receives. The
  difference shows what was lost in transit.
- **Ignoring TCP window size.** A zero window means the receiver cannot
  accept more data (buffer full). This is a different problem from
  congestion (network overload).
- **Not using `-w` for production captures.** Writing to a pcap file
  preserves all packet details. Terminal output loses data and is
  not analyzable later.

## Key Takeaway

Packet analysis reveals what happens at the wire level. TCP congestion
control starts with a small window and grows as packets are acknowledged.
Packet loss causes retransmissions and window reduction, dramatically
reducing throughput. A small congestion window combined with high latency
creates a bandwidth-delay product bottleneck. BBR congestion control
can maintain throughput even with packet loss.
