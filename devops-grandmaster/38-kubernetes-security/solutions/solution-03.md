# Solution 03: Implement RBAC with Least Privilege

## Part A -- Namespace and ServiceAccounts

```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: backend
---
apiVersion: v1
kind: ServiceAccount
metadata:
  name: backend-dev-sa
  namespace: backend
---
apiVersion: v1
kind: Namespace
metadata:
  name: sre
---
apiVersion: v1
kind: ServiceAccount
metadata:
  name: sre-sa
  namespace: sre
```

## Part B -- Backend Devs RBAC

```yaml
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: backend-dev-role
  namespace: backend
rules:
  # Deployments and ReplicaSets (apps API group)
  - apiGroups: ["apps"]
    resources: ["deployments", "replicasets"]
    verbs: ["get", "list", "watch", "create", "update", "delete"]
  # Pods (core API group)
  - apiGroups: [""]
    resources: ["pods"]
    verbs: ["get", "list", "watch", "create", "update", "delete"]
  # Pod logs (subresource)
  - apiGroups: [""]
    resources: ["pods/log"]
    verbs: ["get"]
  # Pod exec (subresource for debugging)
  - apiGroups: [""]
    resources: ["pods/exec"]
    verbs: ["create"]
  # ConfigMaps -- full read, no write
  - apiGroups: [""]
    resources: ["configmaps"]
    verbs: ["get", "list", "watch"]
  # Secrets -- read only
  - apiGroups: [""]
    resources: ["secrets"]
    verbs: ["get", "list", "watch"]
---
apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: backend-dev-rolebinding
  namespace: backend
subjects:
  - kind: ServiceAccount
    name: backend-dev-sa
    namespace: backend
roleRef:
  kind: Role
  name: backend-dev-role
  apiGroup: rbac.authorization.k8s.io
```

## Part C -- SRE Team RBAC

```yaml
# Cluster-wide read-only ClusterRole
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: sre-readonly
rules:
  - apiGroups: [""]
    resources: ["*"]
    verbs: ["get", "list", "watch"]
  - apiGroups: ["apps"]
    resources: ["*"]
    verbs: ["get", "list", "watch"]
  - apiGroups: ["batch"]
    resources: ["*"]
    verbs: ["get", "list", "watch"]
  - apiGroups: ["networking.k8s.io"]
    resources: ["*"]
    verbs: ["get", "list", "watch"]
  - apiGroups: ["rbac.authorization.k8s.io"]
    resources: ["*"]
    verbs: ["get", "list", "watch"]
---
# Bind the read-only ClusterRole to the SRE ServiceAccount
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRoleBinding
metadata:
  name: sre-readonly-binding
subjects:
  - kind: ServiceAccount
    name: sre-sa
    namespace: sre
roleRef:
  kind: ClusterRole
  name: sre-readonly
  apiGroup: rbac.authorization.k8s.io
---
# Namespace-scoped Role for pod deletion in backend
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: sre-pod-delete
  namespace: backend
rules:
  - apiGroups: [""]
    resources: ["pods"]
    verbs: ["delete", "deletecollection"]
---
# Bind the pod-delete Role to the SRE ServiceAccount
apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: sre-pod-delete-binding
  namespace: backend
subjects:
  - kind: ServiceAccount
    name: sre-sa
    namespace: sre
roleRef:
  kind: Role
  name: sre-pod-delete
  apiGroup: rbac.authorization.k8s.io
```

## Why It Works

### Backend Devs

The Role uses separate rules for different API groups. Deployments and
ReplicaSets are in the `apps` group, while Pods, ConfigMaps, and Secrets
are in the core group (empty string `""`). The `pods/log` and `pods/exec`
subresources are listed separately because Kubernetes treats subresources
as distinct resource paths.

Secrets access is limited to read-only verbs (`get`, `list`, `watch`). The
team can inspect configuration but cannot modify secrets, which prevents
accidental credential rotation or corruption.

### SRE Team

The SRE team gets cluster-wide read through a ClusterRole bound via a
ClusterRoleBinding. The ClusterRole uses `resources: ["*"]` with explicit
verbs (`get`, `list`, `watch`) rather than `verbs: ["*"]`, which would
grant write access.

For the namespace-scoped pod deletion in `backend`, a separate Role and
RoleBinding are created. Kubernetes unions permissions from all bindings,
so the SRE account has both cluster-wide read and namespace-scoped delete
on pods in `backend`.

### Least Privilege Principles Applied

- No wildcard verbs are used -- every verb is explicitly listed.
- No `escalate`, `bind`, or `impersonate` verbs are granted.
- Backend devs cannot modify Roles, RoleBindings, or ServiceAccounts.
- Backend devs cannot access other namespaces (the Role is namespace-scoped).
- SRE cannot create or modify any resources cluster-wide (read-only
  ClusterRole).

## Verification

```bash
# Apply all manifests
kubectl apply -f part-a.yaml
kubectl apply -f part-b.yaml
kubectl apply -f part-c.yaml

# Run all can-i checks from the exercise
kubectl auth can-i create deployments \
  --as=system:serviceaccount:backend:backend-dev-sa -n backend
# -> yes

kubectl auth can-i get secrets \
  --as=system:serviceaccount:backend:backend-dev-sa -n backend
# -> yes

kubectl auth can-i delete namespaces \
  --as=system:serviceaccount:backend:backend-dev-sa
# -> no

kubectl auth can-i create roles \
  --as=system:serviceaccount:backend:backend-dev-sa -n backend
# -> no

kubectl auth can-i get pods \
  --as=system:serviceaccount:sre:sre-sa -n kube-system
# -> yes

kubectl auth can-i delete pods \
  --as=system:serviceaccount:sre:sre-sa -n backend
# -> yes

kubectl auth can-i create deployments \
  --as=system:serviceaccount:sre:sre-sa -n backend
# -> no
```

## Common Mistakes

- **Using `apiGroups: [""]` for Deployments.** Deployments are in the
  `apps` group. Using the core group means the Role silently grants nothing
  for Deployments.
- **Forgetting `pods/log` and `pods/exec`.** Without these subresources,
  `kubectl logs` and `kubectl exec` fail with forbidden errors even though
  the user can see the pods.
- **Using `verbs: ["*"]` in the SRE ClusterRole.** This would grant write
  access, violating least privilege.
- **Creating only a RoleBinding for the SRE team.** A RoleBinding is
  namespace-scoped and cannot grant cluster-wide access. A ClusterRole
  must be bound with a ClusterRoleBinding for cluster-wide effect.
- **Granting `delete` on pods to backend-devs.** The scenario does not
  require this. It could be dangerous because deleting pods forces restarts
  and can mask deployment issues.
- **Not checking with `kubectl auth can-i`.** Always verify. RBAC errors
  only surface when a user actually tries an operation.
