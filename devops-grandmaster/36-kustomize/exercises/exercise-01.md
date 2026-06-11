# Exercise 01: Kustomize vs Helm -- When to Use Each

**Type:** Conceptual
**Time:** 20 minutes
**Objective:** Develop a decision framework for choosing between Kustomize and
Helm for Kubernetes configuration management.

---

## Background

Your team is adopting GitOps. You need to standardize how Kubernetes manifests
are managed across 15 microservices. Some services are simple (one Deployment,
one Service), others are complex (multiple Deployments, CronJobs, RBAC,
NetworkPolicies, external dependencies like Redis or PostgreSQL).

Two engineers have competing proposals:

- **Engineer A** argues for Kustomize: "No templating, pure YAML, built into
  kubectl, easy to read and review in PRs."
- **Engineer B** argues for Helm: "Rich ecosystem, dependency management,
  conditional logic, community charts for databases and operators."

You are the platform lead. You must make a recommendation.

---

## Tasks

### Task 1: Feature Comparison Table

Create a comparison table covering at least 8 dimensions. Fill in each cell
with a concrete, specific answer -- not vague statements like "better."

| Dimension | Kustomize | Helm |
|-----------|-----------|------|
| Templating approach | ? | ? |
| Dependency management | ? | ? |
| Built into kubectl | ? | ? |
| Learning curve | ? | ? |
| Secret management | ? | ? |
| Reusability model | ? | ? |
| Output determinism | ? | ? |
| Community ecosystem | ? | ? |

### Task 2: Scenario Classification

For each scenario below, state whether you would choose **Kustomize**, **Helm**,
or **both** and explain your reasoning in 1-2 sentences.

**Scenario A:** You have 12 microservices, each with a Deployment and Service.
They share common labels, resource limits, and a namespace. You need dev/staging/prod
variants that differ only in replica counts, image tags, and resource limits.

**Scenario B:** You need to deploy a PostgreSQL HA cluster with configurable
persistence, backup CronJobs, monitoring sidecars, and RBAC. The team has
never operated PostgreSQL on Kubernetes before.

**Scenario C:** You have a monolithic application that depends on Redis,
RabbitMQ, and Elasticsearch. You want to deploy the full stack in one command
for development, but in production Redis and RabbitMQ are managed externally.

**Scenario D:** You have 40 services across 3 clusters (us-east, us-west,
eu-west). Each cluster has slightly different ConfigMaps (region-specific
endpoints) and different resource quotas. You want a single source of truth
in Git with per-cluster customization.

**Scenario E:** An open-source project provides a Helm chart for their
operator. You need to deploy it but customize resource limits, add a
NetworkPolicy, and change the Service type from ClusterIP to NodePort for
your dev environment.

### Task 3: Anti-Pattern Identification

Read each statement and explain whether it is a misconception or a valid
concern. Provide a concrete counterexample or supporting example.

1. "Kustomize cannot handle secrets safely."
2. "Helm charts always produce non-deterministic output."
3. "Kustomize replaces the need for any scripting in CI/CD."
4. "You cannot use Kustomize with Helm charts."
5. "Kustomize overlays create copy-paste drift over time."

---

## Success Criteria

- [ ] Your comparison table has at least 8 dimensions with specific answers.
- [ ] Each scenario recommendation cites a concrete technical reason.
- [ ] Each anti-pattern answer includes a working example or a cited limitation.
- [ ] You can articulate the single strongest argument for each tool.

---

## Hints

<details>
<summary>Hint 1: Templating vs. Overlays</summary>

Kustomize uses a **patching** model: you define a complete base resource and
then apply strategic merge patches or JSON patches. There is no variable
substitution in the traditional sense (though `vars` and replacements exist).
Helm uses Go templates with `{{ .Values.xxx }}` expressions, which gives you
arbitrary logic at the cost of readability.

Key question: do your manifests need **conditional resources** (e.g., "only
create this CronJob if backups are enabled") or just **value changes** (e.g.,
"change replicas from 1 to 3")?

</details>

<details>
<summary>Hint 2: Dependency Management</summary>

Helm has `Chart.yaml` with a `dependencies` block. You can pull in community
charts (e.g., `bitnami/postgresql`) and configure them through `values.yaml`.
Kustomize has no native concept of pulling external dependencies -- you must
reference raw YAML files or use `resources` with URLs to raw manifests.

Ask yourself: do you need to compose multiple independently-versioned components,
or are you customizing a single set of resources?

</details>

<details>
<summary>Hint 3: The "Both" Answer</summary>

Kustomize and Helm are not mutually exclusive. You can use `helm template` to
render a chart to plain YAML, then use Kustomize to patch that output. You can
also use Kustomize's `helmChart` field (v4.1+) to inline Helm chart rendering
inside a kustomization.yaml. The "both" answer is valid when different parts of
your system have different complexity profiles.

</details>

<details>
<summary>Hint 4: Secret Management</summary>

Neither Kustomize nor Helm should store secrets in plain text in Git.
Kustomize has `secretGenerator` which can read from env files or literals and
base64-encode them. Helm has `helm-secrets` plugin with SOPS integration.
Both require an external secrets management layer (Sealed Secrets, External
Secrets Operator, Vault) for production.

</details>
