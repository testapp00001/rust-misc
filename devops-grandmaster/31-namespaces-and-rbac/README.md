# 31 — Namespaces & RBAC for Multi-Tenancy

> **Previous:** [30 — ConfigMaps & Secrets](../30-configmaps-and-secrets/) |
> **Next:** [32 — Persistent Volumes](../32-persistent-volumes/)

---

## Problem

Your Kubernetes cluster hosts multiple teams, environments, or customers. Everyone
shares the same default namespace. One developer accidentally deletes another team's
deployment. A junior engineer modifies production resources when they should only
touch staging. There is no isolation, no access control, and no audit trail for who
did what.

You need logical separation of workloads and fine-grained permissions so that each
team can only see and modify their own resources.

---

## Naive Way — Everything in `default`, Everyone Is `cluster-admin`

```bash
# Every team deploys to the default namespace
kubectl apply -f team-a-deployment.yaml
kubectl apply -f team-b-deployment.yaml

# Give everyone the cluster-admin ClusterRoleBinding
kubectl create clusterbinding admin-all \
  --clusterrole=cluster-admin \
  --user=developer@company.com
```

**Why this fails:**

- Any user can delete any resource in any namespace.
- Resource names collide (`team-a` and `team-b` both want a `redis` Deployment).
- No audit trail — everything is mixed together.
- A single misconfigured script can destroy the entire cluster.
- You cannot set resource quotas per team.

---

## Right Way — Namespaces + RBAC Roles

### Step 1: Create Namespaces per Team or Environment

```yaml
# namespace-team-a.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: team-a
  labels:
    team: team-a
    environment: dev
```

```bash
kubectl apply -f namespace-team-a.yaml
kubectl apply -f namespace-team-b.yaml
```

### Step 2: Deploy Workloads into Namespaced Scope

```yaml
# deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: api-server
  namespace: team-a          # <-- scoped to team-a
spec:
  replicas: 3
  selector:
    matchLabels:
      app: api-server
  template:
    metadata:
      labels:
        app: api-server
    spec:
      containers:
        - name: api-server
          image: myregistry/api-server:v1.2.0
```

### Step 3: Create a Role (namespace-scoped permissions)

```yaml
# role-team-a-developer.yaml
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: developer
  namespace: team-a
rules:
  - apiGroups: ["apps", ""]
    resources: ["deployments", "services", "configmaps", "pods", "pods/log"]
    verbs: ["get", "list", "watch", "create", "update", "patch"]
  - apiGroups: [""]
    resources: ["pods/exec"]
    verbs: ["create"]                         # kubectl exec for debugging
  - apiGroups: [""]
    resources: ["secrets"]
    verbs: ["get", "list"]                    # read-only on secrets
```

### Step 4: Bind the Role to Users or Groups

```yaml
# rolebinding-team-a-developer.yaml
apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: developer-binding
  namespace: team-a
subjects:
  - kind: Group
    name: team-a-developers        # matches OIDC group claim
    apiGroup: rbac.authorization.k8s.io
roleRef:
  kind: Role
  name: developer
  apiGroup: rbac.authorization.k8s.io
```

### Step 5: Verify Permissions

```bash
# Test what a user can do
kubectl auth can-i create deployments --namespace=team-a --as=alice@company.com
# yes

kubectl auth can-i delete deployments --namespace=team-b --as=alice@company.com
# no

kubectl auth can-i list pods --all-namespaces --as=alice@company.com
# no
```

### RBAC Object Hierarchy

```
ClusterRole / Role           ClusterRoleBinding / RoleBinding
       |                              |
  "what can be done"          "who can do it, and where"
       |                              |
  apiGroups, resources,       subject (User/Group/ServiceAccount)
  verbs, resourceNames        + namespace scope (RoleBinding)
                              or cluster-wide (ClusterRoleBinding)
```

**Key distinction:**

| Object | Scope |
|---|---|
| `Role` | Single namespace |
| `ClusterRole` | Cluster-wide (or granted via RoleBinding to a single namespace) |
| `RoleBinding` | Grants a Role/ClusterRole within a specific namespace |
| `ClusterRoleBinding` | Grants a ClusterRole across all namespaces |

---

## Production Way — Full Multi-Tenant Setup

### 1. Namespace Provisioning with Labels and ResourceQuotas

```yaml
apiVersion: v1
kind: ResourceQuota
metadata:
  name: team-a-quota
  namespace: team-a
spec:
  hard:
    requests.cpu: "20"
    requests.memory: 40Gi
    limits.cpu: "40"
    limits.memory: 80Gi
    persistentvolumeclaims: "10"
    services.loadbalancers: "2"
    pods: "100"
---
apiVersion: v1
kind: LimitRange
metadata:
  name: default-limits
  namespace: team-a
spec:
  limits:
    - default:
        cpu: "500m"
        memory: 512Mi
      defaultRequest:
        cpu: "100m"
        memory: 128Mi
      type: Container
```

### 2. Predefined ClusterRoles for Common Personas

```yaml
# clusterrole-namespace-admin.yaml
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: namespace-admin
rules:
  - apiGroups: ["*"]
    resources: ["*"]
    verbs: ["*"]
  - nonResourceURLs: ["*"]
    verbs: ["get"]              # read-only on non-resource URLs
```

```yaml
# clusterrole-readonly.yaml
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: readonly
rules:
  - apiGroups: [""]
    resources: ["pods", "services", "configmaps", "events"]
    verbs: ["get", "list", "watch"]
  - apiGroups: ["apps"]
    resources: ["deployments", "replicasets", "statefulsets"]
    verbs: ["get", "list", "watch"]
```

### 3. OIDC / Identity Provider Integration

