# Exercise 05: RBAC Audit and Remediation

**Type:** Integration
**Time:** 45-60 minutes
**Difficulty:** Hard

## Objective

Audit an existing Kubernetes cluster's RBAC configuration to find overly permissive
bindings, stale ServiceAccounts, and privilege escalation risks. Then remediate the
issues by replacing broad permissions with least-privilege alternatives.

## Scenario

You have inherited a Kubernetes cluster from a team that "just made it work." The
cluster has been running for two years with multiple teams deploying workloads.
Management has asked you to audit and fix the RBAC configuration before a compliance
review. You suspect the cluster has several security issues.

## Tasks

### Part A: Discovery -- Map the Current State

Write the `kubectl` commands to answer each question. Record the commands and explain
what the output tells you.

1. List all ClusterRoleBindings that grant the `cluster-admin` ClusterRole. How
   many non-system bindings exist?

<details>
<summary>Hint</summary>

```bash
kubectl get clusterrolebindings -o json | \
  jq '.items[] | select(.roleRef.name=="cluster-admin") |
  {name: .metadata.name, subjects: .subjects}'
```

Filter out system bindings like `cluster-admin`, `system:*,` etc. Any user or
ServiceAccount directly bound to `cluster-admin` is a finding.

</details>

2. List all ServiceAccounts across all namespaces. Which ones have no pods using
   them (stale accounts)?

<details>
<summary>Hint</summary>

```bash
# List all ServiceAccounts
kubectl get serviceaccounts --all-namespaces

# For each non-default SA, check if any pods reference it
kubectl get pods --all-namespaces -o json | \
  jq '.items[] | .spec.serviceAccountName' | sort -u
```

Compare the two lists. ServiceAccounts that exist but have no pods referencing them
may be stale.

</details>

3. Find all Roles and ClusterRoles that grant `*` (wildcard) on resources or verbs.
   List each one and explain the risk.

<details>
<summary>Hint</summary>

```bash
kubectl get clusterroles -o json | \
  jq '.items[] | select(.rules[]? | .resources[]? == "*") | .metadata.name'

kubectl get roles --all-namespaces -o json | \
  jq '.items[] | select(.rules[]? | .verbs[]? == "*") |
  {namespace: .metadata.namespace, name: .metadata.name}'
```

A wildcard on resources means the subject can access any current or future resource
type. A wildcard on verbs means the subject can perform any action, including delete
and deletecollection.

</details>

4. Identify all users and groups referenced in RoleBindings and ClusterRoleBindings.
   Are there any bindings to users or groups that look like test accounts, former
   employees, or typos?

<details>
<summary>Hint</summary>

```bash
kubectl get rolebindings,clusterrolebindings --all-namespaces -o json | \
  jq '[.items[].subjects[]? | select(.kind == "User" or .kind == "Group") |
  {binding: .kind + "/" + .name, subject_kind: .kind, subject_name: .name}] |
  unique_by(.subject_name)'
```

Look for patterns like `test@`, `temp@`, `admin@example.com`, or groups named
`test-group`.

</details>

### Part B: Risk Assessment

For each finding in Part A, classify the risk as Critical, High, Medium, or Low.
Explain your reasoning for each classification.

<details>
<summary>Hint</summary>

Use this framework:

- **Critical**: Direct path to cluster compromise (e.g., ServiceAccount with
  cluster-admin, wildcard permissions on secrets)
- **High**: Can escalate privileges or access sensitive data across namespaces
  (e.g., cluster-wide read on secrets, wildcard verbs in a namespace)
- **Medium**: Overly broad but limited to one namespace (e.g., all verbs on all
  resources in a single namespace)
- **Low**: Stale accounts with no active permissions, or read-only wildcards on
  non-sensitive resources

</details>

### Part C: Remediation -- Replace Wildcard Roles

You found a ClusterRole named `developer-all` that grants `*` on all resources with
all verbs. It is bound to the `dev-team` group via a ClusterRoleBinding. Replace it
with a properly scoped configuration:

1. Define a ClusterRole named `developer-scoped` that grants only the permissions a
   developer actually needs (Deployments, Services, ConfigMaps, Pods -- CRUD).
2. Remove the ClusterRoleBinding.
3. Create namespace-scoped RoleBindings for the `dev-team` group in only the
   namespaces they should access.

