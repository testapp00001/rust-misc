# Exercise 01: CNI, NetworkPolicy, and DNS -- How Kubernetes Networking Works

**Type:** Conceptual
**Time:** 25 minutes
**Objective:** Understand the Kubernetes networking model, CNI plugin
architecture, NetworkPolicy semantics, and DNS resolution so you can reason
about traffic flow in any cluster.

---

## Background

Your team has inherited a Kubernetes cluster from another team. The cluster has
50 microservices across 8 namespaces. Nobody knows which services talk to which,
there are no NetworkPolicies, and DNS resolution is failing intermittently.
Before you can fix anything, you need to understand the fundamentals.

---

## Tasks

### Task 1: Kubernetes Networking Model

Answer each question with a specific, concrete answer.

**1a.** Kubernetes guarantees three networking rules. List them and explain what
each one means for application developers.

**1b.** A Pod with IP `10.244.1.5` on node `worker-1` needs to reach a Pod with
IP `10.244.2.8` on node `worker-2`. Explain the packet journey at a high level:
what happens at the source node, between nodes, and at the destination node?

**1c.** Two Pods on the same node have IPs `10.244.1.5` and `10.244.1.6`. Can
they communicate? What if they are in different namespaces? Does namespace
isolation affect network connectivity by default?

### Task 2: CNI Plugin Comparison

**2a.** Fill in the comparison table with specific technical details, not vague
statements.

| Feature | Flannel | Calico | Cilium |
|---------|---------|--------|--------|
| NetworkPolicy enforcement | ? | ? | ? |
| Routing mechanism | ? | ? | ? |
| Encryption support | ? | ? | ? |
| Observability tools | ? | ? | ? |
| Layer 7 capabilities | ? | ? | ? |
| eBPF usage | ? | ? | ? |

**2b.** Your team deploys a NetworkPolicy that denies all ingress to a namespace.
With Flannel as the CNI, what happens? With Calico? Explain why.

**2c.** What happens to existing Pod connectivity when you replace Flannel with
Cilium on a running cluster? Describe the migration risk.

### Task 3: NetworkPolicy Semantics

**3a.** Explain the difference between these two policies. When would you use
each?

```yaml
# Policy A
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: policy-a
spec:
  podSelector:
    matchLabels:
      app: backend
  policyTypes:
  - Ingress
  ingress:
  - from:
    - podSelector:
        matchLabels:
          app: frontend
```

```yaml
# Policy B
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: policy-b
spec:
  podSelector: {}
  policyTypes:
  - Ingress
```

**3b.** A namespace has two NetworkPolicies:

- Policy 1 selects `app=web` and allows ingress from `app=frontend` on port 80.
- Policy 2 selects `app=web` and allows ingress from `app=monitoring` on
  port 9090.

Can `app=frontend` reach `app=web` on port 9090? Can `app=monitoring` reach
`app=web` on port 80? Explain using the additive semantics of NetworkPolicy.

**3c.** Write a NetworkPolicy that:
- Selects Pods with label `role=database`
- Denies all ingress except from Pods with label `role=backend` on TCP port 5432
- Denies all egress except to Pods with label `role=backend` on TCP port 5432,
  and to kube-dns on UDP port 53

### Task 4: DNS Resolution

**4a.** A Pod in namespace `payments` needs to reach a Service called `ledger` in
namespace `finance`. List every DNS name that would resolve, in order of
resolution attempt.

**4b.** A Pod has this `/etc/resolv.conf`:

```
nameserver 10.96.0.10
search payments.svc.cluster.local svc.cluster.local cluster.local
ndots:5
```

The Pod does `curl http://ledger`. What DNS queries does the resolver make
before it finds (or fails to find) the Service? List each FQDN tried.

**4c.** CoreDNS is running but a Pod cannot resolve `kubernetes.default`. List
five distinct things you would check, in order of likelihood.

---

## Success Criteria

- [ ] You can explain all three Kubernetes networking guarantees with examples.
- [ ] Your CNI comparison table has specific technical details for all six rows.
- [ ] You can predict the behavior of combined NetworkPolicies using additive
      semantics.
- [ ] You can trace DNS resolution from a Pod with custom search domains.
- [ ] You can write a complete NetworkPolicy with both ingress and egress rules.

---

## Hints

<details>
<summary>Hint 1: Networking Guarantees</summary>

The three guarantees are:

1. Every Pod gets its own IP address (no port sharing between Pods on the same
   node, unlike Docker default bridge networking).
2. Pods can communicate with each other without NAT (flat network).
3. Agents on a node (kubelet, system daemons) can communicate with all Pods on
   that node.

These are guarantees the CNI must implement. Kubernetes itself does not ship a
networking implementation -- it delegates to plugins.

</details>

<details>
<summary>Hint 2: CNI and NetworkPolicy</summary>

NetworkPolicy is a Kubernetes API resource. The kube-apiserver stores it. But
the **enforcement** happens in the CNI plugin on each node. If the CNI does not
watch for NetworkPolicy resources (like Flannel), the policies exist in the API
server but are never enforced. There is no error or warning -- they are
silently ignored.

Calico uses iptables or eBPF to enforce policies. Cilium uses eBPF exclusively,
which allows it to enforce policies at Layer 7 (HTTP paths, gRPC methods).

</details>

<details>
<summary>Hint 3: Additive NetworkPolicy Semantics</summary>

NetworkPolicies are **additive** (union). If any policy selects a Pod and
allows a traffic flow, that flow is permitted -- even if another policy that
also selects the Pod does not include that flow.

The key rule: once ANY policy selects a Pod, ALL traffic not explicitly allowed
by ANY selecting policy is denied. Policies do not override each other; they
combine.

If Policy 1 allows `app=frontend` -> `app=web:80` and Policy 2 allows
`app=monitoring` -> `app=web:9090`, then the Pod `app=web` allows both flows.
But `app=frontend` -> `app=web:9090` is NOT allowed because no policy permits
it.

</details>

<details>
<summary>Hint 4: DNS Resolution Order</summary>

With `ndots:5`, any name with fewer than 5 dots is considered a relative name
and gets the search domains appended first. The name `ledger` has 0 dots, so:

1. `ledger.payments.svc.cluster.local` (search domain 1)
2. `ledger.svc.cluster.local` (search domain 2)
3. `ledger.cluster.local` (search domain 3)
4. `ledger` (absolute lookup -- fails)

The FQDN `ledger.finance.svc.cluster.local` has 4 dots. With `ndots:5`, this
is ALSO treated as relative, so search domains are tried first.

To avoid this, use FQDNs ending with a dot: `ledger.finance.svc.cluster.local.`
or increase ndots.

</details>

<details>
<summary>Hint 5: Debugging DNS</summary>

Five things to check when DNS fails:

1. Is CoreDNS running? (`kubectl get pods -n kube-system -l k8s-app=kube-dns`)
2. Does the Pod have the correct `/etc/resolv.conf`? (`kubectl exec pod -- cat /etc/resolv.conf`)
3. Is there a NetworkPolicy blocking DNS egress (port 53 UDP/TCP)?
4. Is CoreDNS healthy? Check its logs for errors.
5. Can the Pod reach the CoreDNS Service IP? (network connectivity, not DNS)

</details>
