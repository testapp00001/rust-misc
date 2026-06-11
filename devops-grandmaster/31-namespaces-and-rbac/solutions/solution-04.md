# Solution 04: Multi-Tenancy with Quotas and Network Policies

## Part A: Namespace and Labels

```yaml
# namespace-tenant-omega.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: tenant-omega
  labels:
    tenant: omega
    managed-by: platform-team
    environment: production
```

```yaml
# namespace-tenant-zeta.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: tenant-zeta
  labels:
    tenant: zeta
    managed-by: platform-team
    environment: production
```

```bash
kubectl apply -f namespace-tenant-omega.yaml
kubectl apply -f namespace-tenant-zeta.yaml
```

**Why this works**: Labels serve multiple purposes: they identify the owning team,
indicate that the namespace is managed by the platform team (not created ad-hoc),
and can be used by NetworkPolicies via `namespaceSelector` to allow or deny traffic
based on label matches.

---

## Part B: ResourceQuotas

```yaml
# resourcequota-tenant-omega.yaml
apiVersion: v1
kind: ResourceQuota
metadata:
  name: tenant-quota
  namespace: tenant-omega
spec:
  hard:
    requests.cpu: "10"
    requests.memory: 20Gi
    limits.cpu: "20"
    limits.memory: 40Gi
    pods: "50"
    services: "10"
    persistentvolumeclaims: "5"
    services.loadbalancers: "1"
```

**Why this works**:

- `requests.cpu` and `requests.memory` cap the total requested resources. A Pod
  that requests 2 CPU can only have 5 replicas (10 / 2) before the quota blocks
  further scheduling.
- `limits.cpu` and `limits.memory` cap the total limit resources. This prevents
  over-commitment.
- `pods: "50"` caps the number of pods. This prevents a tenant from creating
  thousands of tiny pods that overwhelm the scheduler.
- `services.loadbalancers: "1"` is critical: LoadBalancer services provision
  expensive cloud resources. Without this cap, a tenant could create hundreds of
  load balancers and run up a huge cloud bill.

Apply the same manifest to `tenant-zeta` with the namespace changed.

---

## Part C: LimitRanges

```yaml
# limitrange-tenant-omega.yaml
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

**Why this works**:

- `default` sets the resource limits for containers that do not specify their own.
  Without this, a container without limits can consume unbounded resources within
  the namespace quota.
- `defaultRequest` sets the resource requests. Requests are used for scheduling
  decisions. Without this, pods without requests might be scheduled on overloaded
  nodes.
- `max` caps the maximum resources any single container can request. Even if a
  developer explicitly sets `resources.limits.cpu: 4`, the LimitRange will reject
  it because it exceeds the `max` of `2`.
- The LimitRange works together with the ResourceQuota: the LimitRange ensures
  every container has requests/limits, and the ResourceQuota ensures the total
  across all containers stays within bounds.

Apply the same manifest to `tenant-zeta` with the namespace changed.

---

## Part D: NetworkPolicy

```yaml
# networkpolicy-tenant-omega.yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: default-ingress-policy
  namespace: tenant-omega
spec:
  podSelector: {}          # apply to ALL pods in this namespace
  policyTypes:
    - Ingress
  ingress:
    - from:
        # Allow traffic from pods within the same namespace
        - podSelector: {}
        # Allow traffic from the shared-services namespace
        - namespaceSelector:
            matchLabels:
              shared-services: "true"
