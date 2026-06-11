# Solution 03: Least-Privilege CI/CD Pipeline Access

## Part A: Design the ServiceAccount

```yaml
apiVersion: v1
kind: ServiceAccount
metadata:
  name: cicd-deployer
  namespace: webapp-staging
```

**Why this works**: This is the identity the CI/CD pipeline will use. The pipeline
authenticates using a token associated with this ServiceAccount (either a long-lived
Secret-based token or a short-lived TokenRequest token). All subsequent RBAC checks
are performed against this identity.

---

## Part B: Define the Role

```yaml
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: cicd-deployer-role
  namespace: webapp-staging
rules:
  # Requirement 1: Deploy new container images by updating Deployments
  # We need get (to read current state), list (to find deployments),
  # watch (for rollout status), update and patch (to change image/tag).
  # Intentionally NO create (deployments must already exist) and NO delete.
  - apiGroups: ["apps"]
    resources: ["deployments"]
    verbs: ["get", "list", "watch", "update", "patch"]

  # Also need to read ReplicaSets and Pods to verify rollout status
  - apiGroups: ["apps"]
    resources: ["replicasets"]
    verbs: ["get", "list", "watch"]
  - apiGroups: [""]
    resources: ["pods"]
    verbs: ["get", "list", "watch"]

  # Requirement 2: Create and update ConfigMaps and Secrets
  # The pipeline creates or updates configuration for the application.
  # get and list are needed to check if the resource already exists.
  - apiGroups: [""]
    resources: ["configmaps", "secrets"]
    verbs: ["get", "list", "create", "update", "patch"]

  # Requirement 3: Check pod status and read pod logs
  # pods/log is a subresource -- must be listed separately from pods
  - apiGroups: [""]
    resources: ["pods/log"]
    verbs: ["get"]

  # Requirement 4: Create Jobs for database migrations
  # The pipeline creates a Job, then watches for completion.
  - apiGroups: ["batch"]
    resources: ["jobs"]
    verbs: ["get", "list", "watch", "create"]
```

**Why this works**:

- Each rule maps directly to a pipeline requirement. There are no extraneous
  permissions.
- `delete` is not included on Deployments because rollback is a manual process.
- `create` is not included on Deployments because the pipeline only updates
  existing deployments (it changes the container image, not the deployment spec).
- `pods/exec` is not included because the pipeline does not need to exec into pods.
- `pods/portforward` is not included because the pipeline does not need port-forwarding.
- RBAC objects (roles, rolebindings, serviceaccounts) are not listed, so the
  pipeline cannot modify access controls.
- Jobs only get `create` and read verbs, not `delete` or `patch` (migrations should
  be idempotent and non-modifiable once created).

---

## Part C: Create the RoleBinding

```yaml
apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: cicd-deployer-binding
  namespace: webapp-staging
subjects:
  - kind: ServiceAccount
    name: cicd-deployer
    namespace: webapp-staging
roleRef:
  kind: Role
  name: cicd-deployer-role
  apiGroup: rbac.authorization.k8s.io
```

**Why this works**: The RoleBinding connects the ServiceAccount (who) to the Role
(what) within the `webapp-staging` namespace (where). The ServiceAccount gets exactly
the permissions defined in the Role, and only within this namespace.

---

## Part D: Verify the Permissions

```bash
SA="system:serviceaccount:webapp-staging:cicd-deployer"

# Update a deployment in webapp-staging -> yes
kubectl auth can-i update deployments \
  --namespace=webapp-staging --as=$SA

# Delete a deployment in webapp-staging -> no
kubectl auth can-i delete deployments \
  --namespace=webapp-staging --as=$SA

# Create a Secret in webapp-staging -> yes
kubectl auth can-i create secrets \
  --namespace=webapp-staging --as=$SA

# List Secrets in default -> no
kubectl auth can-i list secrets \
  --namespace=default --as=$SA

# Create a Job in webapp-staging -> yes
kubectl auth can-i create jobs \
  --namespace=webapp-staging --as=$SA

# Exec into a pod in webapp-staging -> no
kubectl auth can-i create pods/exec \
  --namespace=webapp-staging --as=$SA

# Create a Role in webapp-staging -> no
kubectl auth can-i create roles \
  --namespace=webapp-staging --as=$SA

# List namespaces -> no
kubectl auth can-i list namespaces --as=$SA
```

