# Exercise 05: Production Network Troubleshooting

**Type:** Integration
**Time:** 60 min
**Difficulty:** Hard

## Objective

Apply a systematic troubleshooting methodology to diagnose and resolve complex production network issues using multiple diagnostic tools.

## Scenario

You are the on-call engineer. Multiple alerts are firing:

```
Alert 1: api-server latency P99 > 5s (normally 200ms)
Alert 2: Database connection timeouts in api-server logs
Alert 3: Health check failures on 3 of 10 api-server pods
Alert 4: Increased TCP retransmissions on node-7

Timeline:
  14:00 - All alerts fire simultaneously
  14:01 - You receive the PagerDuty notification
  14:02 - You begin investigation

Environment:
  - Kubernetes cluster with 10 api-server pods
  - PostgreSQL database on a separate server (10.0.2.10:5432)
  - Pods on node-7 are affected, pods on other nodes are fine
  - No recent deployments or configuration changes
```

## Tasks

### Part A: Systematic Diagnosis

Write the step-by-step diagnostic procedure you would follow. For each step, specify:
1. What tool to use
2. What command to run
3. What you are looking for in the output

```
Step 1: _______________
  Tool: _______________
  Command: _______________
  Looking for: _______________

Step 2: _______________
...
```

<details>
<summary>Hint</summary>

Start with the broadest check (cluster-wide status) and narrow down. The fact that only node-7 pods are affected points to a node-specific issue, not a service-wide issue.

</details>

### Part B: Network Path Analysis

The affected pods are on node-7. You need to test connectivity from node-7 to the database:

1. How do you get a shell on a pod running on node-7?
2. What commands would you run to test connectivity to 10.0.2.10:5432?
3. How do you capture packets on the pod's network namespace?

<details>
<summary>Hint</summary>

Use `kubectl exec` to get a shell. Use `ping`, `nc` (netcat), and `curl` to test connectivity. For packet capture, you can use `tcpdump` inside the pod (if available) or capture on the node's interface.

</details>

### Part C: Root Cause Identification

Given these diagnostic results, identify the root cause:

```
Result 1: kubectl get nodes
  node-7   NotReady   45m   v1.28.2

Result 2: ping 10.0.2.10 from node-7 pod
  PING 10.0.2.10: 56 data bytes
  64 bytes: icmp_seq=1 ttl=63 time=0.5 ms
  64 bytes: icmp_seq=2 ttl=63 time=0.4 ms
  (ping works fine)

Result 3: nc -zv 10.0.2.10 5432
  nc: connect to 10.0.2.10 port 5432: Connection timed out

Result 4: tcpdump on node-7 interface
  14:00:01.000 IP node-7.45678 > db.5432: Flags [S], seq 1000
  14:00:01.500 IP node-7.45678 > db.5432: Flags [S], seq 1000 (retransmit)
  14:00:02.500 IP node-7.45678 > db.5432: Flags [S], seq 1000 (retransmit)
  14:00:04.500 IP node-7.45678 > db.5432: Flags [S], seq 1000 (retransmit)

Result 5: Same test from node-3 (healthy)
  nc -zv 10.0.2.10 5432
  Connection to 10.0.2.10 5432 port [tcp/postgresql] succeeded!
```

1. What does "NotReady" mean for a Kubernetes node?
2. Why does ping work but TCP port 5432 fail?
3. What is the root cause?
4. What is the fix?

<details>
<summary>Hint</summary>

Ping uses ICMP, TCP uses TCP. If ICMP works but TCP to a specific port fails, and SYN retransmissions occur with no SYN-ACK response, something is blocking TCP traffic from this specific node. The "NotReady" status suggests the node has a problem.

</details>

### Part D: Resolution and Prevention

Write the incident resolution plan:

1. Immediate action (restore service within 5 minutes)
2. Short-term fix (within 1 hour)
3. Long-term prevention (within 1 week)

<details>
<summary>Hint</summary>

Immediate: drain node-7, reschedule pods to healthy nodes. Short-term: investigate why node-7 is NotReady (network issue, kubelet problem, resource exhaustion). Long-term: add monitoring for node health, implement pod disruption budgets, set up network policies.

</details>

## Success Criteria

- [ ] You can follow a systematic diagnostic workflow
- [ ] You can test network connectivity from Kubernetes pods
- [ ] You can interpret tcpdump output to identify TCP connection failures
- [ ] You can distinguish between ICMP and TCP connectivity issues
- [ ] You can design immediate, short-term, and long-term resolution plans

## What You Should Understand After This Exercise

Production network troubleshooting requires a systematic approach: start broad (cluster status) and narrow down (specific node, specific port, specific packets). The key insight is that different protocols can behave differently: ICMP (ping) may work while TCP fails, indicating a firewall, routing, or iptables issue. Always correlate multiple data points (node status, packet captures, application logs) to identify the root cause.