```

**Why this works**:

- `podSelector: {}` matches all pods in the namespace. This means the policy
  applies to every pod as an ingress target.
- `policyTypes: [Ingress]` means only ingress traffic is affected. Egress is
  unrestricted (pods can still reach external services, databases, etc.).
- The `from` list is OR-ed: traffic is allowed if it matches ANY of the entries.
  - `podSelector: {}` matches all pods in the same namespace. This allows
    intra-namespace communication (e.g., frontend calling backend).
  - `namespaceSelector` with `shared-services: "true"` allows traffic from any
    pod in a namespace labeled `shared-services: true`. This is for shared
    monitoring (Prometheus scraping), logging (Fluentd), or service mesh
    sidecars.
- Traffic from all other sources (other tenant namespaces, external IPs) is denied
  by default once a NetworkPolicy exists for the namespace.

Apply the same manifest to `tenant-zeta` with the namespace changed. Create the
`shared-services` namespace if it does not exist:

```bash
kubectl create namespace shared-services
kubectl label namespace shared-services shared-services=true
```

---

## Part E: RBAC Roles and Bindings

### Developer Role

```yaml
# role-developer-omega.yaml
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: developer
  namespace: tenant-omega
rules:
  - apiGroups: ["apps"]
    resources: ["deployments", "replicasets"]
    verbs: ["get", "list", "watch", "create", "update", "patch"]
  - apiGroups: [""]
    resources: ["services", "configmaps", "pods", "pods/log", "secrets"]
    verbs: ["get", "list", "watch", "create", "update", "patch"]
```

### Auditor Role

```yaml
# role-auditor-omega.yaml
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: auditor
  namespace: tenant-omega
rules:
  - apiGroups: ["apps"]
    resources: ["deployments", "replicasets", "statefulsets"]
    verbs: ["get", "list", "watch"]
  - apiGroups: [""]
    resources: ["pods", "pods/log", "services", "configmaps", "events"]
    verbs: ["get", "list", "watch"]
```

### RoleBindings

```yaml
# rolebinding-developer-omega.yaml
apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: developer-binding
  namespace: tenant-omega
subjects:
  - kind: Group
    name: omega-developers
    apiGroup: rbac.authorization.k8s.io
roleRef:
  kind: Role
  name: developer
  apiGroup: rbac.authorization.k8s.io
```

```yaml
# rolebinding-auditor-omega.yaml
apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: auditor-binding
  namespace: tenant-omega
subjects:
  - kind: Group
    name: omega-auditors
    apiGroup: rbac.authorization.k8s.io
roleRef:
  kind: Role
  name: auditor
  apiGroup: rbac.authorization.k8s.io
```

### Role vs. ClusterRole Decision

These should be namespace-scoped `Role` objects, not `ClusterRole` objects, because:

1. The permissions are specific to each tenant namespace. A `developer` Role in
   `tenant-omega` should not grant access to `tenant-zeta`.
2. ClusterRoles are visible to all namespaces. If you used ClusterRoles, you would
   need to carefully name them (e.g., `tenant-omega-developer`) to avoid conflicts.
3. Roles are simpler to reason about: the Role's namespace IS the scope.

**However**, if both tenants need identical permission sets, you could define the
permission set once as a ClusterRole (e.g., `tenant-developer`) and bind it to each
namespace using a RoleBinding. This is the "ClusterRole + RoleBinding" pattern: the
ClusterRole defines the permissions, and the RoleBinding scopes them to a namespace.

```yaml
# This is an alternative approach -- reuse the same ClusterRole across namespaces
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: tenant-developer
rules:
  - apiGroups: ["apps"]
    resources: ["deployments", "replicasets"]
    verbs: ["get", "list", "watch", "create", "update", "patch"]
  - apiGroups: [""]
    resources: ["services", "configmaps", "pods", "pods/log", "secrets"]
    verbs: ["get", "list", "watch", "create", "update", "patch"]
```

Then in each tenant namespace, create a RoleBinding that references this ClusterRole:

```yaml
apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: developer-binding
  namespace: tenant-omega
subjects:
  - kind: Group
    name: omega-developers
    apiGroup: rbac.authorization.k8s.io
roleRef:
  kind: ClusterRole       # note: ClusterRole, not Role
  name: tenant-developer
  apiGroup: rbac.authorization.k8s.io
