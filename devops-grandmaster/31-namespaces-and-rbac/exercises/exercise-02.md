# Exercise 02: Team Namespaces with RBAC

**Type:** Guided
**Time:** 30 minutes
**Difficulty:** Easy-Medium

## Objective

Create namespaces for two teams, define RBAC Roles with appropriate permissions,
bind those Roles to users, and verify that isolation works using `kubectl auth can-i`.

## Scenario

You are the platform engineer at a company with two teams:

- **Team Alpha**: Needs full CRUD access to Deployments, Services, ConfigMaps, and Pods
  in their own namespace. They should be able to exec into pods for debugging but should
  only have read access to Secrets.
- **Team Beta**: Needs the same permissions as Alpha, but scoped to their own namespace.

No team should be able to see or modify another team's resources.

## Tasks

### Part A: Create the Namespaces

Create two namespaces: `alpha` and `beta`. Apply labels that identify the owning team.

Write the YAML manifests and the `kubectl` commands to apply them.

<details>
<summary>Hint 1</summary>

Use `apiVersion: v1` and `kind: Namespace`. Add `metadata.labels` with a `team` key.

</details>

<details>
<summary>Hint 2</summary>

```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: alpha
  labels:
    team: alpha
```

Apply with `kubectl apply -f namespace-alpha.yaml`.

</details>

### Part B: Create the Developer Role

Create a Role named `developer` in each namespace with these permissions:

| Resource | Verbs |
|----------|-------|
| deployments, services, configmaps, pods, pods/log | get, list, watch, create, update, patch |
| pods/exec | create |
| secrets | get, list |

Write the YAML for the Role in namespace `alpha`.

<details>
<summary>Hint 1</summary>

A Role is namespace-scoped. You must set `metadata.namespace: alpha`. The `rules`
field is a list of objects, each with `apiGroups`, `resources`, and `verbs`.

</details>

<details>
<summary>Hint 2</summary>

Deployments and ReplicaSets live in the `apps` API group. Pods, Services, ConfigMaps,
Secrets, and pods/exec live in the core API group (`""`). pods/log is a subresource
of pods.

</details>

### Part C: Bind the Role to Users

Create RoleBindings that grant:
- User `alice@company.com` the `developer` Role in namespace `alpha`
- User `bob@company.com` the `developer` Role in namespace `beta`

Write the YAML manifests for both RoleBindings.

<details>
<summary>Hint</summary>

A RoleBinding references a Role via `roleRef` and lists subjects (users, groups, or
ServiceAccounts). The `roleRef.kind` must be `Role` (not `ClusterRole`) when binding
a namespace-scoped Role.

```yaml
subjects:
  - kind: User
    name: alice@company.com
    apiGroup: rbac.authorization.k8s.io
roleRef:
  kind: Role
  name: developer
  apiGroup: rbac.authorization.k8s.io
```

</details>

### Part D: Verify Isolation

Use `kubectl auth can-i` to verify the following. Run each command and record the
expected result (yes/no):

1. Can Alice create deployments in `alpha`?
2. Can Alice create deployments in `beta`?
3. Can Bob list pods in `alpha`?
4. Can Bob list pods in `beta`?
5. Can Alice delete namespaces?
6. Can Alice delete secrets in `alpha`?

<details>
<summary>Hint</summary>

The command format is:

```bash
kubectl auth can-i <verb> <resource> --namespace=<ns> --as=<user>
```

For cluster-level checks, omit `--namespace`. The `--as` flag simulates a user
identity without needing actual OIDC credentials.

</details>

### Part E: Create a ServiceAccount

Create a ServiceAccount named `deploy-bot` in namespace `alpha`. Bind it to the
`developer` Role. Write the ServiceAccount YAML, the RoleBinding YAML, and the
command to verify the ServiceAccount can list pods.

<details>
<summary>Hint</summary>

A ServiceAccount is a namespace-scoped resource. In a RoleBinding, the subject
`kind` is `ServiceAccount` (not `User`). You must specify `namespace` in the
subject if the ServiceAccount is in the same namespace as the RoleBinding.

Verify with:

```bash
kubectl auth can-i list pods --namespace=alpha --as=system:serviceaccount:alpha:deploy-bot
```

</details>

## Success Criteria

- [ ] Namespaces `alpha` and `beta` exist with appropriate labels
- [ ] A `developer` Role exists in each namespace with the correct permissions
- [ ] Alice can manage resources in `alpha` but not in `beta`
- [ ] Bob can manage resources in `beta` but not in `alpha`
- [ ] Neither Alice nor Bob can perform cluster-level operations like deleting namespaces
- [ ] Alice cannot delete secrets (only read them) in `alpha`
- [ ] The `deploy-bot` ServiceAccount can list pods in `alpha`

## What You Should Understand After This Exercise

RBAC is a two-layer system: Roles define *what* can be done, and RoleBindings define
*who* can do it and *where*. By creating separate Roles per namespace and binding
them to different users, you achieve team isolation. ServiceAccounts provide identity
for automated workloads, and they follow the same Role/RoleBinding model as human users.