---

## Part E: Explain the Denied Permissions

### "Delete deployment" -- Denied because verb is missing

The Role grants `get`, `list`, `watch`, `update`, `patch` on deployments, but not
`delete`. RBAC is deny-by-default, so `delete` is implicitly denied. There is no
"deny" rule needed -- the absence of an "allow" rule is sufficient.

### "List secrets in default" -- Denied because namespace is wrong

The Role and RoleBinding exist in namespace `webapp-staging`. The ServiceAccount has
no permissions in the `default` namespace because there is no RoleBinding in `default`
referencing any Role that the ServiceAccount is a subject of. RBAC permissions are
namespace-scoped when granted through Role (not ClusterRole).

### "Exec into pod" -- Denied because resource is missing

The Role does not list `pods/exec` in any rule. Even though the Role grants access
to `pods` (get, list, watch), `pods/exec` is a separate subresource that must be
explicitly listed. Access to `pods` does not imply access to `pods/exec`.

### "Create Role" -- Denied because RBAC objects are not listed

The Role does not include any RBAC API group resources (`roles`, `rolebindings`,
`clusterroles`, `clusterrolebindings`). Since there is no rule allowing access to
these resources, the action is denied. This is the privilege escalation prevention:
a ServiceAccount cannot grant itself additional permissions.

### "List namespaces" -- Denied because namespaces are cluster-scoped

Namespaces are cluster-scoped resources. The `cicd-deployer-role` is a namespace-scoped
Role in `webapp-staging`. Namespace-scoped Roles cannot grant access to cluster-scoped
resources. To access namespaces, the ServiceAccount would need a ClusterRole bound
via a ClusterRoleBinding, which we intentionally did not create.

---

## Common Mistakes

1. **Granting `delete` "for emergencies"**: If the pipeline needs rollback, implement
   it as a separate, gated workflow with a different ServiceAccount that has `delete`
   permission. Never give the deploy pipeline destructive access "just in case."

2. **Using `*` for apiGroups or verbs**: Wildcards grant access to current and
   future resources. If Kubernetes adds a new API group or resource type, the
   pipeline automatically gains access to it. Always enumerate.

3. **Forgetting `pods/log`**: If you only grant access to `pods`, the pipeline
   cannot read logs. `pods/log` is a subresource and must be explicitly listed.
   This is a common source of "forbidden" errors in CI/CD pipelines.

4. **Granting access to RBAC objects**: Even read access to RBAC objects leaks
   information about your access control structure. The CI/CD pipeline has no
   reason to read or modify Roles.

5. **Using ClusterRole instead of Role**: A ClusterRole with a ClusterRoleBinding
   would give the pipeline access to all namespaces. Use a namespace-scoped Role
   with a RoleBinding to limit the blast radius to `webapp-staging`.

6. **Not distinguishing between "create" and "update" on Deployments**: If the
   pipeline can create deployments, it might create unexpected deployments. If it
   can only update, it can only modify existing ones -- which means someone must
   create the initial deployment through a separate, controlled process.

## Relevant README Sections

- [Step 3: Create a Role (namespace-scoped permissions)](../README.md#step-3-create-a-role-namespace-scoped-permissions)
- [Step 4: Bind the Role to Users or Groups](../README.md#step-4-bind-the-role-to-users-or-groups)
- [Step 5: Verify Permissions](../README.md#step-5-verify-permissions)
- [ServiceAccount Isolation for Workloads](../README.md#4-serviceaccount-isolation-for-workloads)