```

This grants the `tenant-developer` ClusterRole's permissions, but only within
`tenant-omega`. The `omega-developers` group does not gain access to `tenant-zeta`.

---

## Part F: ServiceAccount for Workloads

No, the application ServiceAccount should NOT have the same permissions as the
developer Role. The developer Role grants write access to ConfigMaps and Secrets,
which means a compromised pod could modify application configuration or credentials.
The application only needs to read its own configuration.

```yaml
# serviceaccount-app-omega.yaml
apiVersion: v1
kind: ServiceAccount
metadata:
  name: app-sa
  namespace: tenant-omega
```

```yaml
# role-app-omega.yaml
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: app-role
  namespace: tenant-omega
rules:
  # Application needs to read its own configuration
  - apiGroups: [""]
    resources: ["configmaps"]
    verbs: ["get", "list", "watch"]
  # Application needs to read secrets (e.g., database credentials)
  - apiGroups: [""]
    resources: ["secrets"]
    verbs: ["get", "list"]
  # Application can read its own pod info (for status reporting)
  - apiGroups: [""]
    resources: ["pods"]
    verbs: ["get", "list"]
  # Application can emit events (for Kubernetes event integration)
  - apiGroups: [""]
    resources: ["events"]
    verbs: ["create"]
```

```yaml
# rolebinding-app-omega.yaml
apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: app-binding
  namespace: tenant-omega
subjects:
  - kind: ServiceAccount
    name: app-sa
    namespace: tenant-omega
roleRef:
  kind: Role
  name: app-role
  apiGroup: rbac.authorization.k8s.io
```

**Why this works**: The application ServiceAccount has read-only access to ConfigMaps
and Secrets (it cannot modify them) and can emit events (for observability). It
cannot create, update, or delete any workload resources. If the application is
compromised, the attacker gains only read access to configuration -- not the ability
to modify deployments or exfiltrate secrets to a different namespace.

---

## Common Mistakes

1. **Setting quotas without LimitRanges**: Without LimitRanges, pods without explicit
   resource requests get zero requests but still consume real resources. The quota
   tracks requests, not actual usage, so the namespace can be overcommitted without
   the quota triggering.

2. **Using `podSelector` in NetworkPolicy without `namespaceSelector`**: If you
   only use `podSelector: {}`, you allow traffic from all pods in all namespaces
   (because `podSelector` in the `from` field, when alone, matches pods in the
   policy's namespace only -- but this subtlety is often misunderstood). Always
   be explicit about whether you mean same-namespace pods or cross-namespace pods.

3. **Forgetting that NetworkPolicy is deny-all for the selected pods**: Once a
   NetworkPolicy selects a pod (via `podSelector`), all traffic NOT explicitly
   allowed is denied. If you create a NetworkPolicy that selects all pods but
   only defines ingress rules, all egress traffic is still allowed (because
   `policyTypes` only includes `Ingress`). If you add `Egress` to `policyTypes`,
   then egress not explicitly allowed is also denied.

4. **Using ClusterRole when Role would suffice**: ClusterRoles with
   ClusterRoleBindings grant access across all namespaces. For tenant isolation,
   always use namespace-scoped Roles or ClusterRoles bound via RoleBindings.

5. **Granting workload ServiceAccounts the same permissions as developers**: The
   principle of least privilege applies to ServiceAccounts too. A web application
   does not need to create or delete deployments.

## Relevant README Sections

- [Namespace Provisioning with Labels and ResourceQuotas](../README.md#1-namespace-provisioning-with-labels-and-resourcequotas)
- [Predefined ClusterRoles for Common Personas](../README.md#2-predefined-clusterroles-for-common-personas)
- [ServiceAccount Isolation for Workloads](../README.md#4-serviceaccount-isolation-for-workloads)
- [Network Isolation per Namespace](../README.md#5-network-isolation-per-namespace)
- [Namespace-per-Environment vs Namespace-per-Team](../README.md#namespace-per-environment-vs-namespace-per-team)
