# 66 - Network Troubleshooting

## tcpdump, traceroute, Packet Analysis

> **Previous:** [65 - Service Mesh](../65-service-mesh/README.md) | **Next:** [67 - Auto Scaling](../67-auto-scaling/README.md)

Network troubleshooting is the systematic process of diagnosing and resolving connectivity issues between services. When a request fails, the problem could be at any layer of the network stack -- from DNS resolution to application protocol. You need tools and a methodology to isolate the failure quickly.

---

## Problem

A user reports: "The app is slow." Where do you start?

```
Possible failure points (request path):

User's Browser
  -> DNS Resolution
    -> CDN / Load Balancer
      -> Ingress Controller
        -> Service Mesh (if present)
          -> Kubernetes Service
            -> Pod Network (CNI)
              -> Container Network Namespace
                -> Application Process
                  -> Application Logic
                    -> Database Connection
                      -> Database
```

Without the right tools and methodology, debugging network issues is guesswork. "It works on my machine" is not a diagnosis.

---

## Naive Way: Ping and Pray

```bash
# The classic "debugging" approach
ping google.com
# "It works" -> ???
# "It doesn't work" -> ???

# Or worse:
curl https://api.internal/endpoint
# "Connection refused"
# "Now what?"
```

Problems:
- Ping only tests ICMP, not TCP connectivity
- No visibility into what is happening at the packet level
- No understanding of which layer is failing
- Cannot diagnose intermittent issues
- No methodology to isolate the problem

---

## Right Way: The OSI Model and Systematic Debugging

### The OSI Model for Troubleshooting

```
Layer 7: Application   -- HTTP status codes, DNS resolution, app errors
Layer 6: Presentation  -- TLS/SSL handshake, encoding issues
Layer 5: Session        -- Connection state, session management
Layer 4: Transport      -- TCP connection, ports, SYN/ACK
Layer 3: Network        -- IP routing, ICMP, path MTU
Layer 2: Data Link      -- MAC addresses, ARP, VLANs
Layer 1: Physical       -- Cables, NICs, link status

Debug bottom-up:
  1. Is the link up? (Layer 1-2)
  2. Can you route to the destination? (Layer 3)
  3. Can you establish a TCP connection? (Layer 4)
  4. Is TLS working? (Layer 5-6)
  5. Is the application responding correctly? (Layer 7)
```

### tcpdump: Packet-Level Analysis

tcpdump captures raw network packets. It is the ultimate network debugging tool -- if the packets are not reaching the destination, no amount of application debugging will help.

```bash
# Basic capture on an interface
tcpdump -i eth0

# Capture HTTP traffic on port 80
tcpdump -i eth0 port 80

# Capture traffic to/from a specific host
tcpdump -i eth0 host 10.0.1.50

# Capture TCP SYN packets (new connections)
tcpdump -i eth0 'tcp[tcpflags] & (tcp-syn) != 0'

# Capture DNS queries
tcpdump -i eth0 port 53

# Capture with full packet content (hex + ASCII)
tcpdump -i eth0 -X port 8080

# Write to file for analysis in Wireshark
tcpdump -i eth0 -w capture.pcap port 80

# Read from file
tcpdump -r capture.pcap

# Capture only TCP RST packets (connection resets)
tcpdump -i eth0 'tcp[tcpflags] & (tcp-rst) != 0'

# Capture with verbose output and timestamps
tcpdump -i eth0 -tttt -v port 443

# Capture on a specific Kubernetes pod's network namespace
# First, find the pod's network namespace
PID=$(docker inspect --format '{{.State.Pid}}' $(crictl ps --label io.kubernetes.pod.name=myapp -q | head -1))
nsenter -t $PID -n tcpdump -i eth0 port 8080
```

### Reading tcpdump Output

