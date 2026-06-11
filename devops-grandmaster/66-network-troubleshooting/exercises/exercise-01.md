# Exercise 01: Network Diagnostic Tools

**Type:** Conceptual
**Time:** 20 min
**Difficulty:** Easy

## Objective

Understand the purpose and use case of common network diagnostic tools and know when to use each one.

## Scenario

You are troubleshooting a connectivity issue. A user reports: "I cannot access the web application at https://app.example.com." You have access to these tools:

```
Available tools:
├── ping
├── traceroute / tracepath
├── dig / nslookup
├── curl / wget
├── tcpdump
├── ss / netstat
├── nmap
└── ip / ifconfig
```

## Tasks

### Part A: Tool Selection

For each diagnostic question, select the best tool and explain why:

| Question | Tool | Why |
|----------|------|-----|
| Is the server reachable at all? | | |
| Where is the connection being dropped? | | |
| Is DNS resolving correctly? | | |
| What ports are open on the server? | | |
| What does the actual TCP handshake look like? | | |
| Are there established connections to the server? | | |

<details>
<summary>Hint</summary>

ping tests basic reachability. traceroute shows the path. dig checks DNS. nmap scans ports. tcpdump captures packets. ss shows socket state.

</details>

### Part B: Diagnostic Workflow

Arrange these diagnostic steps in the correct order for troubleshooting "cannot access web application":

```
Step 1: _______________
Step 2: _______________
Step 3: _______________
Step 4: _______________
Step 5: _______________
Step 6: _______________

Available steps:
A. Use tcpdump to capture the TCP handshake
B. Use curl to test HTTP response
C. Use dig to verify DNS resolution
D. Use ping to test basic connectivity
E. Use ss to check if the service is listening
F. Use traceroute to find where packets are dropped
```

<details>
<summary>Hint</summary>

Start from the bottom of the network stack and work up. DNS first (can you resolve the name?), then connectivity (can you reach the IP?), then path (where do packets go?), then port (is the service listening?), then protocol (does HTTP work?), then packet-level (what is happening at the wire level?).

</details>

### Part C: Output Interpretation

For each tool output, identify what it tells you about the network:

**Output 1: ping**
```
PING app.example.com (203.0.113.10) 56(84) bytes of data.
64 bytes from 203.0.113.10: icmp_seq=1 ttl=62 time=12.3 ms
64 bytes from 203.0.113.10: icmp_seq=2 ttl=62 time=11.8 ms
64 bytes from 203.0.113.10: icmp_seq=3 ttl=62 time=12.1 ms
--- app.example.com ping statistics ---
3 packets transmitted, 3 received, 0% packet loss, time 2003ms
rtt min/avg/max/mdev = 11.8/12.1/12.3/0.2 ms
```

**Output 2: ss**
```
State     Recv-Q    Send-Q       Local Address:Port       Peer Address:Port
LISTEN    0         128                0.0.0.0:80              0.0.0.0:*
LISTEN    0         128                0.0.0.0:443             0.0.0.0:*
ESTAB     0         0           10.0.1.10:443        192.168.1.100:52344
ESTAB     0         0           10.0.1.10:443        192.168.1.101:52345
```

<details>
<summary>Hint</summary>

ping output shows: IP resolution, round-trip time (latency), packet loss, and TTL. ss output shows: listening ports, connection state, and connected clients.

</details>

## Success Criteria

- [ ] You can select the right diagnostic tool for each question
- [ ] You can order diagnostic steps from bottom-up (layer 1 to layer 7)
- [ ] You can interpret ping and ss output
- [ ] You understand the diagnostic workflow: DNS → connectivity → path → port → protocol → packets

## What You Should Understand After This Exercise

Network troubleshooting follows a systematic bottom-up approach: start with DNS resolution, then test basic connectivity, then trace the path, then check ports, then test the protocol, and finally capture packets for detailed analysis. Each tool has a specific purpose, and using the right tool at the right time saves hours of debugging.
