# Exercise 04: Multi-Tenancy with Quotas and Network Policies

**Type:** Challenge
**Time:** 45 minutes
**Difficulty:** Medium-Hard

## Objective

Design a complete multi-tenant namespace setup that combines RBAC, ResourceQuotas,
LimitRanges, and NetworkPolicies. You will provision two tenant namespaces with
hard resource boundaries, default container limits, and network isolation.

## Scenario

You are the platform engineer at a SaaS company. Two customer teams -- `tenant-omega`
and `tenant-zeta` -- share a Kubernetes cluster. Each tenant must have:

- A dedicated namespace with resource quotas so one tenant cannot starve the other.
- Default container resource limits so that pods without explicit requests/limits
  still get sane defaults.
- A NetworkPolicy that isolates traffic to within the namespace only.
- A developer Role and a read-only auditor Role, each bound to the appropriate group.
- A ServiceAccount for each tenant's application workload.

## Tasks

### Part A: Namespace and Labels

Create namespaces `tenant-omega` and `tenant-zeta` with labels that identify the
tenant and indicate the namespace is managed.

<details>
<summary>Hint</summary>

```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: tenant-omega
  labels:
    tenant: omega
    managed-by: platform-team
```

Labels are not just for identification -- NetworkPolicies use `namespaceSelector`
to match namespaces by label.

</details>

### Part B: ResourceQuotas

Create a ResourceQuota for each namespace with these limits:

| Resource | Limit |
|----------|-------|
| requests.cpu | 10 |
| requests.memory | 20Gi |
| limits.cpu | 20 |
| limits.memory | 40Gi |
| pods | 50 |
| services | 10 |
| persistentvolumeclaims | 5 |
| services.loadbalancers | 1 |

Write the YAML for `tenant-omega`.

<details>
<summary>Hint</summary>

```yaml
apiVersion: v1
kind: ResourceQuota
metadata:
  name: tenant-quota
  namespace: tenant-omega
spec:
  hard:
    requests.cpu: "10"
    requests.memory: 20Gi
    ...
```

The quota applies to all pods in the namespace. If a pod does not specify resource
requests, it may be rejected by the quota if the default LimitRange sets requests
that exceed the remaining quota.

</details>

### Part C: LimitRanges

Create a LimitRange for each namespace that sets:

| Setting | Value |
|---------|-------|
| Default CPU limit | 500m |
| Default memory limit | 512Mi |
| Default CPU request | 100m |
| Default memory request | 128Mi |
| Max CPU limit | 2 |
| Max memory limit | 4Gi |

Write the YAML for `tenant-omega`.

<details>
<summary>Hint</summary>

A LimitRange has a `spec.limits` list. Each entry has a `type` (Container, Pod, or
PersistentVolumeClaim). Use `type: Container` and set `default`, `defaultRequest`,
and `max` fields.

```yaml
apiVersion: v1
kind: LimitRange
metadata:
  name: default-limits
  namespace: tenant-omega
spec:
  limits:
    - type: Container
      default:
        cpu: 500m
        memory: 512Mi
      defaultRequest:
        cpu: 100m
        memory: 128Mi
      max:
        cpu: "2"
        memory: 4Gi
```

</details>

### Part D: NetworkPolicy

Create a NetworkPolicy for each namespace that:

1. Denies all ingress traffic by default.
2. Allows ingress only from pods within the same namespace.
3. Allows ingress from pods in a namespace labeled `shared-services: true` (for
   shared monitoring, logging, etc.).

Write the YAML for `tenant-omega`.

<details>
<summary>Hint</summary>

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: default-deny-with-exceptions
  namespace: tenant-omega
spec:
  podSelector: {}          # apply to all pods in the namespace
  policyTypes:
    - Ingress
  ingress:
    - from:
        - podSelector: {}  # allow from same namespace
        - namespaceSelector:
            matchLabels:
              shared-services: "true"
```

An empty `podSelector: {}` matches all pods. Multiple `from` entries are OR-ed:
traffic from any matching source is allowed.

</details>

### Part E: RBAC Roles and Bindings

Create for each namespace:

1. A `developer` Role with full CRUD on Deployments, Services, ConfigMaps, Pods,
   and Secrets.
2. A `auditor` Role with read-only (get, list, watch) on all workload resources.
3. RoleBindings that grant:
   - Group `omega-developers` the `developer` Role in `tenant-omega`
   - Group `omega-auditors` the `auditor` Role in `tenant-omega`

Should these Roles be namespace-scoped (`Role`) or cluster-scoped (`ClusterRole`)?
Explain your choice.

<details>
<summary>Hint</summary>

Use namespace-scoped `Role` objects, not ClusterRoles. Each tenant's permissions
should be isolated to their own namespace. If you used ClusterRoles with
ClusterRoleBindings, the permissions would apply across all namespaces.

However, if the same "developer" permission set is reused across many namespaces,
you could define it once as a ClusterRole and bind it per-namespace using a
RoleBinding (this is a valid pattern -- ClusterRole + RoleBinding = namespace-scoped
grant of cluster-level permission definition).

</details>

### Part F: ServiceAccount for Workloads

Create a ServiceAccount `app-sa` in each namespace. Bind it to the `developer` Role
so that the application can read its own ConfigMaps and Secrets (but not modify them).

Wait -- should the application ServiceAccount have the same permissions as the
developer Role? Explain why or why not, and if not, create a more appropriate Role.

<details>
<summary>Hint</summary>

The application workload should NOT have the same permissions as a human developer.
It needs read-only access to its own ConfigMaps and Secrets, and the ability to
report its own status. Create a separate `app-role` with minimal permissions:
`get` and `list` on `configmaps` and `secrets`, and `get`, `list`, `watch` on `pods`.

</details>

## Success Criteria

- [ ] Both tenant namespaces exist with identifying labels
- [ ] ResourceQuotas are applied and prevent resource exhaustion
- [ ] LimitRanges set sane defaults and maximums for container resources
- [ ] NetworkPolicies restrict ingress to same-namespace and shared-services only
- [ ] Developer and auditor Roles exist with correct permissions
- [ ] RoleBindings grant permissions to the correct groups
- [ ] Workload ServiceAccounts have minimal permissions (not developer-level)
- [ ] You can explain why ClusterRole + RoleBinding is a valid pattern for reusable Roles
- [ ] You can explain why workload ServiceAccounts should have fewer permissions than human developers

## What You Should Understand After This Exercise

Multi-tenancy is not just RBAC. It requires combining namespace isolation, resource
quotas (to prevent one tenant from consuming all cluster resources), LimitRanges
(to ensure pods have sane defaults), NetworkPolicies (to prevent cross-tenant
network access), and RBAC (to control who can do what). Each layer addresses a
different dimension of isolation, and all layers must work together for true
multi-tenancy.
