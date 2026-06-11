# Solution 03: Traceroute and Path Analysis

## Part A: How Traceroute Works

### 1. Default Protocol

traceroute uses **UDP** packets by default (on Linux). It sends UDP
datagrams to high port numbers (starting at 33434). On Windows, tracert
uses **ICMP** echo requests.

### 2. How It Discovers Each Hop

```
Mechanism: TTL (Time To Live) exploitation

Step 1: Send packet with TTL=1
  → First router decrements TTL to 0
  → Router sends back ICMP "Time Exceeded" message
  → traceroute records the router's IP and round-trip time

Step 2: Send packet with TTL=2
  → First router decrements to 1, forwards
  → Second router decrements to 0
  → Second router sends back ICMP "Time Exceeded"
  → traceroute records the second router

Step N: Send packet with TTL=N
  → Packet reaches Nth router before TTL expires
  → Eventually reaches destination (gets ICMP "Port Unreachable" for UDP)

Each hop is tested 3 times (3 packets per TTL value).
```

### 3. What Does `*` Mean?

A `*` (asterisk) means **no response was received** within the timeout
period (default: 5 seconds). Possible causes:

- **Firewall dropping packets silently:** The router or a firewall between
  you and the destination drops the packet without sending an ICMP response.
- **Rate limiting:** The router is configured to not send ICMP responses
  for traceroute traffic (to prevent information leakage).
- **Router CPU overload:** The router is too busy to generate ICMP responses.
- **ICMP filtering:** The network blocks ICMP "Time Exceeded" messages.

Three `* * *` in a row usually means the hop is firewalled. traceroute
continues to the next hop because subsequent routers may respond.

### 4. traceroute vs tracert

| Aspect | traceroute (Linux) | tracert (Windows) |
|--------|-------------------|-------------------|
| Default protocol | UDP | ICMP |
| Default port | 33434+ | N/A (ICMP) |
| Behavior with firewalls | May be blocked (UDP) | May be blocked (ICMP) |
| Options | Many (-I for ICMP, -T for TCP) | Few |
| Output format | Similar | Similar |

## Part B: Interpret Traceroute Output

### Hop-by-Hop Analysis

```
 1  gateway.eu-office.com (10.0.0.1)        1.234 ms  ← Local gateway (1ms)
 2  isp-core-1.eu-isp.net (192.168.1.1)     5.678 ms  ← ISP backbone (5ms)
 3  isp-edge.eu-isp.net (192.168.2.1)       8.901 ms  ← ISP edge (9ms)
 4  transatlantic-cable.net (10.10.10.1)     85.234 ms ← Transatlantic link (85ms)
 5  * * *                                              ← No response (firewall)
 6  us-backbone-1.isp.net (172.16.1.1)      150.567 ms ← US backbone (150ms)
 7  us-backbone-2.isp.net (172.16.2.1)      290.123 ms ← BOTTLENECK! (+140ms jump)
 8  us-backbone-3.isp.net (172.16.3.1)      291.234 ms ← Similar latency
 9  datacenter-gw.example.com (203.0.113.1) 292.345 ms ← DC gateway
10  app.example.com (203.0.113.10)           293.456 ms ← Destination
```

### 1. Where Does Latency Increase Significantly?

**Between hops 6 and 7:** Latency jumps from ~150ms to ~290ms (+140ms).

This indicates a problem on the link between us-backbone-1 and
us-backbone-2. Possible causes:
- **Network congestion:** The link is saturated, causing queuing delays
- **Longer path:** Traffic is being routed through a longer physical path
- **QoS policy:** Traffic is being deprioritized
- **Link degradation:** Physical link issues (bad cable, failing hardware)

### 2. What Does Hop 5 Indicate?

Hop 5 (`* * *`) means no ICMP "Time Exceeded" response was received.
This is common for routers that:
- Drop traceroute traffic silently (security policy)
- Are firewalled to prevent network topology disclosure
- Rate-limit ICMP responses

This is **not necessarily a problem** -- it just means that hop does not
respond to traceroute. The trace continues to hop 6, which does respond.

