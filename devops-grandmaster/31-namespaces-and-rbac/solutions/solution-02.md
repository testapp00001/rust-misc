# Solution 02: Team Namespaces with RBAC

## Part A: Create the Namespaces

```yaml
# namespace-alpha.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: alpha
  labels:
    team: alpha
    managed-by: platform-team
```

```yaml
# namespace-beta.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: beta
  labels:
    team: beta
    managed-by: platform-team
```

```bash
kubectl apply -f namespace-alpha.yaml
kubectl apply -f namespace-beta.yaml
```

**Why this works**: Namespaces are cluster-scoped resources. Labels are metadata
that other resources (NetworkPolicies, ResourceQuotas, RBAC bindings) can use for
selection. The `managed-by` label identifies namespaces provisioned by the platform
team vs. those created ad-hoc.

---

## Part B: Create the Developer Role

```yaml
# role-developer-alpha.yaml
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: developer
  namespace: alpha
rules:
  # Core workload resources: full CRUD
  - apiGroups: ["apps"]
    resources: ["deployments", "replicasets"]
    verbs: ["get", "list", "watch", "create", "update", "patch"]
  # Core API group resources
  - apiGroups: [""]
    resources: ["services", "configmaps", "pods", "pods/log"]
    verbs: ["get", "list", "watch", "create", "update", "patch"]
  # Debugging: exec into pods
  - apiGroups: [""]
    resources: ["pods/exec"]
    verbs: ["create"]
  # Secrets: read-only
  - apiGroups: [""]
    resources: ["secrets"]
    verbs: ["get", "list"]
```

**Why this works**:

- The Role is namespace-scoped (`metadata.namespace: alpha`). It only grants
  permissions within namespace `alpha`.
- `deployments` and `replicasets` are in the `apps` API group. `services`, `pods`,
  `configmaps`, `secrets`, `pods/exec`, and `pods/log` are in the core API group (`""`).
- `pods/exec` requires only the `create` verb -- that is how `kubectl exec` works
  under the hood (it creates an exec session on the pod).
- `delete` is intentionally excluded from all resources. Developers cannot destroy
  resources -- they can only update or patch them. This prevents accidental deletions.
- Secrets are read-only (`get`, `list`). Developers can see secret names and values
  (for debugging) but cannot modify them. In a stricter environment, you might
  remove even `get` and require developers to use a secrets management tool.

---

## Part C: Bind the Role to Users

```yaml
# rolebinding-alpha.yaml
apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: developer-binding
  namespace: alpha
subjects:
  - kind: User
    name: alice@company.com
    apiGroup: rbac.authorization.k8s.io
roleRef:
  kind: Role
  name: developer
  apiGroup: rbac.authorization.k8s.io
```

```yaml
# rolebinding-beta.yaml
apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: developer-binding
  namespace: beta
subjects:
  - kind: User
    name: bob@company.com
    apiGroup: rbac.authorization.k8s.io
roleRef:
  kind: Role
  name: developer
  apiGroup: rbac.authorization.k8s.io
```

**Why this works**:

- The RoleBinding is in the same namespace as the Role. This means the binding
  applies only within that namespace.
- `roleRef.kind: Role` tells Kubernetes to look for the Role named `developer` in
  the same namespace as the RoleBinding.
- The `subjects` field identifies who gets the permissions. For Users, the `name`
  must match the identity from the authentication system (OIDC, client certificate,
  etc.).
- Note that `RoleBinding` names are scoped to their namespace, so both bindings
  can be named `developer-binding` without conflict.

---

## Part D: Verify Isolation