```
14:23:01.123456 IP 10.0.1.10.45678 > 10.0.2.20.8080: Flags [S], seq 123456789, win 65535, options [mss 1460,sackOK,TS val 123 ecr 0,nop,wscale 7], length 0
14:23:01.123789 IP 10.0.2.20.8080 > 10.0.1.10.45678: Flags [S.], seq 987654321, ack 123456790, win 65535, options [mss 1460,sackOK,TS val 456 ecr 123,nop,wscale 7], length 0
14:23:01.124000 IP 10.0.1.10.45678 > 10.0.2.20.8080: Flags [.], ack 987654322, win 512, length 0

Flags:
  [S]   = SYN (connection initiation)
  [S.]  = SYN-ACK (connection acknowledgment)
  [.]   = ACK (data acknowledgment)
  [P.]  = PSH-ACK (push data)
  [F.]  = FIN-ACK (connection close)
  [R]   = RST (connection reset -- PROBLEM!)

A normal TCP handshake:
  Client -> Server:  [S]     (I want to connect)
  Server -> Client:  [S.]    (OK, I accept)
  Client -> Server:  [.]     (Great, let's go)

If you see [R] instead of [S.], the server is refusing the connection.
If you see nothing at all, packets are being dropped (firewall? routing?).
```

### traceroute and mtr

```bash
# traceroute: show the path packets take to a destination
traceroute google.com

# Output:
#  1  gateway (10.0.0.1)  1.234 ms  1.123 ms  1.089 ms
#  2  10.0.1.1 (10.0.1.1)  5.678 ms  5.456 ms  5.234 ms
#  3  isp-router (203.0.113.1)  10.123 ms  10.012 ms  9.890 ms
#  4  * * *  (firewall blocking ICMP)
#  5  destination (142.250.80.46)  15.678 ms  15.456 ms  15.234 ms

# mtr: continuous traceroute with statistics
mtr --report google.com

# Output:
# Host                  Loss%  Snt   Last  Avg   Best  Wrst  StDev
# 1. gateway            0.0%   100   1.2   1.3   1.0   2.1   0.3
# 2. 10.0.1.1          0.0%   100   5.4   5.5   5.0   6.2   0.4
# 3. isp-router         2.0%   100   10.1  10.3  9.8   12.1  0.8
# 4. ???                100.0  100   0.0   0.0   0.0   0.0   0.0
# 5. destination        0.0%   100   15.4  15.5  15.0  16.2  0.4

# mtr with TCP (bypasses ICMP blocks)
mtr --tcp --port 80 --report google.com

# mtr for Kubernetes pod-to-pod debugging
mtr --report 10.244.1.50
```

### netstat and ss

```bash
# netstat: show network connections (deprecated, use ss)
netstat -tlnp

# ss: modern replacement for netstat
# Show all listening TCP ports
ss -tlnp

# Output:
# State   Recv-Q  Send-Q  Local Address:Port  Peer Address:Port  Process
# LISTEN  0       128     0.0.0.0:8080        0.0.0.0:*          users:(("myapp",pid=1234,fd=3))
# LISTEN  0       128     0.0.0.0:22          0.0.0.0:*          users:(("sshd",pid=567,fd=3))

# Show all established connections
ss -tnp

# Show connections to a specific port
ss -tnp dport = :8080

# Show connections from a specific source
ss -tnp src = 10.0.1.10

# Show TCP socket statistics
ss -s

# Show socket memory usage
ss -tm

# Show connections in TIME_WAIT state (connection recently closed)
ss -t state time-wait

# Kubernetes: exec into pod and check its connections
kubectl exec -it myapp-pod -- ss -tnp
```

### curl for HTTP Debugging

```bash
# Basic request with verbose output
curl -v http://api.internal:8080/health

# Output:
# *   Trying 10.0.1.50:8080...
# * Connected to api.internal (10.0.1.50) port 8080
# > GET /health HTTP/1.1
# > Host: api.internal:8080
# > User-Agent: curl/8.0
# >
# < HTTP/1.1 200 OK
# < Content-Type: application/json
# < Content-Length: 15
# <
# {"status":"ok"}

# Debug TLS handshake
curl -v https://api.internal/health

# Show only headers
curl -I http://api.internal:8080/health

# Follow redirects
curl -L http://api.internal/redirect

# Set timeout
curl --connect-timeout 5 --max-time 10 http://api.internal:8080/slow

# Test with specific headers
curl -H "Authorization: Bearer $TOKEN" http://api.internal:8080/protected

# Measure request timing
curl -o /dev/null -s -w "DNS: %{time_namelookup}s\nConnect: %{time_connect}s\nTLS: %{time_appconnect}s\nTTFB: %{time_starttransfer}s\nTotal: %{time_total}s\n" https://api.internal/health

# Resolve to a specific IP (bypass DNS)
curl --resolve api.internal:8080:10.0.1.50 http://api.internal:8080/health

# Test from inside a Kubernetes pod
kubectl exec -it debug-pod -- curl -v http://myapp-service:8080/health
```

