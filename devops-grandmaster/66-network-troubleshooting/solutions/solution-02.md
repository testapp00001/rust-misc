# Solution 02: tcpdump Fundamentals

## Part A: Basic Capture Commands

### 1. Capture all traffic on interface eth0

```bash
tcpdump -i eth0
```

### 2. Capture only TCP traffic to port 80

```bash
tcpdump -i eth0 tcp port 80
```

### 3. Capture traffic between specific client and server

```bash
tcpdump -i eth0 host 192.168.1.100 and host 10.0.1.10
```

Or more specifically for traffic between them:

```bash
tcpdump -i eth0 src 192.168.1.100 and dst 10.0.1.10
tcpdump -i eth0 src 10.0.1.10 and dst 192.168.1.100
```

### 4. Capture HTTP GET requests only

```bash
tcpdump -i eth0 -A -s 0 'tcp port 80 and (((ip[2:2] - ((ip[0]&0xf)<<2)) - ((tcp[12]&0xf0)>>2)) != 0)' | grep "GET "
```

Or more simply with tshark:

```bash
tshark -i eth0 -f "tcp port 80" -Y "http.request.method == GET"
```

### 5. Save capture to a file

```bash
tcpdump -i eth0 -w capture.pcap
# Later, read the file:
tcpdump -r capture.pcap
```

Or with a filter:

```bash
tcpdump -i eth0 -w http-traffic.pcap tcp port 80
```

### Common Options

| Option | Purpose |
|--------|---------|
| `-i eth0` | Capture on specific interface |
| `-w file.pcap` | Write raw packets to file |
| `-r file.pcap` | Read from capture file |
| `-A` | Print packets in ASCII |
| `-X` | Print packets in hex and ASCII |
| `-n` | Do not resolve hostnames (faster) |
| `-v`, `-vv`, `-vvv` | Increasing verbosity |
| `-c 100` | Capture only 100 packets |
| `-s 0` | Capture full packet (no truncation) |

## Part B: Interpret a TCP Handshake

### Packet 1: SYN

```
14:23:01.001 IP 192.168.1.100.52344 > 10.0.1.10.80: Flags [S], seq 1000, win 65535
```

- **Flag [S]:** SYN (Synchronize) - initiating a TCP connection
- **seq 1000:** Initial sequence number chosen by client
- **win 65535:** Client's receive window size (64KB)
- **Stage:** First step of three-way handshake (client → server)

### Packet 2: SYN-ACK

```
14:23:01.003 IP 10.0.1.10.80 > 192.168.1.100.52344: Flags [S.], seq 2000, ack 1001, win 65535
```

- **Flag [S.]:** SYN-ACK (Synchronize-Acknowledge) - server accepts connection
- **seq 2000:** Server's initial sequence number
- **ack 1001:** Acknowledges client's SYN (seq 1000 + 1 = 1001)
- **Stage:** Second step of three-way handshake (server → client)
- **RTT so far:** 2ms (0.003 - 0.001)

### Packet 3: ACK

```
14:23:01.003 IP 192.168.1.100.52344 > 10.0.1.10.80: Flags [.], ack 2001, win 65535
```

- **Flag [.]:** ACK (Acknowledge) - client confirms connection established
- **ack 2001:** Acknowledges server's SYN (seq 2000 + 1 = 2001)
- **Stage:** Third step of three-way handshake (client → server)
- **Connection established!** Both sides have synchronized sequence numbers.

### Packet 4: Data (HTTP Request)

```
14:23:01.004 IP 192.168.1.100.52344 > 10.0.1.10.80: Flags [P.], seq 1001:1500, ack 2001
```

- **Flag [P.]:** PSH-ACK (Push-Acknowledge) - sending data immediately
- **seq 1001:1500:** 499 bytes of data (sequence range)
- **PSH flag:** Tells receiver to deliver data to application immediately
  (do not buffer)
- **Stage:** HTTP GET request being sent

### Packet 5: Data Acknowledgment

```
14:23:01.006 IP 10.0.1.10.80 > 192.168.1.100.52344: Flags [.], ack 1500, win 65535
```

- **Flag [.]:** ACK - server acknowledges received data
- **ack 1500:** All data up to sequence 1500 has been received
- **Stage:** TCP acknowledgment of HTTP request

### TCP Flag Summary

| Flag | Symbol | Meaning |
|------|--------|---------|
| SYN | `[S]` | Synchronize sequence numbers (connection initiation) |
| SYN-ACK | `[S.]` | Acknowledge SYN (connection acceptance) |
| ACK | `[.]` | Acknowledge data receipt |
| PSH | `[P.]` | Push data to application immediately |
| FIN | `[F.]` | Finish (graceful connection close) |
| RST | `[R.]` | Reset (abort connection) |

## Part C: Filter Expressions

### 1. Capture only SYN packets

```bash
tcpdump -i eth0 'tcp[tcpflags] & (tcp-syn) != 0 and tcp[tcpflags] & (tcp-ack) == 0'
```

This captures packets with ONLY the SYN flag set (not SYN-ACK).

### 2. Capture traffic from a specific subnet

```bash
tcpdump -i eth0 src net 192.168.1.0/24
```

Or for both directions:

```bash
tcpdump -i eth0 net 192.168.1.0/24
```

### 3. Capture DNS queries

```bash
tcpdump -i eth0 udp port 53
```

Or for DNS over TCP (large responses):

```bash
tcpdump -i eth0 tcp port 53
```

### 4. Exclude SSH traffic

```bash
tcpdump -i eth0 not port 22
```

Or:

```bash
tcpdump -i eth0 'port != 22'
```

### 5. Capture TCP RST packets

```bash
tcpdump -i eth0 'tcp[tcpflags] & (tcp-rst) != 0'
```

### BPF Filter Syntax Reference

```
Operators:
  and / &&    Both conditions must match
  or  / ||    Either condition must match
  not / !     Negate the condition

Keywords:
  host        Match IP address
  net         Match subnet (e.g., 192.168.1.0/24)
  port        Match port number
  src         Source address/port
  dst         Destination address/port
  tcp         Match TCP protocol
  udp         Match UDP protocol
  icmp        Match ICMP protocol

TCP flag filters:
  tcp-syn     SYN flag
  tcp-ack     ACK flag
  tcp-fin     FIN flag
  tcp-rst     RST flag
  tcp-push    PSH flag
```

### Common Mistakes to Avoid

- **Not using `-n` for production captures.** DNS resolution during capture
  slows everything down and can cause packet drops. Always use `-n` in
  production.
- **Capturing on the wrong interface.** Use `ip addr` or `ifconfig` to
  list interfaces first. On Docker/Kubernetes, traffic may be on `docker0`,
  `cni0`, or `veth*` interfaces.
- **Forgetting `-s 0` for full captures.** By default, tcpdump truncates
  packets at 262144 bytes. Use `-s 0` to capture the full packet.
- **Not saving to file when debugging.** Terminal output loses data.
  Always save to a pcap file (`-w`) for later analysis with Wireshark.

## Key Takeaway

tcpdump captures raw network packets using BPF filters. Understanding TCP
flags (SYN, ACK, FIN, RST) and sequence numbers is essential for diagnosing
connection issues. The three-way handshake (SYN → SYN-ACK → ACK) establishes
connections. Always capture to a file for later analysis, and use `-n` to
avoid DNS resolution overhead.