| # | Command | Expected | Why |
|---|---------|----------|-----|
| 1 | `kubectl auth can-i create deployments --namespace=alpha --as=alice@company.com` | yes | Alice has the `developer` Role in `alpha` via RoleBinding |
| 2 | `kubectl auth can-i create deployments --namespace=beta --as=alice@company.com` | no | Alice has no RoleBinding in `beta` |
| 3 | `kubectl auth can-i list pods --namespace=alpha --as=bob@company.com` | no | Bob has no RoleBinding in `alpha` |
| 4 | `kubectl auth can-i list pods --namespace=beta --as=bob@company.com` | yes | Bob has the `developer` Role in `beta` via RoleBinding |
| 5 | `kubectl auth can-i delete namespaces --as=alice@company.com` | no | Alice has no ClusterRoleBinding granting namespace delete |
| 6 | `kubectl auth can-i delete secrets --namespace=alpha --as=alice@company.com` | no | The `developer` Role only grants `get` and `list` on secrets |

**Why this works**: RBAC is deny-by-default. There is no rule allowing Alice to
access `beta`, so the answer is "no." There is no rule allowing `delete` on secrets,
so the answer is "no." The `--as` flag simulates the identity without requiring
real credentials.

---

## Part E: Create a ServiceAccount

```yaml
# serviceaccount-deploy-bot.yaml
apiVersion: v1
kind: ServiceAccount
metadata:
  name: deploy-bot
  namespace: alpha
```

```yaml
# rolebinding-deploy-bot.yaml
apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: deploy-bot-binding
  namespace: alpha
subjects:
  - kind: ServiceAccount
    name: deploy-bot
    namespace: alpha
roleRef:
  kind: Role
  name: developer
  apiGroup: rbac.authorization.k8s.io
```

```bash
kubectl auth can-i list pods --namespace=alpha \
  --as=system:serviceaccount:alpha:deploy-bot
# yes
```

**Why this works**:

- ServiceAccounts are namespace-scoped. The name format for `--as` is
  `system:serviceaccount:<namespace>:<name>`.
- In the RoleBinding subject, `kind: ServiceAccount` tells Kubernetes this is a
  ServiceAccount, not a User. The `namespace` field in the subject must match the
  ServiceAccount's actual namespace.
- The ServiceAccount inherits the same permissions as the `developer` Role through
  the same RoleBinding mechanism used for Users.

---

## Common Mistakes

1. **Forgetting `apiGroups` for Deployments**: Deployments are in the `apps` API
   group, not the core (`""`) API group. If you put `deployments` under `apiGroups: [""]`,
   the Role will not grant access to Deployments and you will get "forbidden" errors.

2. **Using `ClusterRoleBinding` instead of `RoleBinding`**: If you use a
   ClusterRoleBinding to bind the Role, the permissions would apply across all
   namespaces, defeating the purpose of isolation. (Note: ClusterRoleBinding
   actually requires a ClusterRole reference, not a Role reference -- Kubernetes
   will reject a ClusterRoleBinding that references a namespace-scoped Role.)

3. **Not specifying `namespace` in ServiceAccount subjects**: When a RoleBinding
   references a ServiceAccount in the same namespace, you still need to specify
   the `namespace` field in the subject. Without it, Kubernetes assumes the
   ServiceAccount is in the `default` namespace.

4. **Granting `delete` "for convenience"**: It is tempting to add `delete` to the
   verbs list so developers can clean up. But `delete` is a destructive operation
   that should be gated. Use `update` and `patch` for modifications, and reserve
   `delete` for controlled processes.

5. **Confusing `pods/log` with `pods`**: `pods/log` is a subresource. Granting
   access to `pods` does not automatically grant access to `pods/log`. You must
   list `pods/log` as a separate resource if you want to allow log reading.

## Relevant README Sections

- [Step 1: Create Namespaces per Team or Environment](../README.md#step-1-create-namespaces-per-team-or-environment)
- [Step 3: Create a Role (namespace-scoped permissions)](../README.md#step-3-create-a-role-namespace-scoped-permissions)
- [Step 4: Bind the Role to Users or Groups](../README.md#step-4-bind-the-role-to-users-or-groups)
- [Step 5: Verify Permissions](../README.md#step-5-verify-permissions)
