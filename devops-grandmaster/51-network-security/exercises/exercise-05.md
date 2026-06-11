# Exercise 05: From Flat Network to Zero-Trust Segmentation

**Type:** Integration
**Time:** 45 minutes
**Difficulty:** Hard

## Objective
Combine network security concepts with container orchestration (Kubernetes), service mesh technology, and VPN infrastructure to design a complete zero-trust network architecture for a microservices application. This exercise integrates knowledge from multiple modules and simulates the kind of architecture decision you would make as a senior platform engineer.

## The Scenario

"FinLedger" is a financial services company running a microservices application on Kubernetes. The current architecture is a flat network with no segmentation:

```
Current State: Everything on one network
==========================================

Internet -> LoadBalancer -> Kubernetes Cluster (all pods can talk to all pods)
                            |
                            +-- frontend (React SPA)
                            +-- api-gateway (Kong)
                            +-- account-service
                            +-- transaction-service
                            +-- notification-service
                            +-- postgresql (StatefulSet)
                            +-- redis (StatefulSet)
                            +-- vault (HashiCorp Vault)
                            +-- prometheus
                            +-- grafana
```

**Problems with the current design:**
- Any pod can reach any other pod
- No encryption between pods (plain HTTP)
- Database accessible from any pod
- No network-level audit trail
- Secrets are in environment variables
- No VPN for remote access to internal dashboards

## Tasks

### Part A: Design the Network Segmentation Model
Create a zero-trust network segmentation model for the Kubernetes cluster. For each microservice, define:
- Which other services it is allowed to communicate with
- What ports and protocols are permitted
- What identity/authentication is required for each connection

Present your design as a NetworkPolicy YAML manifest for Kubernetes that implements namespace-level and pod-level isolation.

```yaml
# Write your Kubernetes NetworkPolicy manifests here
```

<details>
<summary>Hint</summary>
Use default-deny ingress and egress policies in each namespace, then add specific allow rules. Group services by trust level: public-facing (frontend, api-gateway), business logic (account, transaction), data (postgres, redis), infrastructure (vault, prometheus, grafana). Each allow rule should specify both the source and destination pods by label selector.
</details>

### Part B: Implement Service Mesh with mTLS
Design the service mesh configuration that provides:
- Mutual TLS (mTLS) between all services
- Traffic policies that enforce the segmentation from Part A
- Observability (distributed tracing, metrics)
- Circuit breaking for resilience

Write the Istio (or Linkerd) configuration for:
1. Enabling strict mTLS cluster-wide
2. AuthorizationPolicy resources that enforce which services can call which
3. Traffic management policies (timeouts, retries, circuit breakers)

```yaml
# Write your Istio/Linkerd configuration here
```

<details>
<summary>Hint</summary>
Start with a PeerAuthentication resource setting mtls mode to STRICT. Then use AuthorizationPolicy to define allow rules per service. The api-gateway should be allowed to call account-service and transaction-service. The transaction-service should be allowed to call account-service, postgresql, and vault. No service should be allowed to call prometheus except prometheus itself.
</details>

### Part C: Design VPN Access to Internal Services
Design a WireGuard VPN solution that allows:
- DevOps engineers to access Grafana dashboards
- Database administrators to access PostgreSQL directly (for emergencies)
- Developers to access the Kubernetes API server

Specify:
1. WireGuard server configuration (on a bastion host)
2. Client configuration template
3. DNS resolution for internal services through the VPN
4. Firewall rules that restrict VPN users to only their authorized services

```ini
# Write WireGuard server configuration
[Interface]
# ...

[Peer]
# ...
```

<details>
<summary>Hint</summary>
Place the WireGuard server in a management subnet with firewall rules to the Kubernetes cluster. Use different peer groups with different AllowedIPs ranges to control access. Configure split DNS so that internal service names resolve through the VPN. Use preshared keys plus additional authentication (e.g., SSO integration) for production use.
</details>

### Part D: Integrate HashiCorp Vault for Secrets
Redesign the secrets management to:
1. Remove all secrets from environment variables and ConfigMaps
2. Use Vault Agent sidecar to inject secrets into pods
3. Implement short-lived database credentials
4. Enable audit logging of all secret access

Write the Vault policy, Kubernetes auth configuration, and a sample deployment manifest that uses Vault injection.

```yaml
# Write Vault policy and Kubernetes deployment with Vault injection
```

<details>
<summary>Hint</summary>
Enable Kubernetes auth method in Vault. Create policies that restrict each service to only its secrets path. Use the `vault.hashicorp.com/agent-inject-*` annotations on pods. Configure the database secrets engine for dynamic credentials. Every secret access should be audit-logged.
</details>

### Part E: Create the Complete Architecture Diagram and Documentation
Draw the final architecture as an ASCII diagram showing:
- Network segments (namespaces, subnets)
- Allowed traffic flows with protocols and authentication
- VPN access paths
- Secret injection paths
- Monitoring data flows

Write a one-page architecture decision record (ADR) explaining:
- Why each technology was chosen
- What trade-offs were made
- What threats each control mitigates
- What operational overhead is introduced

<details>
<summary>Hint</summary>
The diagram should show at least: the internet boundary, the ingress controller, the service mesh with mTLS arrows, the data tier, the management tier, and the VPN endpoint. The ADR should address: why Istio over Linkerd (or vice versa), why WireGuard over OpenVPN, the performance cost of mTLS, and the operational complexity of Vault.
</details>

## Success Criteria
- [ ] Kubernetes NetworkPolicies implement default-deny with specific allow rules
- [ ] mTLS is enforced cluster-wide with AuthorizationPolicy resources
- [ ] WireGuard VPN provides tiered access (DevOps, DBA, Developer)
- [ ] All secrets are managed through Vault with no plaintext in manifests
- [ ] Architecture diagram clearly shows all segments, flows, and controls
- [ ] ADR explains technology choices and trade-offs

## What You Should Understand After This Exercise
Zero-trust networking is not a single product -- it is a design philosophy implemented through multiple layers of control. Network policies control who can talk to whom at the IP level. Service meshes add identity and encryption at the application level. VPNs provide controlled access to internal resources. Vault ensures that even if an attacker compromises a pod, they cannot extract long-lived credentials. Each layer compensates for the weaknesses of the others. The operational cost is real -- more moving parts, more things to debug, more YAML to maintain -- but the security posture is fundamentally stronger than a flat network where every pod trusts every other pod.
