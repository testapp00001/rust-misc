# Solution 01: Namespace Isolation and the RBAC Model

## Part A: Namespace Boundaries

### Incident 1: Intern Deletes All Deployments

`kubectl delete deploy --all` is scoped to the current namespace context (or the
namespace specified by `--namespace`). If every team is in the `default` namespace,
this command deletes all deployments from all teams.

**With namespaces**: If Marketing deploys to the `marketing` namespace and Payments
deploys to the `payments` namespace, the intern's command only deletes Marketing's
deployments. The command does not reach across namespace boundaries. The blast radius
is reduced from "entire cluster" to "one team's staging environment."

**Recommended namespace structure**:

```
payments-dev/
payments-staging/
payments-production/
marketing-dev/
marketing-staging/
platform-dev/
platform-staging/
platform-production/
```

This is the namespace-per-team-per-environment pattern. It prevents cross-team
collisions and allows per-team, per-environment resource quotas.

### Incident 2: ConfigMap Name Collision

Kubernetes resource names must be unique within a namespace, but the same name can
exist in different namespaces. When both teams use `default`, a ConfigMap named
`app-config` created by Payments overwrites Marketing's `app-config`.

**With namespaces**: Payments creates `app-config` in `payments-dev`, Marketing
creates `app-config` in `marketing-dev`. Both coexist without collision. The names
are scoped to their respective namespaces.

---

## Part B: RBAC Scope Semantics

### 1. Role

**What it controls**: Permissions within a single namespace. A Role defines which
resources can be accessed and which verbs are allowed, but only within the namespace
where the Role is created.

**When to use**: When a user or ServiceAccount needs access to resources in exactly
one namespace. For example, a developer who works only on the `payments-dev` namespace.

**Example**: A Role in `payments-dev` that allows `get`, `list`, `watch`, `create`,
`update`, `patch` on Deployments and Services.

### 2. ClusterRole

**What it controls**: Permissions across the entire cluster, or reusable permission
definitions that can be bound to specific namespaces. A ClusterRole can grant access
to cluster-scoped resources (Nodes, Namespaces, PersistentVolumes) or serve as a
template for namespace-scoped bindings.

**When to use**:
- When a user needs access to cluster-scoped resources like Nodes or Namespaces.
- When you want to define a "developer" permission set once and reuse it across
  multiple namespaces via RoleBindings.

**Example**: A ClusterRole named `developer` that grants CRUD on Deployments, Services,
and ConfigMaps. This ClusterRole can be bound to namespace `payments-dev` via a
RoleBinding, granting those permissions only in that namespace.

### 3. RoleBinding

**What it controls**: Assigns a Role or ClusterRole to a user, group, or ServiceAccount
within a specific namespace. The permissions are scoped to the namespace where the
RoleBinding exists.

**When to use**: Whenever you want to grant a user access to a specific namespace.
This is the standard binding mechanism for namespace-scoped access.

**Example**: A RoleBinding in `payments-dev` that binds the `developer` Role to user
`alice@company.com`. Alice can now manage resources in `payments-dev` but nowhere else.

### 4. ClusterRoleBinding

**What it controls**: Assigns a ClusterRole to a user, group, or ServiceAccount
across the entire cluster. The permissions apply to all namespaces.

**When to use**: When a user needs cluster-wide access to specific resources. This
should be rare and carefully controlled.

**Example**: A ClusterRoleBinding that grants the `cluster-viewer` ClusterRole to
the `sre-team` group, allowing them to read pods and services in all namespaces.

**Key insight**: A RoleBinding can reference a ClusterRole. This grants the
ClusterRole's permissions but scopes them to the RoleBinding's namespace. This is
the recommended pattern for reusing permission definitions across namespaces without
granting cluster-wide access.

---

## Part C: Why Not Just Use ClusterRoleBinding for Everything?

### 1. Unlimited Blast Radius

With `cluster-admin`, a single mistake affects every namespace. The intern's
`kubectl delete deploy --all` would not just hit one namespace -- it would hit
every namespace in the cluster. The Platform engineer's pipe-to-delete script
would destroy all workloads across all teams. With namespace-scoped Roles, mistakes
are contained to a single namespace.

### 2. No Audit Granularity

