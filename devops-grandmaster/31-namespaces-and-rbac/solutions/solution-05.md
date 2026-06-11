# Solution 05: RBAC Audit and Remediation

## Part A: Discovery -- Map the Current State

### 1. ClusterRoleBindings to cluster-admin

```bash
kubectl get clusterrolebindings -o json | \
  jq '.items[] | select(.roleRef.name=="cluster-admin") |
  select(.metadata.name | startswith("system:") | not) |
  {name: .metadata.name, subjects: .subjects}'
```

**What this tells you**: Any non-system ClusterRoleBinding to `cluster-admin` is a
critical finding. The built-in `cluster-admin` ClusterRole grants `*` on `*` with
`*` verbs. Human users and application ServiceAccounts should never be directly
bound to it. Common non-system bindings include `admin-all`, `dev-admin`, or
bindings created by `kubectl create clusterrolebinding` during initial setup.

### 2. Stale ServiceAccounts

```bash
# Get all non-default ServiceAccounts
kubectl get serviceaccounts --all-namespaces -o json | \
  jq -r '.items[] | select(.metadata.name != "default") |
  "\(.metadata.namespace)/\(.metadata.name)"' | sort > /tmp/all-sa.txt

# Get all ServiceAccounts referenced by pods
kubectl get pods --all-namespaces -o json | \
  jq -r '.items[] | "\(.metadata.namespace)/\(.spec.serviceAccountName)"' | \
  sort -u > /tmp/active-sa.txt

# Find stale ones
comm -23 /tmp/all-sa.txt /tmp/active-sa.txt
```

**What this tells you**: ServiceAccounts with no pods are either:
- Stale (the workload was deleted but the ServiceAccount was not cleaned up).
- Pre-provisioned (the ServiceAccount was created for a workload that has not
  been deployed yet).
- CI/CD or automation accounts (used by pipelines, not by pods directly).

The second and third cases are valid. The first case is dead code that should
be removed to reduce attack surface.

### 3. Wildcard Permissions

```bash
# ClusterRoles with wildcard resources
kubectl get clusterroles -o json | \
  jq '.items[] | select(.rules[]? | .resources[]? == "*") |
  .metadata.name' | grep -v "^system:"

# Roles with wildcard verbs
kubectl get roles --all-namespaces -o json | \
  jq -r '.items[] | select(.rules[]? | .verbs[]? == "*") |
  "\(.metadata.namespace)/\(.metadata.name)"'
```

**What this tells you**: Wildcard permissions are the most common source of
over-privilege. A ClusterRole with `resources: ["*"]` can access any current or
future resource type, including CRDs. A Role with `verbs: ["*"]` grants
`delete`, `deletecollection`, and any future verbs.

### 4. Users and Groups in Bindings

```bash
kubectl get rolebindings,clusterrolebindings --all-namespaces -o json | \
  jq '[.items[].subjects[]? |
  select(.kind == "User" or .kind == "Group") |
  {kind: .kind, name: .name}] | unique_by(.name) | .[]'
```

**What this tells you**: Look for:
- `test@`, `temp@`, `debug@` accounts (should not exist in production).
- Users who have left the company (cross-reference with HR/IdP).
- Groups with names like `everyone`, `all-users` (too broad).
- Typos in group names (e.g., `platfrom-admins` instead of `platform-admins`).

---

## Part B: Risk Assessment

| Finding | Risk | Reasoning |
|---------|------|-----------|
| User bound to `cluster-admin` via ClusterRoleBinding | **Critical** | Direct path to full cluster compromise. Any code running as this user (including browser-based tools, CI/CD pipelines) can delete all namespaces, modify RBAC, and exfiltrate secrets. |
| ServiceAccount bound to `cluster-admin` | **Critical** | Any pod using this ServiceAccount is effectively root on the cluster. If the pod is compromised, the attacker gains full cluster control. |
| ClusterRole with `resources: ["*"]` and `verbs: ["*"]` | **Critical** | Equivalent to cluster-admin but harder to detect. Grants access to all current and future resource types. |
| ClusterRole with `verbs: ["*"]` on secrets cluster-wide | **High** | Can read and modify all secrets in all namespaces. This includes TLS certificates, database credentials, API keys, and cloud provider tokens. |
| Role with wildcard verbs in a single namespace | **Medium** | The blast radius is limited to one namespace, but within that namespace the subject can do anything, including `deletecollection` which wipes all resources of a type at once. |
| Stale ServiceAccount with active RoleBinding | **Medium** | The ServiceAccount is not used, but its permissions still exist. If someone creates a pod that accidentally uses this ServiceAccount, it gains the bound permissions. |
| Stale ServiceAccount with no bindings | **Low** | No permissions, but still a potential confusion point. Remove to keep the namespace clean. |
| Test user accounts in bindings | **Low-Medium** | Depends on the permissions. A test user with read-only access is low risk. A test user with write access to production namespaces is high risk. |

