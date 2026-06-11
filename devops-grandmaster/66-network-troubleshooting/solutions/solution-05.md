# Solution 05: Production Network Troubleshooting

## Part A: Systematic Diagnosis

### Step-by-Step Procedure

```
Step 1: Check cluster-wide status
  Tool: kubectl
  Command: kubectl get nodes; kubectl get pods -A -o wide
  Looking for: Node status (Ready/NotReady), pod distribution, which node hosts affected pods

Step 2: Check node-7 status
  Tool: kubectl
  Command: kubectl describe node node-7
  Looking for: Conditions (Ready, MemoryPressure, DiskPressure), Events, capacity/allocatable

Step 3: Check affected pod logs
  Tool: kubectl
  Command: kubectl logs <affected-pod> -n production --tail=100
  Looking for: Connection timeout errors, database errors, stack traces

Step 4: Test basic connectivity from affected pod
  Tool: ping (inside pod)
  Command: kubectl exec -it <pod> -- ping -c 3 10.0.2.10
  Looking for: Is the database reachable at IP level?

Step 5: Test TCP connectivity to database port
  Tool: nc (netcat) inside pod
  Command: kubectl exec -it <pod> -- nc -zv 10.0.2.10 5432 -w 5
  Looking for: Can the pod connect to port 5432?

Step 6: Capture packets on the pod's interface
  Tool: tcpdump
  Command: kubectl exec -it <pod> -- tcpdump -i eth0 -n -c 50 host 10.0.2.10 and port 5432
  Looking for: SYN packets, SYN-ACK responses, retransmissions

Step 7: Compare with healthy pod
  Tool: nc + tcpdump on a healthy node
  Command: kubectl exec -it <healthy-pod> -- nc -zv 10.0.2.10 5432
  Looking for: Does the same test work from a healthy pod?
```

### Why This Order

1. **Cluster status** -- identifies the scope (node-specific vs cluster-wide)
2. **Node details** -- identifies root cause on the node
3. **Pod logs** -- shows application-level symptoms
4. **Ping** -- tests IP-level connectivity
5. **nc** -- tests TCP-level connectivity
6. **tcpdump** -- shows packet-level details
7. **Comparison** -- confirms the issue is node-specific

## Part B: Network Path Analysis

### 1. Get a Shell on a Pod on node-7

```bash
# List pods on node-7
kubectl get pods -A -o wide | grep node-7

# Get shell on an affected pod
kubectl exec -it <affected-pod-name> -n production -- /bin/bash

# If bash is not available, try sh
kubectl exec -it <affected-pod-name> -n production -- /bin/sh
```

### 2. Test Connectivity

```bash
# Test ICMP (ping)
ping -c 3 10.0.2.10
# Expected: replies if IP routing works

# Test TCP port
nc -zv 10.0.2.10 5432 -w 5
# Expected: "Connection succeeded" if port is open

# Test with timeout
timeout 5 bash -c 'echo > /dev/tcp/10.0.2.10/5432' && echo "OK" || echo "FAIL"

# Test with curl (if HTTP)
curl -v --connect-timeout 5 http://10.0.2.10:5432

# Check DNS (if using hostname)
nslookup database.internal
dig database.internal
```

### 3. Capture Packets

```bash
# Method 1: tcpdump inside pod (if available)
kubectl exec -it <pod> -- tcpdump -i eth0 -n -w /tmp/capture.pcap host 10.0.2.10 and port 5432

# Method 2: tcpdump on the node (more reliable)
# SSH to node-7
ssh node-7
# Find the pod's veth interface
ip link show | grep veth
# Capture on the veth interface
tcpdump -i veth12345 -n -w /tmp/capture.pcap host 10.0.2.10 and port 5432

# Method 3: Use kubectl debug (Kubernetes 1.25+)
kubectl debug node/node-7 -it --image=nicolaka/netshoot
# Then run tcpdump on the node's network namespace
```

## Part C: Root Cause Identification

### 1. What Does "NotReady" Mean?

A Kubernetes node in `NotReady` status means the **kubelet** on that node
has not sent a heartbeat to the control plane within the timeout period
(default: 40 seconds).

Common causes:
- **Kubelet process crashed** on node-7
- **Network connectivity lost** between node-7 and the control plane
- **Resource exhaustion** (memory, disk, PID pressure)
- **Container runtime failure** (Docker/containerd crashed)

```
kubectl describe node node-7
  Conditions:
    Ready: False (kubelet not posting status)
    MemoryPressure: ???
    DiskPressure: ???
    PIDPressure: ???
```

### 2. Why Does Ping Work But TCP Fail?

This is the key diagnostic clue:

```
Ping (ICMP):    WORKS   → IP routing is functional
TCP port 5432:  FAILS   → Something blocks TCP traffic
```

Possible causes:
1. **iptables/nftables rules** on node-7 blocking outbound TCP to port 5432
2. **NetworkPolicy** denying traffic from this pod to the database
3. **CNI plugin issue** on node-7 (Calico/Flannel/Cilium misconfigured)
4. **conntrack table overflow** on node-7 (too many connections, new ones dropped)
5. **MTU mismatch** causing TCP segmentation to fail (ICMP works because
   it uses smaller packets)