Write all the YAML manifests and the commands to apply them. Explain the order of
operations (why order matters when removing and replacing bindings).

<details>
<summary>Hint 1</summary>

Order matters because if you delete the ClusterRoleBinding first, developers lose
all access before the new bindings are in place. If you create the new bindings
first, there is a brief window where developers have both the old wildcard access
and the new scoped access. The safest approach:

1. Create the new scoped ClusterRole.
2. Create the namespace-scoped RoleBindings.
3. Verify the new bindings work.
4. Delete the old ClusterRoleBinding.
5. Delete the old ClusterRole.

</details>

<details>
<summary>Hint 2</summary>

```yaml
# New scoped ClusterRole
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: developer-scoped
rules:
  - apiGroups: ["apps"]
    resources: ["deployments", "replicasets"]
    verbs: ["get", "list", "watch", "create", "update", "patch"]
  - apiGroups: [""]
    resources: ["services", "configmaps", "pods", "pods/log"]
    verbs: ["get", "list", "watch", "create", "update", "patch"]
```

Then bind it per-namespace using RoleBindings. A RoleBinding can reference a
ClusterRole -- this grants the ClusterRole's permissions but scopes them to the
RoleBinding's namespace.

</details>

### Part D: Remediation -- Remove Stale ServiceAccounts

Write the commands to:
1. List all ServiceAccounts that have no pods using them.
2. Delete them (with confirmation that no pods reference them).

<details>
<summary>Hint</summary>

```bash
# Get all non-default ServiceAccounts
kubectl get serviceaccounts --all-namespaces -o json | \
  jq -r '.items[] | select(.metadata.name != "default") |
  "\(.metadata.namespace)/\(.metadata.name)"' | \
  while read sa; do
    ns=$(echo $sa | cut -d/ -f1)
    name=$(echo $sa | cut -d/ -f2)
    count=$(kubectl get pods -n $ns -o json | \
      jq "[.items[] | select(.spec.serviceAccountName == \"$name\")] | length")
    if [ "$count" -eq 0 ]; then
      echo "STALE: $ns/$name"
      # kubectl delete serviceaccount $name -n $ns  # uncomment to actually delete
    fi
  done
```

Always print before deleting. Never delete without confirming.

</details>

### Part E: Write an Audit Script

Write a bash script that produces a single report covering:
1. ClusterRoleBindings to `cluster-admin` (excluding system bindings)
2. Roles and ClusterRoles with wildcard permissions
3. ServiceAccounts with no pods
4. Users or groups that look like test accounts

The script should output a human-readable report, not raw JSON.

<details>
<summary>Hint</summary>

Structure the script as separate sections with clear headers. Use `kubectl` with
`-o json` piped to `jq` for filtering. Print section headers, findings, and a
summary count for each category.

```bash
#!/bin/bash
echo "=== RBAC Audit Report ==="
echo "Date: $(date)"
echo ""

echo "--- 1. cluster-admin Bindings (non-system) ---"
kubectl get clusterrolebindings -o json | \
  jq -r '.items[] | select(.roleRef.name=="cluster-admin") |
  select(.metadata.name | startswith("system:") | not) |
  .metadata.name'
echo ""
# ... continue for each category
```

</details>

## Success Criteria

- [ ] You can list all cluster-admin bindings and identify non-system ones
- [ ] You can find stale ServiceAccounts with no referencing pods
- [ ] You can identify Roles and ClusterRoles with wildcard permissions
- [ ] You can classify RBAC findings by risk level with justification
- [ ] You can replace a wildcard ClusterRole with a scoped ClusterRole + namespace RoleBindings
- [ ] You understand the correct order of operations when replacing RBAC bindings
- [ ] You can write an audit script that produces a readable report
- [ ] You can explain why "works" is not the same as "secure"

## What You Should Understand After This Exercise

RBAC audit is not optional -- it is a continuous process. Clusters accumulate
permissions over time as teams come and go, and the default Kubernetes setup includes
many system bindings that should not be extended to human users. The audit process
involves discovery (what exists), risk assessment (what is dangerous), and remediation
(replace broad permissions with scoped ones). The hardest part is not writing the YAML
-- it is convincing the team that "it works" does not mean "it is correct."