### 3. Is the Problem on Your Network, ISP, or Destination?

```
Your network (hops 1-3):     Latency 1-9ms    → NORMAL
Transatlantic (hop 4):       Latency 85ms     → NORMAL (expected for EU→US)
US backbone (hops 6-7):      Latency jump     → PROBLEM HERE
Destination (hops 8-10):     Latency stable   → NORMAL
```

**The problem is on the US ISP backbone** (between hops 6 and 7).
This is outside your control. You would need to:
1. Contact the ISP to report the issue
2. Consider using a CDN to serve content from EU
3. Consider a different ISP or peering arrangement

## Part C: Traceroute Variants

| Tool | Protocol | When to Use |
|------|----------|-------------|
| `traceroute` | UDP | Default. Works when UDP is not blocked. Fastest. |
| `traceroute -I` | ICMP | When UDP is blocked. More likely to pass through firewalls. Use when traceroute shows many `*`. |
| `traceroute -T` | TCP | When only TCP ports are open (e.g., behind a firewall that allows only port 80/443). Most reliable through firewalls. |
| `tcptraceroute` | TCP | Same as `-T` but more features. Useful for testing specific TCP ports. |
| `mtr` | ICMP/UDP | Combines ping and traceroute. Continuous monitoring. Shows packet loss and latency statistics over time. Best for intermittent issues. |

### When to Use Each

```
Scenario 1: Normal traceroute works
  → Use default `traceroute`

Scenario 2: Many * in output
  → Try `traceroute -I` (ICMP) or `traceroute -T -p 80` (TCP port 80)

Scenario 3: Intermittent latency issues
  → Use `mtr --report app.example.com` (runs 100 cycles, shows statistics)

Scenario 4: Behind a strict firewall
  → Use `traceroute -T -p 443` (TCP to HTTPS port, most likely allowed)

Scenario 5: Windows environment
  → Use `tracert` (ICMP) or `Test-NetConnection` (PowerShell)
```

## Part D: Path Analysis

### Possible Causes of Latency Jump (Hops 6-7)

1. **Network congestion:** The link between backbone-1 and backbone-2 is
   saturated. Packets are queuing in router buffers, adding latency.

2. **Routing change:** BGP may have changed the path, routing traffic
   through a longer or more congested route.

3. **Link saturation:** The physical link is at capacity. New traffic
   must wait for existing traffic to clear.

4. **QoS deprioritization:** Traffic may be classified as lower priority
   and queued behind other traffic.

### Investigation Steps

```bash
# 1. Run mtr for continuous monitoring
mtr --report --report-cycles 100 app.example.com

# 2. Check if the path is consistent
traceroute app.example.com  # Run 3 times, compare hops

# 3. Test from different source
traceroute app.example.com  # From a different EU location

# 4. Compare with known-good path
traceroute other-us-server.com  # Does the same bottleneck appear?

# 5. Check BGP routing
whois 203.0.113.10  # Check the ASN
bgp.he.net          # Check BGP paths
```

### Common Mistakes to Avoid

- **Assuming `*` means the router is down.** It usually means the router
  does not respond to traceroute (firewall policy). The trace continues.
- **Blaming the first high-latency hop.** Latency is cumulative. A hop
  with 100ms does not necessarily add 100ms -- it may just be far away.
  Look for **jumps** between consecutive hops.
- **Running traceroute once.** Routing can change. Run it multiple times
  to see if the path is consistent. Use mtr for continuous monitoring.
- **Ignoring asymmetric paths.** The path from A to B may differ from B
  to A. Run traceroute from both directions if possible.

## Key Takeaway

Traceroute reveals the network path by exploiting TTL. Each hop shows the
router IP and round-trip time. Latency jumps between hops indicate
bottlenecks. `*` means no response (usually firewalled, not down). Use mtr
for continuous monitoring and TCP traceroute when UDP/ICMP are blocked.
The key skill is identifying where latency increases and determining whether
the problem is in your network, the ISP, or the destination.