### DNS Debugging

```bash
# Resolve a domain name
dig api.internal

# Query specific DNS server
dig @10.0.0.10 api.internal

# Query specific record type
dig api.internal A
dig api.internal AAAA
dig api.internal CNAME
dig api.internal MX
dig api.internal SRV

# Trace the full DNS resolution path
dig +trace api.internal

# Reverse DNS lookup
dig -x 10.0.1.50

# Check Kubernetes CoreDNS
kubectl exec -it debug-pod -- nslookup kubernetes.default
kubectl exec -it debug-pod -- nslookup myapp-service.production.svc.cluster.local

# Debug DNS from within a pod
kubectl exec -it debug-pod -- cat /etc/resolv.conf
# Should show:
# nameserver 10.96.0.10
# search production.svc.cluster.local svc.cluster.local cluster.local

# Test DNS resolution from a pod
kubectl run dns-test --image=busybox:1.36 --rm -it --restart=Never -- nslookup myapp-service
```

### Firewall Debugging

```bash
# iptables: show all rules
iptables -L -n -v

# Show NAT rules
iptables -t nat -L -n -v

# Check if a specific port is allowed
iptables -L -n -v | grep 8080

# nftables (modern replacement)
nft list ruleset

# Kubernetes NetworkPolicy debugging
# Check if NetworkPolicy is blocking traffic
kubectl get networkpolicy -n production -o yaml

# Test connectivity from a pod
kubectl exec -it debug-pod -- nc -zv myapp-service 8080

# Check conntrack (connection tracking table)
conntrack -L | grep 8080

# Check if packets are being dropped by the kernel
cat /proc/net/snmp | grep -i drop
dmesg | grep -i drop
```

### Kubernetes-Specific Network Debugging

```bash
# 1. Check if the pod is running
kubectl get pods -n production -o wide

# 2. Check pod IP and node
kubectl get pod myapp-pod -n production -o jsonpath='{.status.podIP}'
kubectl get pod myapp-pod -n production -o jsonpath='{.spec.nodeName}'

# 3. Check service endpoints
kubectl get endpoints myapp-service -n production
# If endpoints are empty, the service selector does not match any pods

# 4. Check if the service is reachable from within the cluster
kubectl run debug --image=nicolaka/netshoot --rm -it --restart=Never -- bash
# Inside the debug pod:
curl -v http://myapp-service.production.svc.cluster.local:8080/health
traceroute myapp-service.production.svc.cluster.local
tcpdump -i eth0 port 8080

# 5. Check CoreDNS
kubectl logs -n kube-system -l k8s-app=kube-dns

# 6. Check CNI plugin
kubectl get pods -n kube-system -l k8s-app=cilium  # or calico, flannel, etc.

# 7. Check kube-proxy / iptables rules
iptables -t nat -L KUBE-SERVICES -n -v | grep myapp-service

# 8. Check if the node can reach the pod
# SSH to the node, then:
ping <pod-ip>
curl http://<pod-ip>:8080/health

# 9. Check MTU issues
ping -M do -s 1472 <destination-ip>  # Should work if MTU is 1500

# 10. Check for IP exhaustion
kubectl get pods --all-namespaces -o json | jq '.items | length'
```

---

## Production Way: Systematic Network Debugging Playbook

### The Debug Flowchart

