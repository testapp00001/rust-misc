# Solution 01: CNI, NetworkPolicy, and DNS -- How Kubernetes Networking Works

---

## Task 1: Kubernetes Networking Model

### 1a. The Three Guarantees

1. **Every Pod gets its own IP address.** Unlike Docker's default bridge
   networking where containers share the host IP and use port mapping, each Pod
   in Kubernetes gets a unique, cluster-routable IP. This means two Pods on the
   same node can both listen on port 8080 without conflict. For developers, this
   means you never need to think about port allocation -- your app binds to its
   native ports.

2. **Pod-to-Pod communication works without NAT.** Any Pod can reach any other
   Pod directly using its IP address. There is no network address translation
   between Pods. This means `10.244.1.5` can talk to `10.244.2.8` and the
   destination sees the real source IP. For developers, this means your
   application sees the actual client IP, which is critical for logging, rate
   limiting, and security.

3. **Agents on a node can communicate with all Pods on that node.** The
   kubelet, system daemons, and node-local agents need to reach Pods for health
   checks, log collection, and metrics scraping. This guarantee ensures the
   control plane can function. For developers, this means liveness and readiness
   probes work without special networking configuration.

### 1b. Packet Journey

1. **Source node:** The Pod at `10.244.1.5` sends a packet to `10.244.2.8`.
   The CNI plugin on `worker-1` intercepts the packet. If the destination is on
   a different node, the CNI encapsulates the packet (VXLAN for Flannel/Calico,
   or direct routing for Cilium with native routing).

2. **Between nodes:** The encapsulated packet travels across the node network
   (physical or virtual). With VXLAN, the original packet is wrapped in a UDP
   packet with the source and destination node IPs. With BGP (Calico) or native
   routing (Cilium), the packet is routed directly using standard IP routing.

3. **Destination node:** The CNI plugin on `worker-2` decapsulates the packet
   and delivers it to the Pod at `10.244.2.8`. The destination Pod sees the
   original source IP (`10.244.1.5`), not the node IP.

### 1c. Same-Node and Cross-Namespace Communication

Yes, two Pods on the same node can communicate directly through the node's
local bridge or veth pair setup. The CNI creates a virtual network on each node.

**Namespace isolation does NOT affect network connectivity by default.**
Namespaces are an organizational boundary, not a network boundary. A Pod in
namespace `dev` can reach a Pod in namespace `production` using its IP address
or DNS name. Network segmentation requires explicit NetworkPolicies.

---

## Task 2: CNI Plugin Comparison

### 2a. Comparison Table

| Feature | Flannel | Calico | Cilium |
|---------|---------|--------|--------|
| **NetworkPolicy enforcement** | None. Policies are stored but silently ignored. | Full L3/L4 enforcement via iptables or eBPF. | Full L3/L4/L7 enforcement via eBPF only. |
| **Routing mechanism** | VXLAN overlay. All traffic is encapsulated in UDP. | BGP (default) or VXLAN. BGP advertises Pod CIDRs to the network. | eBPF-based routing. Supports native routing (no encapsulation) and VXLAN. |
| **Encryption support** | None. | WireGuard (node-to-node). | WireGuard and IPsec (node-to-node). |
| **Observability tools** | None built-in. | Basic flow logs via `calicoctl`. | Hubble: real-time flow visualization, Prometheus metrics, Grafana dashboards. |
| **Layer 7 capabilities** | None. Operates at Layer 3 only. | None natively. Requires a service mesh (Istio) for L7. | Full L7 policy: HTTP method, path, headers, gRPC method. No sidecar needed. |
| **eBPF usage** | None. Uses iptables and kernel networking. | Optional. Calico can use eBPF dataplane instead of iptables. | Core architecture. All networking, load balancing, and policy enforcement uses eBPF. |

### 2b. Flannel vs. Calico with NetworkPolicy

**With Flannel:** The NetworkPolicy resource is created in the API server, but
nothing happens. Flannel does not watch for NetworkPolicy resources. Traffic
continues to flow unrestricted. There is no error, no warning, no event on the
resource. This is the most dangerous scenario -- the team believes the policy
is in effect because `kubectl get networkpolicy` shows it exists.

**With Calico:** The Calico agent (`calico-node`) on each node watches for
NetworkPolicy changes. When the policy is applied, Calico programs iptables
rules (or eBPF maps) on each node to drop all ingress traffic to Pods in the
namespace. Traffic is immediately blocked.

### 2c. Flannel to Cilium Migration

Replacing a CNI on a running cluster is risky:

1. **Pod IP changes:** Cilium may allocate IPs from a different CIDR range.
   Existing Pod IPs may change, breaking any hardcoded IP references.

2. **Brief network outage:** During the transition, nodes running Flannel
   cannot communicate with nodes running Cilium. They use different
   encapsulation and routing mechanisms.

3. **NetworkPolicy activation:** If you had NetworkPolicies defined (but
   unenforced under Flannel), they will suddenly take effect under Cilium.
   This can break applications that relied on unrestricted connectivity.

4. **Kernel version requirements:** Cilium requires Linux kernel 4.19+ (5.4+
   recommended) for full eBPF support. Flannel has lower kernel requirements.