### 3. Root Cause: conntrack Table Overflow

The most likely cause (given the symptoms: ICMP works, TCP fails, only
on one node, no recent changes):

```bash
# Check conntrack table on node-7
ssh node-7
cat /proc/sys/net/netfilter/nf_conntrack_count
cat /proc/sys/net/netfilter/nf_conntrack_max

# If count ≈ max, conntrack table is full
# New TCP connections are dropped silently
# ICMP still works because it uses a different conntrack mechanism
```

```
Example output:
  nf_conntrack_count: 65536
  nf_conntrack_max:   65536  ← FULL!

When conntrack is full:
  - New TCP connections: DROPPED (SYN packets ignored)
  - Existing TCP connections: Continue working (already tracked)
  - ICMP: Works (different tracking, or falls back to stateless)
```

### 4. The Fix

```bash
# Immediate fix: increase conntrack table size
ssh node-7
sysctl -w net.netfilter.nf_conntrack_max=262144

# Verify
cat /proc/sys/net/netfilter/nf_conntrack_count
# Should now be well below max

# Test from affected pod
kubectl exec -it <pod> -- nc -zv 10.0.2.10 5432
# Should now succeed
```

## Part D: Resolution and Prevention

### Immediate Action (Restore Service Within 5 Minutes)

```
1. Increase conntrack table on node-7
   ssh node-7
   sysctl -w net.netfilter.nf_conntrack_max=262144

2. If that does not work, drain node-7 and reschedule pods
   kubectl drain node-7 --ignore-daemonsets --delete-emptydir-data
   Pods will be rescheduled to healthy nodes

3. Verify service recovery
   kubectl get pods -n production -o wide  # Pods on new nodes
   curl https://api-server/health           # Verify health
```

### Short-Term Fix (Within 1 Hour)

```
1. Investigate why conntrack table filled up
   - Check for connection leaks in application code
   - Check for SYN flood attacks
   - Check for misconfigured keepalive settings

2. Make conntrack increase persistent
   echo "net.netfilter.nf_conntrack_max=262144" >> /etc/sysctl.d/99-conntrack.conf
   sysctl -p /etc/sysctl.d/99-conntrack.conf

3. Uncordon node-7 (if drained)
   kubectl uncordon node-7

4. Monitor conntrack utilization
   watch -n 1 'cat /proc/sys/net/netfilter/nf_conntrack_count'
```

### Long-Term Prevention (Within 1 Week)

```
1. Add conntrack monitoring to Prometheus
   - Export node_nf_conntrack_entries metric
   - Alert when utilization > 80%

2. Set conntrack limits in node provisioning
   - Include sysctl tuning in node bootstrap script
   - Set nf_conntrack_max=262144 for all nodes

3. Add network policy monitoring
   - Verify NetworkPolicies are not blocking required traffic
   - Test connectivity after policy changes

4. Implement pod disruption budgets
   - Ensure minimum available pods during node failures
   - kubectl apply -f pdb.yaml

5. Add node health monitoring
   - Monitor kubelet health from outside the node
   - Alert on NotReady nodes within 30 seconds

6. Consider using eBPF-based CNI (Cilium)
   - eBPF avoids conntrack for pod-to-pod traffic
   - Better performance and fewer conntrack issues
```

### Monitoring Alerts to Add

```yaml
groups:
- name: network-alerts
  rules:
  - alert: ConntrackTableFull
    expr: node_nf_conntrack_entries / node_nf_conntrack_entries_limit > 0.8
    for: 2m
    labels:
      severity: warning
    annotations:
      summary: "Conntrack table >80% full on {{ $labels.instance }}"

  - alert: NodeNotReady
    expr: kube_node_status_condition{condition="Ready",status="true"} == 0
    for: 30s
    labels:
      severity: critical
    annotations:
      summary: "Node {{ $labels.node }} is NotReady"

  - alert: TCPRetransmissions
    expr: rate(node_netstat_Tcp_RetransSegs[5m]) > 10
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: "High TCP retransmissions on {{ $labels.instance }}"
```

### Common Mistakes to Avoid

- **Restarting pods without fixing the root cause.** If the node is the
  problem, rescheduling pods to the same node will not help. Drain the
  node first.
- **Only checking application logs.** The application logs show "connection
  timeout" but not WHY. The root cause is at the network level (conntrack),
  not the application level.
- **Not checking both ICMP and TCP.** The fact that ICMP works but TCP
  fails is the critical clue. Many engineers only test ping and assume
  connectivity is fine.
- **Ignoring node-level issues.** In Kubernetes, the node is a shared
  resource. A node-level issue (conntrack, iptables, CNI) affects all
  pods on that node. Always check node status first.

## Key Takeaway

Production network troubleshooting requires correlating multiple data points:
node status, pod logs, connectivity tests, and packet captures. The key
insight is that different protocols (ICMP vs TCP) can behave differently
when there is a network-level issue. conntrack table overflow is a common
cause of "ICMP works but TCP fails" symptoms. The systematic approach is:
check cluster status, then node status, then connectivity, then packets.