---

## Part C: Remediation -- Replace Wildcard Roles

### Safe Order of Operations

1. **Create the new scoped ClusterRole** (no disruption -- adding a new role does
   not affect existing bindings).
2. **Create namespace-scoped RoleBindings** referencing the new ClusterRole (no
   disruption -- bindings are additive; developers now have both old and new access).
3. **Verify** that developers can still work with the new bindings (`kubectl auth can-i`).
4. **Delete the old ClusterRoleBinding** (developers lose the wildcard access but
   retain the new scoped access).
5. **Delete the old ClusterRole** (cleanup).

### New Scoped ClusterRole

```yaml
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: developer-scoped
rules:
  # Deployments: CRUD, no deletecollection
  - apiGroups: ["apps"]
    resources: ["deployments", "replicasets"]
    verbs: ["get", "list", "watch", "create", "update", "patch"]
  # Core workload resources: CRUD
  - apiGroups: [""]
    resources: ["services", "configmaps", "pods", "pods/log"]
    verbs: ["get", "list", "watch", "create", "update", "patch"]
  # Secrets: read-only
  - apiGroups: [""]
    resources: ["secrets"]
    verbs: ["get", "list"]
  # Events: read-only (for debugging)
  - apiGroups: [""]
    resources: ["events"]
    verbs: ["get", "list", "watch"]
```

### Namespace-Scoped RoleBindings

```yaml
# Bind to the dev-team group in the development namespace
apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: dev-team-binding
  namespace: development
subjects:
  - kind: Group
    name: dev-team
    apiGroup: rbac.authorization.k8s.io
roleRef:
  kind: ClusterRole
  name: developer-scoped
  apiGroup: rbac.authorization.k8s.io
```

```yaml
# Bind to the dev-team group in the staging namespace
apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: dev-team-binding
  namespace: staging
subjects:
  - kind: Group
    name: dev-team
    apiGroup: rbac.authorization.k8s.io
roleRef:
  kind: ClusterRole
  name: developer-scoped
  apiGroup: rbac.authorization.k8s.io
```

**Why this works**: The ClusterRole defines the permission set once. Each RoleBinding
grants those permissions within a specific namespace. The `dev-team` group can work
in `development` and `staging` but not in `production` or any other namespace.

### Verification and Cleanup

```bash
# Verify new bindings work
kubectl auth can-i list deployments --namespace=development --as=dev-user@company.com
# yes

kubectl auth can-i list deployments --namespace=production --as=dev-user@company.com
# should be no (no RoleBinding in production)

# Only after verification: delete the old binding
kubectl delete clusterrolebinding admin-all  # or whatever the old binding is named
kubectl delete clusterrole developer-all      # or whatever the old role is named
```

---

## Part D: Remediation -- Remove Stale ServiceAccounts

```bash
#!/bin/bash
echo "=== Stale ServiceAccount Cleanup ==="
echo ""

# Get all non-default ServiceAccounts
kubectl get serviceaccounts --all-namespaces -o json | \
  jq -r '.items[] | select(.metadata.name != "default") |
  "\(.metadata.namespace) \(.metadata.name)"' | \
while read ns name; do
  # Check if any pod uses this ServiceAccount
  count=$(kubectl get pods -n "$ns" -o json 2>/dev/null | \
    jq "[.items[] | select(.spec.serviceAccountName == \"$name\")] | length")

  if [ "$count" -eq 0 ]; then
    echo "STALE: namespace=$ns serviceaccount=$name"
    echo "  Deleting..."
    kubectl delete serviceaccount "$name" -n "$ns"
  fi
done

echo ""
echo "=== Cleanup Complete ==="
```

**Why this works**: The script iterates all non-default ServiceAccounts, checks if
any pod references each one, and deletes those with zero references. The `default`
ServiceAccount is excluded because it is automatically created in every namespace
and used by pods that do not specify a ServiceAccount.

**Safety**: Always print what you are deleting before deleting it. In a production
environment, you might want to first output the list to a file, review it, then
run the deletions separately.

---

## Part E: Audit Script