```
Problem: Service A cannot reach Service B
                |
                v
+-------------------------------+
| 1. Is Service B running?      |
| kubectl get pods -l app=B     |
+-------------------------------+
         |              |
        YES             NO --> Check deployment, events, logs
         |
         v
+-------------------------------+
| 2. Does Service B have an IP? |
| kubectl get endpoints B       |
+-------------------------------+
         |              |
        YES             NO --> Service selector does not match pod labels
         |
         v
+-------------------------------+
| 3. Can you DNS-resolve B?     |
| nslookup B from A's pod       |
+-------------------------------+
         |              |
        YES             NO --> CoreDNS issue, check /etc/resolv.conf
         |
         v
+-------------------------------+
| 4. Can you reach B's IP?      |
| curl http://B-IP:port/health  |
| from A's pod                  |
+-------------------------------+
         |              |
        YES             NO --> NetworkPolicy, firewall, routing
         |
         v
+-------------------------------+
| 5. Can you reach B's service? |
| curl http://B.namespace:port  |
+-------------------------------+
         |              |
        YES             NO --> kube-proxy / iptables issue
         |
         v
+-------------------------------+
| 6. Is the application         |
| responding correctly?         |
| Check app logs, error codes   |
+-------------------------------+
```

### Automated Network Diagnostic Script

```bash
#!/bin/bash
# network-diag.sh - Systematic network diagnostics for Kubernetes
set -euo pipefail

SOURCE_POD="${1:?Usage: $0 <source-pod> <target-service> <port>}"
TARGET_SVC="${2:?}"
TARGET_PORT="${3:?}"
NAMESPACE="${4:-default}"

echo "=== Network Diagnostics ==="
echo "Source: $SOURCE_POD"
echo "Target: $TARGET_SVC:$TARGET_PORT"
echo "Namespace: $NAMESPACE"
echo ""

# Step 1: Pod status
echo "--- Step 1: Pod Status ---"
kubectl get pod "$SOURCE_POD" -n "$NAMESPACE" -o wide
echo ""

# Step 2: Target service endpoints
echo "--- Step 2: Service Endpoints ---"
kubectl get endpoints "$TARGET_SVC" -n "$NAMESPACE"
echo ""

# Step 3: DNS resolution
echo "--- Step 3: DNS Resolution ---"
kubectl exec "$SOURCE_POD" -n "$NAMESPACE" -- \
    nslookup "$TARGET_SVC" 2>&1 || echo "DNS FAILED"
echo ""

# Step 4: TCP connectivity
echo "--- Step 4: TCP Connectivity ---"
kubectl exec "$SOURCE_POD" -n "$NAMESPACE" -- \
    nc -zv "$TARGET_SVC" "$TARGET_PORT" 2>&1 || echo "TCP FAILED"
echo ""

# Step 5: HTTP request
echo "--- Step 5: HTTP Request ---"
kubectl exec "$SOURCE_POD" -n "$NAMESPACE" -- \
    curl -s -o /dev/null -w "HTTP %{http_code} in %{time_total}s\n" \
    "http://$TARGET_SVC:$TARGET_PORT/health" 2>&1 || echo "HTTP FAILED"
echo ""

# Step 6: Network policies
echo "--- Step 6: Network Policies ---"
kubectl get networkpolicy -n "$NAMESPACE" -o wide
echo ""

# Step 7: Recent events
echo "--- Step 7: Recent Events ---"
kubectl get events -n "$NAMESPACE" --sort-by='.lastTimestamp' | tail -10
echo ""

echo "=== Diagnostics Complete ==="
```

### Packet Capture for Intermittent Issues

```bash
# Capture packets for analysis of intermittent failures
# Start a capture in the background, wait for the issue to occur

# On the source pod
kubectl exec -it "$SOURCE_POD" -- sh -c '
  tcpdump -i eth0 -w /tmp/capture.pcap port '"$TARGET_PORT"' &
  TCPDUMP_PID=$!
  sleep 300  # Capture for 5 minutes
  kill $TCPDUMP_PID
  # Copy the capture file out
'

# Copy the capture file for analysis
kubectl cp "$SOURCE_POD":/tmp/capture.pcap ./capture.pcap

# Analyze with tshark (Wireshark CLI)
tshark -r capture.pcap -Y "tcp.analysis.retransmission" | head -20
tshark -r capture.pcap -Y "tcp.analysis.reset" | head -20
tshark -r capture.pcap -q -z io,stat,1  # Traffic rate per second
```

