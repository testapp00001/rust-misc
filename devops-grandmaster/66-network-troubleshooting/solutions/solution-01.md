# Solution 01: Network Diagnostic Tools

## Part A: Tool Selection

| Question | Tool | Why |
|----------|------|-----|
| **Is the server reachable at all?** | `ping` | Sends ICMP echo requests. If you get replies, the server is reachable at the IP level. Fastest way to test basic connectivity. |
| **Where is the connection being dropped?** | `traceroute` | Shows the path packets take, hop by hop. Identifies where latency increases or packets are dropped. |
| **Is DNS resolving correctly?** | `dig` | Queries DNS servers directly. Shows the resolved IP, TTL, and authoritative nameservers. More detailed than nslookup. |
| **What ports are open on the server?** | `nmap` | Port scanner. Tests TCP and UDP ports. Shows open, closed, and filtered ports. Can detect services and OS. |
| **What does the actual TCP handshake look like?** | `tcpdump` | Captures raw packets. Shows TCP flags, sequence numbers, and timing. Essential for diagnosing connection-level issues. |
| **Are there established connections to the server?** | `ss` | Shows socket state (LISTEN, ESTAB, TIME_WAIT). Replaces netstat. Shows which processes have connections open. |

## Part B: Diagnostic Workflow

```
Step 1: C. Use dig to verify DNS resolution
  └─ If DNS fails, the user cannot resolve the hostname
  └─ Fix: check DNS server, domain configuration

Step 2: D. Use ping to test basic connectivity
  └─ If ping fails, the server is unreachable at IP level
  └─ Fix: check firewall, routing, server status

Step 3: F. Use traceroute to find where packets are dropped
  └─ If traceroute shows drops, there is a network path issue
  └─ Fix: contact ISP, check routing

Step 4: E. Use ss to check if the service is listening
  └─ If port is not LISTEN, the service is not running
  └─ Fix: start the service, check port configuration

Step 5: B. Use curl to test HTTP response
  └─ If curl fails, there is an application-level issue
  └─ Fix: check application logs, reverse proxy config

Step 6: A. Use tcpdump to capture the TCP handshake
  └─ If packets are malformed or unexpected, there is a protocol issue
  └─ Fix: check TLS configuration, proxy settings
```

### Why This Order Matters

This follows the **OSI model bottom-up**:
1. **DNS** (Application layer 7) - Can you resolve the name?
2. **Ping** (Network layer 3) - Can you reach the IP?
3. **Traceroute** (Network layer 3) - What path do packets take?
4. **ss** (Transport layer 4) - Is the port open?
5. **curl** (Application layer 7) - Does HTTP work?
6. **tcpdump** (Data link layer 2) - What is on the wire?

Starting with the most common and easiest-to-check issues saves time.
DNS issues are the most common cause of "cannot access" problems.

## Part C: Output Interpretation

### Output 1: ping

```
PING app.example.com (203.0.113.10) 56(84) bytes of data.
```
- DNS resolution works: `app.example.com` resolved to `203.0.113.10`
- Packet size: 56 bytes data, 84 bytes total (with IP + ICMP headers)

```
64 bytes from 203.0.113.10: icmp_seq=1 ttl=62 time=12.3 ms
```
- Server is reachable (got a reply)
- TTL=62 means the packet traversed ~2 routers (128 - 62 = 66 hops from initial TTL of 128... actually TTL is decremented per hop, so 128-62=66 hops, but typical initial TTL is 64 or 128)
- RTT = 12.3ms (round-trip time)

```
3 packets transmitted, 3 received, 0% packet loss
rtt min/avg/max/mdev = 11.8/12.1/12.3/0.2 ms
```
- 0% packet loss: good connectivity
- Average latency: 12.1ms with 0.2ms jitter: very stable connection

### Output 2: ss

```
LISTEN    0    128    0.0.0.0:80     0.0.0.0:*
LISTEN    0    128    0.0.0.0:443    0.0.0.0:*
```
- Port 80 (HTTP) is listening on all interfaces (0.0.0.0)
- Port 443 (HTTPS) is listening on all interfaces
- Recv-Q=0: no pending connections
- Send-Q=128: listen backlog of 128 connections

```
ESTAB    0    0    10.0.1.10:443    192.168.1.100:52344
ESTAB    0    0    10.0.1.10:443    192.168.1.101:52345
```
- Two established HTTPS connections
- Recv-Q=0, Send-Q=0: no data waiting to be sent/received (healthy)
- Clients: 192.168.1.100 and 192.168.1.101

### Common Mistakes to Avoid

- **Starting with tcpdump.** tcpdump produces a lot of data. Start with
  simpler tools (ping, dig) to narrow down the issue first.
- **Ignoring DNS.** Many "connectivity" issues are actually DNS issues.
  Always verify DNS resolution first.
- **Using ping to test TCP services.** Ping uses ICMP, not TCP. A server
  can respond to ping but have its TCP port closed. Use `nc` or `curl`
  to test TCP connectivity.
- **Not checking both directions.** Traffic from client to server may work,
  but traffic from server to client may be blocked (asymmetric routing).
  Check both directions.

## Key Takeaway

Network troubleshooting follows a systematic bottom-up approach: DNS first,
then connectivity, then path, then ports, then protocol, then packets.
Each tool has a specific purpose. Using the right tool at the right time
saves hours of debugging. The most common mistake is starting with the
most complex tool (tcpdump) instead of the simplest (ping).
