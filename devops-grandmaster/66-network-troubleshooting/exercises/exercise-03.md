# Exercise 03: Traceroute and Path Analysis

**Type:** Independent
**Time:** 35 min
**Difficulty:** Medium

## Objective

Use traceroute to diagnose network path issues, understand how traceroute works, and interpret its output to identify where packets are being dropped or delayed.

## Scenario

Users in your EU office report slow access to a US-hosted application. The normal latency is ~100ms, but users are seeing 300ms+ with occasional timeouts. You suspect a network path issue.

## Tasks

### Part A: How Traceroute Works

Explain the mechanism traceroute uses to discover the network path:

1. What protocol does traceroute use by default?
2. How does it discover each hop?
3. What does a `*` (asterisk) in the output mean?
4. What is the difference between `traceroute` and `tracert`?

<details>
<summary>Hint</summary>

Traceroute sends packets with incrementing TTL (Time To Live). Each router decrements TTL and returns an ICMP "Time Exceeded" message when TTL reaches 0. The `*` means no response was received (firewall drop or rate limiting). tracert is the Windows version.

</details>

### Part B: Interpret Traceroute Output

Given this traceroute from EU to US, identify the problem:

```
traceroute to app.example.com (203.0.113.10), 30 hops max, 60 byte packets
 1  gateway.eu-office.com (10.0.0.1)     1.234 ms  1.156 ms  1.089 ms
 2  isp-core-1.eu-isp.net (192.168.1.1)  5.678 ms  5.432 ms  5.210 ms
 3  isp-edge.eu-isp.net (192.168.2.1)    8.901 ms  8.765 ms  8.543 ms
 4  transatlantic-cable.net (10.10.10.1)  85.234 ms  84.987 ms  85.123 ms
 5  * * *
 6  us-backbone-1.isp.net (172.16.1.1)   150.567 ms  149.876 ms  150.234 ms
 7  us-backbone-2.isp.net (172.16.2.1)   290.123 ms  289.456 ms  290.789 ms
 8  us-backbone-3.isp.net (172.16.3.1)   291.234 ms  290.567 ms  291.890 ms
 9  datacenter-gw.example.com (203.0.113.1) 292.345 ms 291.678 ms 292.012 ms
10  app.example.com (203.0.113.10)        293.456 ms 292.789 ms 293.123 ms
```

1. Where does the latency increase significantly?
2. What does hop 5 (the `* * *`) indicate?
3. Is the problem on your network, the ISP, or the destination?

<details>
<summary>Hint</summary>

Look at the latency jump between hops. A sudden increase indicates a bottleneck. Hop 5 shows no response (firewall or rate limiting). The latency jump from hop 6 to hop 7 (~150ms to ~290ms) is the key issue.

</details>

### Part C: Traceroute Variants

Explain when to use each traceroute variant:

| Tool | Protocol | When to Use |
|------|----------|-------------|
| `traceroute` | UDP | |
| `traceroute -I` | ICMP | |
| `traceroute -T` | TCP | |
| `tcptraceroute` | TCP | |
| `mtr` | ICMP/UDP | |

<details>
<summary>Hint</summary>

UDP traceroute may be blocked by firewalls. ICMP traceroute (`-I`) is more likely to get through. TCP traceroute (`-T`) is useful when only TCP ports are open (e.g., port 80). mtr combines ping and traceroute for continuous monitoring.

</details>

### Part D: Path Analysis

Given that the latency jump is between hops 6 and 7 (from ~150ms to ~290ms), what are the possible causes and how would you investigate further?

<details>
<summary>Hint</summary>

Possible causes: network congestion, routing change (longer path), link saturation, or QoS policy. Run mtr for continuous monitoring. Check if the path is consistent. Compare with a known-good traceroute.

</details>

## Success Criteria

- [ ] You can explain how traceroute discovers the network path
- [ ] You can identify latency bottlenecks in traceroute output
- [ ] You can interpret `*` (no response) in traceroute output
- [ ] You can choose the right traceroute variant for different scenarios

## What You Should Understand After This Exercise

Traceroute reveals the network path by sending packets with incrementing TTL. Each router returns an ICMP message when TTL expires. Latency increases at each hop indicate distance or congestion. `*` means the router does not respond (firewall or rate limiting). The key skill is identifying where latency jumps and determining whether the problem is in your network, the ISP, or the destination.
