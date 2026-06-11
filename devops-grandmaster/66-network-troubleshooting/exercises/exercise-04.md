# Exercise 04: Packet Analysis with Wireshark

**Type:** Challenge
**Time:** 45 min
**Difficulty:** Medium-Hard

## Objective

Analyze packet captures to diagnose network issues, understand protocol behavior, and identify performance problems at the packet level.

## Scenario

You have captured a packet trace of a slow HTTP connection. The client reports that downloading a 1MB file takes 30 seconds when it should take 2 seconds. The capture file contains the following sequence:

```
Packet #  Time      Source              Dest                Protocol  Info
1         0.000     192.168.1.100       10.0.1.10           TCP       52344→80 [SYN]
2         0.025     10.0.1.10           192.168.1.100       TCP       80→52344 [SYN, ACK]
3         0.025     192.168.1.100       10.0.1.10           TCP       52344→80 [ACK]
4         0.026     192.168.1.100       10.0.1.10           HTTP      GET /file.bin HTTP/1.1
5         0.050     10.0.1.10           192.168.1.100       HTTP      HTTP/1.1 200 OK
6         0.050     10.0.1.10           192.168.1.100       TCP       [TCP segment] 1460 bytes
7         0.051     10.0.1.10           192.168.1.100       TCP       [TCP segment] 1460 bytes
8         0.052     10.0.1.10           192.168.1.100       TCP       [TCP segment] 1460 bytes
9         0.550     192.168.1.100       10.0.1.10           TCP       [TCP Dup ACK] ACK 4381
10        1.050     10.0.1.10           192.168.1.100       TCP       [TCP Retransmission] 1460 bytes
11        1.051     10.0.1.10           192.168.1.100       TCP       [TCP segment] 1460 bytes
...
```

## Tasks

### Part A: Connection Analysis

Analyze the TCP handshake (packets 1-3):

1. What is the round-trip time (RTT)?
2. Is there any issue with the handshake?
3. What is the client's initial window size?

<details>
<summary>Hint</summary>

RTT = time between SYN (packet 1) and SYN-ACK (packet 2). The handshake itself looks normal. Initial window size is in the TCP options of the SYN packet.

</details>

### Part B: Data Transfer Analysis

Analyze the data transfer (packets 5-11):

1. What is the initial congestion window?
2. What happens at packet 9 (TCP Dup ACK)?
3. Why does packet 10 say "Retransmission"?
4. How many bytes were transferred in the first second?

<details>
<summary>Hint</summary>

Packets 6-8 show 3 segments sent immediately (congestion window = 3 segments). A duplicate ACK means the receiver got packets out of order. Retransmission means the sender re-sent a packet that was lost. Calculate bytes: segments × 1460 bytes.

</details>

### Part C: Performance Diagnosis

Based on the packet capture, explain why the 1MB file takes 30 seconds:

1. What is the effective throughput?
2. What is causing the low throughput?
3. What TCP parameter could be tuned to improve performance?

<details>
<summary>Hint</summary>

1MB = 1,048,576 bytes. At 1460 bytes per segment, that is ~718 segments. If the congestion window is small (3 segments) and packet loss causes retransmissions, throughput drops dramatically. TCP slow start and congestion control are the mechanisms at play.

</details>

### Part D: Packet Capture Commands

Write the tcpdump or tshark commands to:

1. Capture only HTTP traffic and display the request/response headers
2. Capture traffic and show TCP window sizes
3. Filter for retransmissions only
4. Capture the first 100 packets on port 80

<details>
<summary>Hint</summary>

tshark (Wireshark's CLI) has more display options than tcpdump. Use `-T fields` to extract specific fields. Use `-Y` for display filters (Wireshark syntax) and `-f` for capture filters (BPF syntax).

</details>

## Success Criteria

- [ ] You can analyze TCP handshake timing from packet captures
- [ ] You can identify packet loss, retransmissions, and duplicate ACKs
- [ ] You can diagnose throughput issues from packet-level data
- [ ] You can write capture and display filter commands

## What You Should Understand After This Exercise

Packet analysis reveals what happens at the wire level. TCP congestion control starts with a small window and grows as packets are acknowledged. Packet loss causes retransmissions and window reduction, dramatically reducing throughput. A small congestion window combined with high latency creates a "bandwidth-delay product" bottleneck. Understanding these mechanics is essential for diagnosing performance issues that are invisible at the application level.