### DNS Debugging in Depth

```bash
# Check CoreDNS configuration
kubectl get configmap coredns -n kube-system -o yaml

# Common CoreDNS issues:
# 1. Forward loop (CoreDNS forwarding to itself)
# 2. Upstream DNS server unreachable
# 3. Custom DNS entries missing

# Check CoreDNS logs for errors
kubectl logs -n kube-system -l k8s-app=kube-dns --tail=100 | grep -i error

# Test external DNS resolution
kubectl exec -it debug-pod -- nslookup google.com

# Test internal service DNS
kubectl exec -it debug-pod -- nslookup kubernetes.default.svc.cluster.local
kubectl exec -it debug-pod -- nslookup myapp.production.svc.cluster.local

# Check ndots configuration (can cause excessive DNS queries)
kubectl exec -it debug-pod -- cat /etc/resolv.conf
# If ndots:5, every name with fewer than 5 dots gets search domain appended
# This can cause 5+ DNS queries for a simple lookup
```

### TCP Connection Debugging

```bash
# Check for SYN backlog overflow (connection queue full)
ss -lnt | grep 8080
# Recv-Q > 0 means connections are queuing up

# Check for TIME_WAIT accumulation
ss -s | grep time-wait
# High TIME_WAIT means lots of short-lived connections

# Check TCP retransmissions
cat /proc/net/snmp | grep Tcp
# Look for RetransSegs -- high count indicates network issues

# Check for connection resets
ss -tnp state established | wc -l
ss -tnp state close-wait | wc -l  # Stale connections

# Trace TCP connection establishment
strace -e trace=connect -f -p $(pgrep myapp) 2>&1 | head -20
```

---

## Hands-On Lab

### Lab: Debug a Network Issue in Kubernetes

#### Part 1: Set Up the Scenario

```yaml
# Create a broken networking scenario
# deploy-broken.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: netlab
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: backend
  namespace: netlab
spec:
  replicas: 2
  selector:
    matchLabels:
      app: backend
  template:
    metadata:
      labels:
        app: backend
    spec:
      containers:
        - name: backend
          image: nginx:alpine
          ports:
            - containerPort: 80
---
apiVersion: v1
kind: Service
metadata:
  name: backend-svc
  namespace: netlab
spec:
  selector:
    app: backend-wrong  # WRONG selector -- intentionally broken
  ports:
    - port: 80
      targetPort: 80
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: frontend
  namespace: netlab
spec:
  replicas: 1
  selector:
    matchLabels:
      app: frontend
  template:
    metadata:
      labels:
        app: frontend
    spec:
      containers:
        - name: frontend
          image: busybox:1.36
          command: ["sleep", "3600"]
---
# NetworkPolicy blocking traffic
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: deny-from-frontend
  namespace: netlab
spec:
  podSelector:
    matchLabels:
      app: backend
  policyTypes:
    - Ingress
  ingress: []  # Deny all ingress
```

```bash
kubectl apply -f deploy-broken.yaml
```

#### Part 2: Diagnose the Issues

```bash
# Issue 1: Service selector mismatch
# Try to reach the backend
kubectl exec -n netlab deploy/frontend -- wget -qO- http://backend-svc:80/
# Expected: Connection refused or timeout

# Check endpoints
kubectl get endpoints backend-svc -n netlab
# Expected: <none> -- no endpoints means selector mismatch

# Debug: compare service selector with pod labels
kubectl get svc backend-svc -n netlab -o jsonpath='{.spec.selector}'
kubectl get pods -n netlab --show-labels
# Fix: change selector from "backend-wrong" to "backend"

# Issue 2: NetworkPolicy blocking traffic
# After fixing the selector, test again
kubectl exec -n netlab deploy/frontend -- wget -qO- http://backend-svc:80/
# Expected: still timeout

# Check network policies
kubectl get networkpolicy -n netlab
kubectl describe networkpolicy deny-from-frontend -n netlab

# Debug: temporarily delete the policy
kubectl delete networkpolicy deny-from-frontend -n netlab

# Test again
kubectl exec -n netlab deploy/frontend -- wget -qO- http://backend-svc:80/
# Expected: success
```

