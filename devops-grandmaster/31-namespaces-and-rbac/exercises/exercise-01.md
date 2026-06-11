# Exercise 01: Namespace Isolation and the RBAC Model

**Type:** Conceptual
**Time:** 15 minutes
**Difficulty:** Easy

## Objective

Understand *why* Kubernetes needs namespaces and RBAC by analyzing concrete failures
of the "everything in default, everyone is cluster-admin" approach. This exercise
trains you to think about isolation boundaries and the principle of least privilege.

## Scenario

Your company has a single Kubernetes cluster shared by three teams: Payments, Marketing,
and Platform. Every team deploys to the `default` namespace. Every engineer has a
`cluster-admin` ClusterRoleBinding. Last week, these incidents happened:

1. A Marketing intern ran `kubectl delete deploy --all` and took down the Payments API.
2. A Payments developer created a ConfigMap named `app-config` that overwrote Marketing's
   ConfigMap with the same name.
3. A Platform engineer ran a script that listed all pods across all namespaces and
   accidentally piped the output to `kubectl delete -f -`.
4. An auditor asked "who changed the production database credentials?" and nobody could
   answer because all changes were mixed together in one namespace.

## Tasks

### Part A: Namespace Boundaries

Explain how namespaces would have prevented incidents 1 and 2 above. Be specific:
what namespace structure would you create, and how does it prevent each incident?

<details>
<summary>Hint</summary>

Think about:
- What does `kubectl delete deploy --all` actually scope to?
- Can two ConfigMaps with the same name coexist if they are in different namespaces?
- What is the smallest unit of resource isolation in Kubernetes?

</details>

### Part B: RBAC Scope Semantics

For each RBAC object below, explain what it controls and when you would use it.
Give a concrete example for each.

1. `Role`
2. `ClusterRole`
3. `RoleBinding`
4. `ClusterRoleBinding`

<details>
<summary>Hint</summary>

Focus on two dimensions:
- **Scope**: Is it namespace-scoped or cluster-scoped?
- **Binding**: Does it attach to a user/group/ServiceAccount, and where?

A Role is useless without a RoleBinding. A ClusterRoleBinding grants access across
all namespaces. What happens if you use a RoleBinding to reference a ClusterRole?

</details>

### Part C: Why Not Just Use ClusterRoleBinding for Everything?

Explain at least three problems with granting every user a ClusterRoleBinding to
the `cluster-admin` ClusterRole. Reference the incidents above where possible.

<details>
<summary>Hint</summary>

Think about:
- Blast radius of mistakes
- Audit and accountability
- The principle of least privilege
- What happens when a user only needs to read pods in one namespace

</details>

### Part D: ServiceAccount vs. User

Explain the difference between a Kubernetes `User` and a `ServiceAccount`. When
would you use each? Why does it matter for the RBAC model?

<details>
<summary>Hint</summary>

Consider:
- Who creates Users? Who creates ServiceAccounts?
- Where are ServiceAccounts stored?
- What identity does a Pod use by default to talk to the API server?
- How does this relate to the incident about "who changed the credentials?"

</details>

## Success Criteria

- [ ] You can explain how namespaces prevent resource name collisions and accidental cross-team deletions
- [ ] You can describe the difference between Role and ClusterRole in terms of scope
- [ ] You can explain why RoleBinding exists separately from Role
- [ ] You can articulate at least three problems with universal cluster-admin access
- [ ] You understand the difference between Users and ServiceAccounts in the RBAC model

## What You Should Understand After This Exercise

Namespaces are not just labels -- they are hard isolation boundaries for resource
names and RBAC scope. RBAC is a two-part system: permissions (Role/ClusterRole) are
separate from assignments (RoleBinding/ClusterRoleBinding). This separation lets you
define a "developer" permission once and bind it to different users in different
namespaces. ServiceAccounts provide workload identity, enabling you to audit which
Pod performed which API call.
