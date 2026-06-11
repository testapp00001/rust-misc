# Exercise 03: Implement RBAC with Least Privilege

**Type:** Independent
**Estimated time:** 45 minutes

## Objective

Design and implement a Role-Based Access Control (RBAC) policy that gives
a development team exactly the permissions they need and nothing more. This
exercise has no step-by-step guidance -- apply what you have learned.

## Scenario

Your company has a team called **backend-devs** who work in the `backend`
namespace. They need:

1. Permission to view, create, update, and delete Deployments, ReplicaSets,
   and Pods.
2. Permission to view ConfigMaps and Secrets (they need to check
   configuration but must not modify secrets).
3. Permission to read Pod logs and exec into pods for debugging.
4. **No** access to any other namespaces.
5. **No** permission to create or modify Roles, RoleBindings, or
   ServiceAccounts (they must not be able to escalate privileges).

A second team called **sre-team** needs:

1. Cluster-wide read-only access to all resources in all namespaces.
2. Permission to restart deployments (delete pods) in the `backend` namespace
   only.

## Instructions

### Part A -- Create the Namespace and ServiceAccounts

1. Create the `backend` namespace.
2. Create a ServiceAccount `backend-dev-sa` in the `backend` namespace.
3. Create a ServiceAccount `sre-sa` in the `sre` namespace (create it first).

### Part B -- Backend Devs RBAC

Create the following resources:

1. A `Role` in the `backend` namespace that grants the permissions listed
   above for the backend-devs team.
2. A `RoleBinding` that binds the Role to the `backend-dev-sa`
   ServiceAccount.

### Part C -- SRE Team RBAC

Create the following resources:

1. A `ClusterRole` for cluster-wide read-only access. Use the built-in
   `view` ClusterRole as inspiration, but create your own so you understand
   the fields.
2. A `ClusterRoleBinding` that binds this ClusterRole to the `sre-sa`
   ServiceAccount.
3. A `Role` in the `backend` namespace that grants delete permission on pods.
4. A `RoleBinding` in the `backend` namespace that binds the pod-delete Role
   to the `sre-sa` ServiceAccount.

### Part D -- Verification

Verify your RBAC configuration using `kubectl auth can-i`:

```bash
# Backend devs -- should be YES
kubectl auth can-i create deployments --as=system:serviceaccount:backend:backend-dev-sa -n backend
kubectl auth can-i get secrets --as=system:serviceaccount:backend:backend-dev-sa -n backend
kubectl auth can-i get pods/log --as=system:serviceaccount:backend:backend-dev-sa -n backend

# Backend devs -- should be NO
kubectl auth can-i delete namespaces --as=system:serviceaccount:backend:backend-dev-sa
kubectl auth can-i create roles --as=system:serviceaccount:backend:backend-dev-sa -n backend
kubectl auth can-i get pods --as=system:serviceaccount:backend:backend-dev-sa -n kube-system

# SRE team -- should be YES
kubectl auth can-i get pods --as=system:serviceaccount:sre:sre-sa -n backend
kubectl auth can-i get pods --as=system:serviceaccount:sre:sre-sa -n kube-system
kubectl auth can-i delete pods --as=system:serviceaccount:sre:sre-sa -n backend

# SRE team -- should be NO
kubectl auth can-i create deployments --as=system:serviceaccount:sre:sre-sa -n backend
kubectl auth can-i delete nodes --as=system:serviceaccount:sre:sre-sa
```

## Success Criteria

- [ ] All `can-i` checks return the expected YES or NO answers.
- [ ] The backend-devs Role uses `apiGroups` correctly (apps/v1 for
      Deployments and ReplicaSets, core/v1 for Pods, etc.).
- [ ] The SRE ClusterRole does not use wildcard `*` for verbs -- it
      explicitly lists `get`, `list`, `watch`.
- [ ] No Role or RoleBinding grants `escalate`, `bind`, or `impersonate`
      verbs.
- [ ] Secrets access for backend-devs is limited to `get`, `list`, `watch`
      (read-only).

## Hints

<details>
<summary>Hint 1 -- apiGroups for Deployments</summary>
Deployments and ReplicaSets belong to the `apps` API group, not the core
group. Use `apiGroups: ["apps"]` with `resources: ["deployments",
"replicasets"]`.
</details>

<details>
<summary>Hint 2 -- Pod logs and exec</summary>
Pod logs are a subresource: `resources: ["pods/log"]`. Exec is also a
subresource but requires `create` on `pods/exec` (or `create` on `pods`
with the `pods/exec` subresource). In practice, granting `create` on
`pods/exec` is sufficient.
</details>

<details>
<summary>Hint 3 -- Built-in view ClusterRole</summary>
Run `kubectl get clusterrole view -o yaml` to see what the built-in
read-only role looks like. Use it as a reference but write your own from
scratch.
</details>

<details>
<summary>Hint 4 -- Aggregating roles</summary>
The SRE team needs cluster-wide read plus namespace-scoped delete on pods.
You can bind the same ServiceAccount to multiple Roles/ClusterRoles using
separate bindings. Kubernetes unions all permissions from all bindings.
</details>
