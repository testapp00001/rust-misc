# Solution 01: Bridge vs Host vs Overlay Networks

## Part A: Completed Comparison Table

| Property | bridge | host | overlay |
|----------|--------|------|---------|
| Default network created by Docker? | Yes (the `bridge` network) | No | No |
| Containers get their own IP? | Yes, from the bridge subnet | No, uses the host's IP | Yes, from the overlay subnet |
| Containers can reach each other by name (DNS)? | Only on custom bridge, NOT on default bridge | N/A (no separate namespace) | Yes |
| Works across multiple hosts? | No | No | Yes |
| Isolates container from host network? | Yes | No | Yes |
| Performance overhead compared to native? | Low (virtual switch) | None (direct host access) | Moderate (VXLAN encapsulation) |
| Requires Docker Swarm? | No | No | Yes |

### Why This Matters

- **bridge** is your workhorse for single-host deployments. Custom bridge
  networks add DNS resolution, which is essential for multi-container apps.
- **host** gives maximum performance but zero isolation. Use it only when
  you need raw network speed or the container must see the host's network
  interfaces directly.
- **overlay** enables multi-host communication by wrapping container packets
  in VXLAN tunnels. The overhead comes from encapsulation and decapsulation
  at each end.

---

## Part B: Scenario-to-Driver Mapping

**1. Web app + database on a laptop**
Custom bridge. Both containers are on the same host and need to communicate.
A custom bridge gives them DNS resolution and isolation from the host.

**2. High-frequency trading application**
Host. The application needs the absolute lowest latency. Removing the
network namespace eliminates the virtual switch overhead. The trade-off is
that the container shares all host ports, which requires careful port
management.

**3. Microservices across three physical servers**
Overlay. Bridge networks are local to a single host. Overlay networks use
VXLAN tunnels to extend a single network across multiple hosts. This
requires Docker Swarm.

**4. CI/CD pipeline job with no network access**
`none`. The job downloads artifacts (which can be done before starting the
container via volume mounts) and should not be able to reach any network.
The `none` driver disables networking entirely. This is not bridge, host,
or overlay -- it is the null driver.

**5. Monitoring agent seeing all host traffic**
Host. The agent needs to see network interfaces and traffic on the host.
With bridge networking, the container has its own virtual interface and
cannot see host traffic. With host networking, the container sees everything
the host sees.

---

## Part C: The Default Bridge Trap

**1. Which network are both containers on?**
Both are on the default `bridge` network (the one named `bridge` that Docker
creates automatically).

**2. Why does `curl http://web:80` fail?**
The default bridge network does **not** have the embedded DNS server that
resolves container names. The `api` container cannot resolve the hostname
`web` to an IP address. You can verify this:

```bash
docker exec api getent hosts web
# No output -- DNS resolution fails
```

However, IP-based communication does work:

```bash
# Find web's IP
docker inspect web --format '{{.NetworkSettings.IPAddress}}'
# Output: 172.17.0.2

docker exec api curl http://172.17.0.2:80
# This works
```

**3. What would you change?**
Create a custom bridge network and run both containers on it:

```bash
docker network create app-net
docker run -d --name web --network app-net nginx:alpine
docker run -d --name api --network app-net python:3.12-slim sleep infinity

docker exec api curl http://web:80
# Now works -- custom bridge has embedded DNS
```

---

## Part D: When Would You Use Host Mode?

**1. Network monitoring and packet capture tools**
Tools like `tcpdump`, `wireshark`, or Prometheus node_exporter need to see
the host's network interfaces. With host networking, they can capture
traffic on `eth0`, `docker0`, and other interfaces.
Gain: Full visibility into host networking.
Lose: Cannot run multiple instances if they bind to the same port.

**2. Applications with dynamic port ranges**
Some applications (like FTP servers, SIP servers, or certain game servers)
open random ports for data channels. Mapping hundreds of ports with `-p`
is impractical. Host networking lets them use any port.
Gain: No port mapping management.
Lose: Port conflicts with other services on the host.

**3. Performance-critical applications**
Applications that process millions of small packets per second (trading
systems, real-time analytics) benefit from removing the virtual switch
overhead. Benchmarks show 5-15% improvement in throughput and latency.
Gain: Near-native network performance.
Lose: Complete loss of network isolation.

**4. Containers that need to bind to specific host interfaces**
A container running a DHCP server or a VPN endpoint needs to bind to
specific physical interfaces. Host networking gives it direct access.
Gain: Can use host interfaces directly.
Lose: Cannot run on multiple hosts without configuration changes.

---

## Common Mistakes

1. **Using the default bridge for multi-container apps.** Always create
   custom networks. The default bridge is a legacy feature.

2. **Confusing host networking with port mapping.** Port mapping (`-p`)
   exposes specific ports. Host networking removes the entire network
   namespace.

3. **Thinking overlay works without Swarm.** Overlay networks require
   Docker Swarm mode. You cannot use them with standalone Docker.

4. **Using host mode "for simplicity."** Host mode seems simpler because
   you do not need port mapping, but it creates port conflicts and
   security problems that are harder to debug than bridge networking
   issues.