```yaml
# kube-apiserver flags (EKS, GKE, or kubeadm)
--oidc-issuer-url=https://auth.company.com
--oidc-client-id=kubernetes
--oidc-username-claim=email
--oidc-groups-claim=groups
```

Now RBAC bindings can reference groups from your IdP:

```yaml
subjects:
  - kind: Group
    name: "platform-admins"       # group in Okta/Auth0/Google
    apiGroup: rbac.authorization.k8s
```

### 4. ServiceAccount Isolation for Workloads

```yaml
apiVersion: v1
kind: ServiceAccount
metadata:
  name: api-server-sa
  namespace: team-a
---
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: api-server-role
  namespace: team-a
rules:
  - apiGroups: [""]
    resources: ["configmaps"]
    resourceNames: ["api-server-config"]
    verbs: ["get", "watch"]       # read-only on a specific ConfigMap
---
apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: api-server-binding
  namespace: team-a
subjects:
  - kind: ServiceAccount
    name: api-server-sa
    namespace: team-a
roleRef:
  kind: Role
  name: api-server-role
  apiGroup: rbac.authorization.k8s.io
```

### 5. Network Isolation per Namespace

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: default-deny-ingress
  namespace: team-a
spec:
  podSelector: {}
  policyTypes:
    - Ingress
  ingress:
    - from:
        - namespaceSelector:
            matchLabels:
              team: team-a       # only allow traffic from same team
```

### 6. Automated Namespace Provisioning (Terraform / Controller)

```hcl
# Using the Kubernetes Terraform provider
resource "kubernetes_namespace" "team_a" {
  metadata {
    name = "team-a"
    labels = {
      team        = "team-a"
      environment = "dev"
      managed-by  = "terraform"
    }
  }
}

resource "kubernetes_role" "team_a_dev" {
  metadata {
    name      = "developer"
    namespace = kubernetes_namespace.team_a.metadata[0].name
  }
  rule {
    api_groups = ["apps", ""]
    resources  = ["deployments", "services", "pods", "configmaps"]
    verbs      = ["get", "list", "watch", "create", "update", "patch"]
  }
}
```

### Namespace-per-Environment vs Namespace-per-Team

```
Option A: Namespace-per-environment
  dev/
  staging/
  production/
  Each contains ALL teams' workloads.

Option B: Namespace-per-team
  team-a/
  team-b/
  team-c/
  Each contains ALL environments.

Option C: Namespace-per-team-per-environment  (recommended)
  team-a-dev/
  team-a-staging/
  team-a-production/
  team-b-dev/
  team-b-staging/
  team-b-production/
  Maximum isolation, per-team quotas per environment.
```

---

## Hands-On Lab

### Lab: Multi-Tenant RBAC on a Local Cluster

```bash
# 1. Create a local cluster with RBAC enabled
kind create cluster --name rbac-lab --config - <<EOF
kind: Cluster
apiVersion: kind.x-k8s.io/v1alpha4
nodes:
  - role: control-plane
  - role: worker
EOF

# 2. Create namespaces
kubectl create namespace alpha
kubectl create namespace beta

# 3. Create a developer Role for namespace alpha
kubectl apply -f - <<EOF
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: dev-role
  namespace: alpha
rules:
  - apiGroups: ["", "apps"]
    resources: ["pods", "services", "deployments"]
    verbs: ["get", "list", "watch", "create", "update", "patch"]
EOF

# 4. Bind it to a user
kubectl create rolebinding dev-binding \
  --role=dev-role \
  --user=alice \
  --namespace=alpha

# 5. Verify: Alice can deploy in alpha
kubectl auth can-i create deployments --namespace=alpha --as=alice
# yes

# 6. Verify: Alice cannot deploy in beta
kubectl auth can-i create deployments --namespace=beta --as=alice
# no

# 7. Verify: Alice cannot delete cluster-scoped resources
kubectl auth can-i delete nodes --as=alice
# no

# 8. Deploy a workload as Alice (simulated)
kubectl create deployment nginx --image=nginx --namespace=alpha --as=alice

# 9. Apply a ResourceQuota
kubectl apply -f - <<EOF
apiVersion: v1
kind: ResourceQuota
metadata:
  name: alpha-quota
  namespace: alpha
spec:
  hard:
    pods: "10"
    requests.cpu: "4"
    requests.memory: 8Gi
EOF

# 10. Verify quota enforcement
kubectl run test-pod --image=nginx --namespace=alpha --as=alice
kubectl describe resourcequota -n alpha
```

### Lab: Audit RBAC Permissions

```bash
# List all Roles and ClusterRoles
kubectl get roles,clusterroles --all-namespaces

# List all bindings
kubectl get rolebindings,clusterrolebindings --all-namespaces

# Check what a service account can do
kubectl auth can-i --list --as=system:serviceaccount:team-a:api-server-sa

# Find overly permissive bindings
kubectl get clusterrolebindings -o json | \
  jq '.items[] | select(.roleRef.name=="cluster-admin") | .metadata.name'
```

---

## Limitation

Namespaces and RBAC provide logical isolation for **stateless** workloads, but they
do not address **stateful** concerns. When your application needs to persist data
beyond the lifecycle of a Pod — databases, file uploads, message queues — you need
storage that survives Pod restarts, rescheduling, and node failures. Kubernetes
namespaces cannot scope PersistentVolumes (they are cluster-scoped), and RBAC
alone does not enforce storage policies.

---

## Next Topic

**[32 — Persistent Volumes](../32-persistent-volumes/)** — Dynamic provisioning,
StorageClasses, PVC lifecycle, StatefulSets, and the CSI driver ecosystem.
