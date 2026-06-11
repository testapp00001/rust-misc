# Exercise 02: tcpdump Fundamentals

**Type:** Guided
**Time:** 30 min
**Difficulty:** Easy-Medium

## Objective

Learn to capture and filter network traffic using tcpdump, and interpret basic packet captures.

## Scenario

You need to debug an HTTP connection issue between a client (192.168.1.100) and a web server (10.0.1.10:80). You will use tcpdump to capture the relevant traffic.

## Tasks

### Part A: Basic Capture Commands

Write the tcpdump command for each requirement:

1. Capture all traffic on interface eth0
2. Capture only TCP traffic to port 80
3. Capture traffic between specific client and server IPs
4. Capture HTTP GET requests only
5. Save capture to a file for later analysis

<details>
<summary>Hint</summary>

tcpdump uses BPF (Berkeley Packet Filter) syntax for filtering. Interface is specified with `-i`. Filters use `host`, `port`, `tcp`, `src`, `dst` keywords. Write to file with `-w`. Read from file with `-r`.

</details>

### Part B: Interpret a TCP Handshake

Given this tcpdump output, trace the TCP three-way handshake:

```
14:23:01.001 IP 192.168.1.100.52344 > 10.0.1.10.80: Flags [S], seq 1000, win 65535
14:23:01.003 IP 10.0.1.10.80 > 192.168.1.100.52344: Flags [S.], seq 2000, ack 1001, win 65535
14:23:01.003 IP 192.168.1.100.52344 > 10.0.1.10.80: Flags [.], ack 2001, win 65535
14:23:01.004 IP 192.168.1.100.52344 > 10.0.1.10.80: Flags [P.], seq 1001:1500, ack 2001
14:23:01.006 IP 10.0.1.10.80 > 192.168.1.100.52344: Flags [.], ack 1500, win 65535
```

For each packet, identify:
1. The TCP flag(s) and their meaning
2. The sequence and acknowledgment numbers
3. What stage of the connection this represents

<details>
<summary>Hint</summary>

TCP flags: S=SYN (synchronize), S.=SYN-ACK (synchronize-acknowledge), .=ACK (acknowledge), P.=PSH-ACK (push-acknowledge). The three-way handshake is: SYN → SYN-ACK → ACK.

</details>

### Part C: Filter Expressions

Write tcpdump filter expressions for these scenarios:

1. Capture only SYN packets (connection initiation)
2. Capture only traffic from a specific subnet (192.168.1.0/24)
3. Capture DNS queries (UDP port 53)
4. Capture traffic that is NOT SSH (exclude port 22)
5. Capture TCP packets with the RST flag (connection resets)

<details>
<summary>Hint</summary>

Use `tcp[tcpflags]` to filter by TCP flags. Use `not` or `!` to exclude traffic. Use `net` for subnet filtering. Combine filters with `and`, `or`, `and not`.

</details>

## Success Criteria

- [ ] You can write tcpdump commands with interface, filter, and output options
- [ ] You can read and interpret TCP handshake packets
- [ ] You can write BPF filter expressions for common scenarios
- [ ] You understand TCP flag names and their meanings

## What You Should Understand After This Exercise

tcpdump is a powerful command-line packet capture tool. It captures raw network traffic and displays packet headers. Filters (BPF syntax) allow you to capture only relevant traffic. Understanding TCP flags (SYN, ACK, FIN, RST) and sequence numbers is essential for diagnosing connection issues at the packet level.
