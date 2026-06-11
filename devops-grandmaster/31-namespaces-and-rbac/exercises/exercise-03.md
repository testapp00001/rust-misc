# Exercise 03: Least-Privilege CI/CD Pipeline Access

**Type:** Independent
**Time:** 30-45 minutes
**Difficulty:** Medium

## Objective

Design and implement a least-privilege RBAC configuration for a CI/CD pipeline that
needs to deploy applications to a Kubernetes namespace. You must grant the pipeline
enough access to do its job but no more, and you must be able to justify every
permission.

## Scenario

Your team runs a CI/CD pipeline (GitHub Actions, GitLab CI, or similar) that deploys
a web application to the `webapp-staging` namespace. The pipeline needs to:

1. Deploy new container images by updating Deployments.
2. Create and update ConfigMaps and Secrets for application configuration.
3. Check pod status and read pod logs to verify deployments succeeded.
4. Run database migrations by creating Jobs.

The pipeline must NOT be able to:

- Delete Deployments (rollback is a manual, approval-gated process).
- Read Secrets from other namespaces.
- Create or modify RBAC objects (no privilege escalation).
- Access the cluster outside the `webapp-staging` namespace.
- Exec into pods or port-forward.

## Tasks

### Part A: Design the ServiceAccount

Create a ServiceAccount named `cicd-deployer` in namespace `webapp-staging`. Write
the YAML manifest.

<details>
<summary>Hint</summary>

```yaml
apiVersion: v1
kind: ServiceAccount
metadata:
  name: cicd-deployer
  namespace: webapp-staging
```

This ServiceAccount will be referenced by the CI/CD pipeline using a token or
cloud-provider workload identity.

</details>

### Part B: Define the Role

Create a Role named `cicd-deployer-role` in namespace `webapp-staging`. Map each
pipeline requirement to specific RBAC rules. Your YAML should include:

- The exact `apiGroups`, `resources`, and `verbs` for each requirement.
- Comments explaining why each rule exists.

<details>
<summary>Hint 1</summary>

Break down each requirement:

1. "Update Deployments" means `get`, `list`, `watch`, `update`, `patch` on
   `deployments` in the `apps` API group. But NOT `create` or `delete`.
2. "Create and update ConfigMaps and Secrets" means `create`, `update`, `patch`,
   `get`, `list` on `configmaps` and `secrets` in the core API group.
3. "Check pod status and read logs" means `get`, `list`, `watch` on `pods` and
   `get` on `pods/log` in the core API group.
4. "Create Jobs" means `create`, `get`, `list`, `watch` on `jobs` in the `batch`
   API group.

</details>

<details>
<summary>Hint 2</summary>

The `delete` verb is intentionally excluded from deployments. The `pods/exec` and
`pods/portforward` subresources are not included. RBAC for objects not listed is
implicitly denied.

</details>

### Part C: Create the RoleBinding

Write the RoleBinding that connects the ServiceAccount to the Role.

<details>
<summary>Hint</summary>

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

</details>

### Part D: Verify the Permissions

Write `kubectl auth can-i` commands to verify each of these expected results:

| Action | Expected |
|--------|----------|
| Update a deployment in `webapp-staging` | yes |
| Delete a deployment in `webapp-staging` | no |
| Create a Secret in `webapp-staging` | yes |
| List Secrets in `default` | no |
| Create a Job in `webapp-staging` | yes |
| Exec into a pod in `webapp-staging` | no |
| Create a Role in `webapp-staging` | no |
| List namespaces | no |

<details>
<summary>Hint</summary>

Use the ServiceAccount identity format:

```bash
kubectl auth can-i update deployments \
  --namespace=webapp-staging \
  --as=system:serviceaccount:webapp-staging:cicd-deployer
```

</details>

### Part E: Explain the Denied Permissions

For each "no" result in the table above, explain which part of the RBAC configuration
causes the denial. Be specific: is it because the verb is missing, the resource is
missing, the namespace is wrong, or the ServiceAccount is not bound?

<details>
<summary>Hint</summary>

RBAC is deny-by-default. A request is allowed only if there is an explicit rule that
permits it. Think about which of these conditions fails for each denied action:
- The verb is not in any rule for that resource
- The resource is not listed in any rule
- The Role is in a different namespace and no ClusterRole/ClusterRoleBinding covers it
- The ServiceAccount is not a subject in any binding that references a Role with that permission

</details>

## Success Criteria

- [ ] The `cicd-deployer` ServiceAccount exists in `webapp-staging`
- [ ] The Role grants exactly the permissions needed -- no more, no less
- [ ] The pipeline can update Deployments but cannot delete them
- [ ] The pipeline can manage ConfigMaps and Secrets in `webapp-staging` only
- [ ] The pipeline can read pod logs but cannot exec into pods
- [ ] The pipeline can create Jobs but cannot modify RBAC objects
- [ ] All eight verification commands produce the expected results
- [ ] You can explain why each denied action is denied

## What You Should Understand After This Exercise

Least-privilege RBAC requires mapping each pipeline action to specific verbs on specific
resources. You do not grant broad permissions and hope for the best -- you enumerate
what the pipeline needs and explicitly deny everything else. ServiceAccounts are the
correct identity for automated systems, and their permissions should be scoped to a
single namespace using Roles (not ClusterRoles).