```bash
#!/bin/bash
# rbac-audit.sh -- Kubernetes RBAC Audit Report
# Run with: bash rbac-audit.sh

echo "========================================"
echo "       RBAC AUDIT REPORT"
echo "Date: $(date)"
echo "Cluster: $(kubectl config current-context)"
echo "========================================"
echo ""

# --- Section 1: cluster-admin Bindings ---
echo "--- 1. Non-system cluster-admin Bindings ---"
count=0
kubectl get clusterrolebindings -o json | \
  jq -r '.items[] | select(.roleRef.name=="cluster-admin") |
  select(.metadata.name | startswith("system:") | not) |
  "\(.metadata.name) -> \(.subjects[]? | .kind + "/" + .name)"' | \
while read line; do
  echo "  FINDING: $line"
  count=$((count + 1))
done
if [ "$count" -eq 0 ]; then
  echo "  (none found)"
fi
echo ""

# --- Section 2: Wildcard Permissions ---
echo "--- 2. Roles/ClusterRoles with Wildcard Permissions ---"
kubectl get clusterroles -o json | \
  jq -r '.items[] |
  select(.rules[]? | (.resources[]? == "*" or .verbs[]? == "*")) |
  select(.metadata.name | startswith("system:") | not) |
  "  ClusterRole: \(.metadata.name) -- wildcards in rules"' 2>/dev/null

kubectl get roles --all-namespaces -o json | \
  jq -r '.items[] |
  select(.rules[]? | (.resources[]? == "*" or .verbs[]? == "*")) |
  "  Role: \(.metadata.namespace)/\(.metadata.name) -- wildcards in rules"' 2>/dev/null
echo ""

# --- Section 3: Stale ServiceAccounts ---
echo "--- 3. ServiceAccounts with No Referencing Pods ---"
kubectl get serviceaccounts --all-namespaces -o json | \
  jq -r '.items[] | select(.metadata.name != "default") |
  "\(.metadata.namespace) \(.metadata.name)"' | \
while read ns name; do
  count=$(kubectl get pods -n "$ns" -o json 2>/dev/null | \
    jq "[.items[] | select(.spec.serviceAccountName == \"$name\")] | length")
  if [ "$count" -eq 0 ]; then
    echo "  STALE: namespace=$ns serviceaccount=$name"
  fi
done
echo ""

# --- Section 4: Suspicious User/Group Names ---
echo "--- 4. Users/Groups in Bindings (review for suspicious entries) ---"
kubectl get rolebindings,clusterrolebindings --all-namespaces -o json | \
  jq -r '.items[].subjects[]? |
  select(.kind == "User" or .kind == "Group") |
  "\(.kind): \(.name)"' | sort -u | \
while read entry; do
  name=$(echo "$entry" | cut -d: -f2 | xargs)
  if echo "$name" | grep -qiE "test|temp|debug|admin@example|old-|deprecated"; then
    echo "  SUSPICIOUS: $entry"
  else
    echo "  OK: $entry"
  fi
done
echo ""

echo "========================================"
echo "       END OF REPORT"
echo "========================================"
```

**What this script produces**:
1. Lists all non-system `cluster-admin` bindings with their subjects.
2. Lists all non-system Roles and ClusterRoles with wildcard permissions.
3. Lists all ServiceAccounts with no pods referencing them.
4. Lists all users and groups, flagging suspicious names.
5. A human-readable report that can be shared with the security team.

**How to use it**: Run the script, save the output to a file, and review each finding.
The script does not make changes -- it only reports. Use the remediation steps from
Parts C and D to fix the issues found.

---

## Common Mistakes

1. **Deleting before verifying**: Never delete a RoleBinding or ServiceAccount
   without first confirming what it affects. Run `kubectl auth can-i --list` for
   the affected identity before and after any change.

2. **Confusing ClusterRole with ClusterRoleBinding**: A ClusterRole is just a
   permission definition. It is harmless until bound. Focus remediation on bindings,
   not on ClusterRoles themselves (unless the ClusterRole itself is too permissive).

3. **Not checking for CRDs**: Wildcard resource access (`resources: ["*"]`) includes
   CustomResourceDefinitions. If your cluster uses CRDs (for Istio, ArgoCD, etc.),
   wildcard access grants control over those resources too.

4. **Ignoring the `default` ServiceAccount**: Every namespace has a `default`
   ServiceAccount. If pods do not specify a ServiceAccount, they use `default`.
   If `default` has been granted broad permissions, every pod in the namespace
   inherits those permissions. Always check what the `default` SA can do.

5. **Audit once and forget**: RBAC configurations drift over time. Teams add
   bindings, create new ServiceAccounts, and sometimes bind to `cluster-admin`
   "temporarily." Schedule regular audits (monthly or quarterly) to catch drift.

6. **Not understanding the `system:` prefix**: Kubernetes creates many
   `system:*` ClusterRoles and ClusterRoleBindings for internal components.
   These are normal and should not be modified. Filter them out of your audit
   results to focus on human-created bindings.

## Relevant README Sections

- [The Naive Way](../README.md#naive-way--everything-in-default-everyone-is-cluster-admin)
- [RBAC Object Hierarchy](../README.md#rbac-object-hierarchy)
- [Predefined ClusterRoles for Common Personas](../README.md#2-predefined-clusterroles-for-common-personas)
- [Lab: Audit RBAC Permissions](../README.md#lab-audit-rbac-permissions)