When everyone is `cluster-admin`, you cannot tell who was supposed to have access
to what. If a Secrets object in the `payments-production` namespace is modified,
you cannot determine whether it was authorized because every user had permission
to modify every Secret everywhere. With namespace-scoped Roles and RoleBindings,
you can trace access to specific users and specific namespaces.

### 3. Violates Least Privilege

The principle of least privilege states that every user should have only the
minimum permissions needed to do their job. A Marketing intern does not need to
access the Payments API. A Payments developer does not need to modify RBAC objects.
`cluster-admin` grants everything to everyone, which means no user has exactly
the permissions they need -- they all have far more.

### 4. No Compliance or Separation of Duties

Regulatory frameworks (SOC 2, PCI-DSS, HIPAA) require separation of duties: the
person who deploys code should not be the same person who manages access controls.
With `cluster-admin`, every user can modify RBAC rules, making separation of duties
impossible to enforce.

### 5. No Resource Quota Enforcement

Even if you set ResourceQuotas per namespace, `cluster-admin` users can modify or
delete quotas. Without RBAC restrictions, quotas are suggestions, not enforcement.

---

## Part D: ServiceAccount vs. User

### Kubernetes User

- **Created by**: An external identity provider (OIDC, LDAP, certificates). Kubernetes
  does not have a User resource -- users exist outside the cluster.
- **Stored in**: The identity provider, not in Kubernetes. The API server validates
  user tokens but does not manage user records.
- **Used by**: Human engineers authenticating via `kubectl`, CI/CD systems using
  OIDC tokens, or anyone with a valid client certificate.
- **RBAC binding**: Referenced by name in RoleBinding/ClusterRoleBinding subjects
  with `kind: User`.

### ServiceAccount

- **Created by**: Kubernetes itself. ServiceAccounts are Kubernetes resources stored
  in etcd.
- **Stored in**: A specific namespace. ServiceAccounts are namespace-scoped.
- **Used by**: Pods and controllers inside the cluster. Each Pod automatically mounts
  a ServiceAccount token (unless `automountServiceAccountToken: false`).
- **RBAC binding**: Referenced in RoleBinding/ClusterRoleBinding subjects with
  `kind: ServiceAccount` and a `namespace` field.

### Why It Matters for RBAC

Users represent humans or external systems. ServiceAccounts represent workloads. This
distinction is critical for the "who changed the credentials?" incident:

- If the change was made by a User, you can trace it to a human via the identity
  provider's audit logs.
- If the change was made by a ServiceAccount, you can trace it to a specific Pod
  in a specific namespace. From there, you can find the Deployment that runs the Pod,
  the container image in the Deployment, and the codebase that built the image.

This is why ServiceAccounts should have minimal permissions: if a Pod is compromised,
the attacker gains only the ServiceAccount's permissions. If every Pod uses the
`default` ServiceAccount with `cluster-admin`, a compromised Pod means a compromised
cluster.

---

## Common Mistakes

1. **Confusing namespace isolation with network isolation**: Namespaces do not
   prevent network traffic between pods in different namespaces by default. You
   need NetworkPolicies for that. Namespaces only isolate API resource names and
   RBAC scope.

2. **Using ClusterRole when Role would suffice**: If a permission only needs to
   apply in one namespace, use a Role. ClusterRoles add complexity and risk.

3. **Forgetting that RoleBinding can reference ClusterRole**: This is a powerful
   pattern (define once, bind per-namespace) but it can be confusing. The
   ClusterRole defines the permissions; the RoleBinding scopes them.

4. **Granting `*` on verbs "just in case"**: This grants `delete`, `deletecollection`,
   and any future verbs. Always enumerate the specific verbs needed.

5. **Not understanding that RBAC is deny-by-default**: If no rule explicitly allows
   an action, it is denied. This means you do not need to write "deny" rules -- you
   only write "allow" rules for what is needed.

## Relevant README Sections

- [The Problem](../README.md#problem)
- [The Naive Way](../README.md#naive-way--everything-in-default-everyone-is-cluster-admin)
- [RBAC Object Hierarchy](../README.md#rbac-object-hierarchy)
- [Predefined ClusterRoles for Common Personas](../README.md#2-predefined-clusterroles-for-common-personas)