#### Part 3: Use tcpdump to Verify Traffic

```bash
# Get backend pod name
BACKEND_POD=$(kubectl get pod -n netlab -l app=backend -o jsonpath='{.items[0].metadata.name}')

# Start packet capture on the backend pod
kubectl exec -n netlab "$BACKEND_POD" -- sh -c 'apk add tcpdump && tcpdump -i eth0 port 80 -c 10' &
sleep 2

# Send a request from the frontend
kubectl exec -n netlab deploy/frontend -- wget -qO- http://backend-svc:80/

# You should see the packets in the tcpdump output
```

#### Part 4: DNS Debugging

```bash
# Check DNS resolution from the frontend pod
kubectl exec -n netlab deploy/frontend -- nslookup backend-svc
kubectl exec -n netlab deploy/frontend -- nslookup backend-svc.netlab.svc.cluster.local

# Check CoreDNS logs
kubectl logs -n kube-system -l k8s-app=kube-dns --tail=20

# Check the pod's resolv.conf
kubectl exec -n netlab deploy/frontend -- cat /etc/resolv.conf
```

#### Part 5: Build a Diagnostic Toolkit

```bash
# Deploy a persistent debug pod
cat <<EOF | kubectl apply -f -
apiVersion: v1
kind: Pod
metadata:
  name: netdebug
  namespace: netlab
spec:
  containers:
    - name: debug
      image: nicolaka/netshoot:latest
      command: ["sleep", "infinity"]
      securityContext:
        capabilities:
          add: ["NET_ADMIN", "NET_RAW"]
EOF

# Use the debug pod for all subsequent diagnostics
kubectl exec -it netdebug -n netlab -- bash

# Inside the debug pod:
# DNS tests
dig backend-svc.netlab.svc.cluster.local
nslookup backend-svc

# TCP tests
nc -zv backend-svc 80
telnet backend-svc 80

# HTTP tests
curl -v http://backend-svc/
curl -w "DNS: %{time_namelookup}s\nConnect: %{time_connect}s\nTTFB: %{time_starttransfer}s\nTotal: %{time_total}s\n" -o /dev/null -s http://backend-svc/

# Packet capture
tcpdump -i eth0 port 80 -c 20

# Route debugging
ip route show
ip route get 10.244.1.50
traceroute backend-svc.netlab.svc.cluster.local
```

---

## Common Network Issues and Solutions

```
| Symptom                    | Likely Cause                  | Fix                              |
|----------------------------|-------------------------------|----------------------------------|
| Connection refused         | Pod not running / wrong port  | Check pod status, service port   |
| Connection timeout         | NetworkPolicy / firewall      | Check network policies           |
| DNS resolution failed      | CoreDNS down / wrong search   | Check CoreDNS pods, resolv.conf  |
| Intermittent timeouts      | Pod scaling / readiness probe | Check readiness probes           |
| Connection reset           | App crash / OOM               | Check pod logs, resource limits  |
| Slow response              | CPU throttling / network RTT  | Check resource limits, node loc  |
| 502 Bad Gateway            | Upstream not ready            | Check upstream health            |
| TLS handshake failure      | Certificate issue             | Check cert expiry, CA trust      |
| MTU issues (large packets) | CNI / overlay network         | Check MTU on interfaces          |
| Ephemeral port exhaustion  | Too many connections          | Increase ip_local_port_range     |
```

---

## Limitation

Network troubleshooting tools tell you what is happening on the wire but do not solve the underlying capacity problem. If your services are struggling under load -- connection timeouts due to saturated bandwidth, dropped packets from overloaded NICs, or slow responses from CPU-starved pods -- you need to scale. Diagnosing traffic spikes and scaling to meet demand is the next challenge.

---

## Next Topic

[67 - Auto Scaling](../67-auto-scaling/README.md) -- Horizontal and vertical pod autoscaling, cluster autoscaling, and handling traffic spikes automatically.