The safe approach is to drain nodes one at a time, install Cilium, and verify
before moving to the next node. Test NetworkPolicies in a staging cluster first.

---

## Task 3: NetworkPolicy Semantics

### 3a. Policy A vs. Policy B

**Policy A** selects only Pods with label `app=backend` and allows ingress from
Pods with label `app=frontend`. All other Pods in the namespace are unaffected
-- they have no policy selecting them, so their traffic remains unrestricted.

**Policy B** selects ALL Pods in the namespace (`podSelector: {}`) with an empty
ingress rules list. This is a **default-deny** policy. Every Pod in the namespace
now has all ingress traffic blocked.

**When to use each:**
- Policy A: when you want to control traffic to a specific service without
  affecting others.
- Policy B: as the foundation of a zero-trust model. Apply Policy B first, then
  add specific allow policies like Policy A on top.

### 3b. Additive Semantics

`app=frontend` CAN reach `app=web` on port 8080. Why? Because Policy 1 allows
it. The fact that Policy 2 also selects `app=web` does not restrict Policy 1's
allowances. Policies are additive -- if ANY policy allows a flow, it is
permitted.

`app=monitoring` CANNOT reach `app=web` on port 8080. Policy 2 allows
monitoring on port 9090 only. Policy 1 allows frontend on port 80 only. No
policy allows monitoring on port 8080. Since both policies select `app=web`,
only the union of all allowed flows is permitted.

**Key insight:** NetworkPolicies define what IS allowed, not what IS denied.
Everything not explicitly allowed by any selecting policy is denied.

### 3c. Complete NetworkPolicy

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: database-policy
spec:
  podSelector:
    matchLabels:
      role: database
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - podSelector:
        matchLabels:
          role: backend
    ports:
    - protocol: TCP
      port: 5432
  egress:
  - to:
    - podSelector:
        matchLabels:
          role: backend
    ports:
    - protocol: TCP
      port: 5432
  - to:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: kube-system
    ports:
    - protocol: UDP
      port: 53
    - protocol: TCP
      port: 53
```

---

## Task 4: DNS Resolution

### 4a. DNS Names for Cross-Namespace Resolution

A Pod in `payments` reaching `ledger` in `finance`:

1. `ledger.finance.svc.cluster.local` -- the FQDN, guaranteed to work
2. `ledger.finance.svc` -- without the cluster domain
3. `ledger.finance` -- without the `svc` suffix

Short names like `ledger` will NOT work across namespaces -- they only resolve
within the same namespace.

### 4b. DNS Query Trace with ndots:5

The name `ledger` has 0 dots. Since `ndots:5`, the resolver treats it as
relative and appends search domains first:

1. `ledger.payments.svc.cluster.local` -- search domain 1 (fails, no such
   Service in payments)
2. `ledger.svc.cluster.local` -- search domain 2 (fails)
3. `ledger.cluster.local` -- search domain 3 (fails)
4. `ledger` -- absolute lookup (fails, not a valid FQDN)

The correct FQDN `ledger.finance.svc.cluster.local` has 4 dots. With
`ndots:5`, even this is treated as relative, so search domains are tried first.

**To fix:** Either use `ledger.finance.svc.cluster.local.` (trailing dot makes
it absolute) or set `ndots:4` or lower in the Pod spec:

```yaml
spec:
  dnsConfig:
    options:
    - name: ndots
      value: "4"
```

### 4c. Five Things to Check for DNS Failure

1. **Is CoreDNS running?** `kubectl get pods -n kube-system -l k8s-app=kube-dns`
   -- if CoreDNS Pods are CrashLoopBackOff or Pending, DNS is down.

2. **Is there a NetworkPolicy blocking DNS egress?** Check for egress policies
   in the Pod's namespace that might not include port 53 UDP/TCP to kube-system.

3. **Is the Pod's resolv.conf correct?** `kubectl exec pod -- cat /etc/resolv.conf`
   -- verify the nameserver points to the CoreDNS Service IP (usually
   `10.96.0.10`).

4. **Can the Pod reach the CoreDNS IP?** `kubectl exec pod -- ping 10.96.0.10`
   -- network connectivity issues (CNI problems, node routing) can prevent
   reaching CoreDNS even if DNS config is correct.

5. **Are CoreDNS logs healthy?** `kubectl logs -n kube-system -l k8s-app=kube-dns`
   -- look for errors like `plugin/kubernetes: connection refused` or
   `out of memory`.

---

## Common Mistakes

1. **Assuming namespaces provide network isolation.** They do not. Without
   NetworkPolicies, any Pod can reach any other Pod regardless of namespace.

2. **Believing NetworkPolicies work with Flannel.** Flannel does not enforce
   NetworkPolicies. Always verify your CNI supports them before relying on them.

3. **Forgetting egress rules when applying deny-all.** A deny-all that includes
   `Egress` in `policyTypes` breaks DNS immediately. Always add DNS egress
   alongside deny-all.

4. **Confusing `ndots` behavior.** With `ndots:5` (the default), any name with
   fewer than 5 dots gets search domains appended first. This causes extra DNS
   queries and can resolve to the wrong Service.

5. **Not understanding additive semantics.** If you have two policies selecting
   the same Pod, the allowed traffic is the UNION of both policies, not the
   intersection. A permissive policy can undermine a restrictive one.
