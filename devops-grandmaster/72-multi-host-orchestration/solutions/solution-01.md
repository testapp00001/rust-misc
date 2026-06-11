# Solution 01: Swarm vs Nomad vs Kubernetes

## Part A: Feature Comparison Table

| Feature | Docker Swarm | Nomad | Kubernetes |
|---------|-------------|-------|------------|
| **Installation complexity** | Single command (`docker swarm init`). Built into Docker. | Single binary download. Minimal configuration. | Multiple components (API server, etcd, scheduler, controller manager, kubelet). Requires tools like kubeadm or managed services. |
| **Supported workloads** | Containers only (Docker runtime). | Containers, VMs, standalone binaries, Java JARs, QEMU. Any executor plugin. | Primarily containers (CRI-compatible runtimes: containerd, CRI-O). |
| **Service discovery** | Built-in DNS on overlay networks. Automatic, no configuration needed. | Relies on Consul integration (external dependency). DNS and HTTP interfaces. | CoreDNS built-in. Service objects with ClusterIP, NodePort, LoadBalancer types. |
| **Secrets management** | Docker secrets (encrypted at rest, mounted as files in containers). | Vault integration (external dependency, but first-class). | Kubernetes Secrets (base64-encoded, not encrypted by default without encryption config). Vault CSI driver available. |
| **Auto-scaling** | Manual scaling only (`docker service scale`). No built-in autoscaler. | External autoscaler required (e.g., Nomad Autoscaler). Supports scaling policies. | HPA (Horizontal Pod Autoscaler), VPA (Vertical Pod Autoscaler), Cluster Autoscaler. All built-in. |
| **Community and ecosystem** | Declining. Limited operator/extension ecosystem. | Growing. Moderate ecosystem, strong HashiCorp integration. | Massive. Thousands of operators, CRDs, Helm charts, CNCF graduated project. |

### Common Mistakes to Avoid

- Saying Swarm and Nomad are "worse" than Kubernetes. They solve different problems at different complexity levels.
- Forgetting that Nomad's service discovery requires Consul -- it is not built-in like Swarm's DNS or Kubernetes' CoreDNS.
- Confusing Docker secrets (encrypted at rest, rotated on service update) with Kubernetes Secrets (base64-encoded, need encryption-at-rest configuration to be truly secure).

---

## Part B: Scenario Matching

**Scenario 1: 5-person team, single Python app, 3 servers.**
**Recommendation: Docker Swarm.** It requires zero additional infrastructure beyond Docker itself. The team already knows Docker, and a 3-server deployment does not need Kubernetes-level complexity. Swarm's built-in overlay networking and DNS discovery are sufficient.

**Scenario 2: Containers + legacy Java WAR + compiled Go binary.**
**Recommendation: Nomad.** Nomad's `exec` and `java` drivers can run non-containerized workloads alongside Docker containers. Neither Swarm nor Kubernetes natively supports running a WAR file or bare binary without containerizing it first.

**Scenario 3: Large enterprise, CRDs, operators, multi-cloud, dedicated platform team.**
**Recommendation: Kubernetes.** The ecosystem (operators, CRDs, Helm) is unmatched. Multi-cloud portability is a first-class K8s feature. A dedicated platform team can absorb the operational complexity. No other orchestrator has comparable community tooling.

**Scenario 4: Existing Consul/Vault/Terraform stack, minimal overhead.**
**Recommendation: Nomad.** It integrates natively with the HashiCorp ecosystem. Consul handles service discovery, Vault handles secrets, and Terraform can manage Nomad jobs. Adding Kubernetes would introduce an entirely separate platform with its own concepts.

### Common Mistakes to Avoid

- Recommending Kubernetes for Scenario 1 because "it is the industry standard." Over-engineering a small deployment wastes time and introduces unnecessary operational burden.
- Recommending Swarm for Scenario 2. Swarm cannot run non-containerized workloads.
- Not considering the team's existing skills and tooling. The best orchestrator is one the team can operate reliably.

---

## Part C: Trade-off Analysis

**Docker Swarm:**
- **Strength:** Lowest barrier to entry. If you know Docker, you know 80% of Swarm. Zero additional infrastructure. Built-in overlay networking and DNS discovery.
- **Weakness:** Limited ecosystem. No auto-scaling, no operators, no CRDs. Community is declining, which means fewer resources, fewer updates, and less long-term viability.

**Nomad:**
- **Strength:** Workload flexibility (containers, VMs, binaries, Java). Clean integration with Consul/Vault/Terraform. Simpler operational model than Kubernetes. Built-in multi-region federation.
- **Weakness:** Smaller community than Kubernetes. Service discovery depends on Consul (external dependency). Fewer third-party integrations and Helm-chart-equivalent ecosystem.

**Kubernetes:**
- **Strength:** Massive ecosystem (operators, CRDs, Helm, service meshes). Multi-cloud portability. Built-in auto-scaling, RBAC, network policies. Industry standard with extensive documentation and community support.
- **Weakness:** Steep learning curve. Complex installation and upgrades. Overkill for simple deployments. Requires a dedicated platform team for production operations.

### Common Mistakes to Avoid

- Treating this as a "which is best" question. Each tool has a sweet spot.
- Ignoring organizational factors (team size, existing skills, vendor relationships) and focusing only on technical features.

---

## Key Takeaway

The choice of orchestrator is a decision about operational complexity vs. capability. Swarm is the simplest path from single-host to multi-host. Nomad adds workload flexibility without Kubernetes-level complexity. Kubernetes provides the richest ecosystem at the cost of significant operational overhead. Match the tool to the team and the problem.
