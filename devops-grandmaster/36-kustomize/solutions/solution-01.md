# Solution 01: Kustomize vs Helm -- When to Use Each

---

## Task 1: Feature Comparison Table

| Dimension | Kustomize | Helm |
|-----------|-----------|------|
| **Templating approach** | Patching via strategic merge and JSON patches. No template language. | Go templates with `{{ .Values.xxx }}`. Full programming logic in templates. |
| **Dependency management** | None. Resources must be referenced as local files or raw URLs. | `Chart.yaml` dependencies block. Pulls charts from OCI/Helm repos. |
| **Built into kubectl** | Yes, since v1.14 via `kubectl apply -k`. | No. Requires `helm` CLI or a GitOps controller. |
| **Learning curve** | Low for basic use. YAML-only. Grows with patch complexity. | Moderate. Must learn Go template syntax, values hierarchy, chart structure. |
| **Secret management** | `secretGenerator` reads from literals, env files, or files. No encryption. | `helm-secrets` plugin with SOPS. Secrets encrypted at rest in Git. |
| **Reusability model** | Bases and components. Compose via `resources` and `components` fields. | Subcharts and library charts. Compose via `dependencies` and `_helpers.tpl`. |
| **Output determinism** | Fully deterministic. Same input always produces same output. | Deterministic with `helm template`. With `helm install`, hooks may inject runtime values. |
| **Community ecosystem** | Limited. No central registry. You write your own manifests. | Massive. Artifact Hub has 10,000+ charts. Bitnami, community operators. |
| **Conditional resources** | Not natively supported. Requires overlays or components to add/remove resources. | Native `{{- if .Values.xxx }}` conditionals. |
| **Lifecycle management** | Stateless. Applies manifests; no concept of releases or rollbacks. | Stateful. Tracks releases, supports rollback, history, hooks. |

---

## Task 2: Scenario Classification

### Scenario A: 12 simple microservices with env variants

**Recommendation: Kustomize.**

These services differ only in values (replicas, image tags, resource limits).
Kustomize's `images`, `replicas`, and `patches` transformers handle this
cleanly. There is no conditional logic or dependency management needed. The
overlay model is purpose-built for this exact use case.

### Scenario B: PostgreSQL HA cluster

**Recommendation: Helm.**

The team has never operated PostgreSQL on Kubernetes. Bitnami's PostgreSQL Helm
chart provides battle-tested configurations for persistence, backup CronJobs,
monitoring, and RBAC. Building this from scratch with Kustomize would take
weeks and likely miss edge cases that the chart community has already solved.

### Scenario C: Full-stack with external service substitution

**Recommendation: Both.**

Use Helm for the application chart since it needs conditional dependency
management (include Redis/RabbitMQ only in dev). Use Kustomize for the
Kubernetes-native resources (Deployment, Service) if the team prefers plain
YAML. Alternatively, use Kustomize's `helmChart` field to render the Helm
chart inline and then apply Kustomize patches.

### Scenario D: 40 services across 3 clusters

**Recommendation: Kustomize.**

This is the canonical Kustomize use case. A base directory with all 40
services, and per-cluster overlays that patch ConfigMaps and resource quotas.
The `replacements` transformer can inject cluster-specific endpoints. No
dependency management or conditional resources needed. The output is
deterministic and easy to review in PRs.

### Scenario E: Open-source Helm chart with customizations

**Recommendation: Both (Helm with Kustomize patches).**

Install the chart with Helm, but use Kustomize to patch the rendered output.
Specifically: `helm template` the chart to get plain YAML, then apply
Kustomize patches for the NetworkPolicy and Service type change. Or use
Kustomize's `helmChart` field:

```yaml
helmChart:
  name: operator-chart
  repo: https://charts.example.com
  version: 1.0.0
  valuesInline:
    resources:
      limits:
        cpu: "500m"
        memory: "512Mi"
patches:
  - path: network-policy.yaml
```

---

## Task 3: Anti-Pattern Identification

### 1. "Kustomize cannot handle secrets safely."

**Partially valid.** `secretGenerator` base64-encodes values and puts them in
YAML, which is not encryption. Anyone with repo access sees the base64 values.
However, this is also true for Helm's default behavior. The real solution for
both tools is an external secrets manager (Sealed Secrets, External Secrets
Operator, Vault). Kustomize integrates with Sealed Secrets via
`kustomize build | kubeseal`.

### 2. "Helm charts always produce non-deterministic output."

**Misconception.** `helm template` produces deterministic output from the same
inputs. The non-determinism comes from hooks that generate values at runtime
(e.g., `randAlphaNum`), not from the templating itself. If you avoid runtime
generators in templates, Helm output is fully deterministic. In a GitOps
workflow, you run `helm template` in CI and commit the rendered YAML.

### 3. "Kustomize replaces the need for any scripting in CI/CD."

**Misconception.** Kustomize handles manifest customization, but CI/CD
pipelines still need scripting for:
- Building and pushing container images
- Running tests
- Coordinating multi-service deployments
- Managing secrets rotation
- Implementing approval gates
- Handling rollback logic

Kustomize is one piece of the pipeline, not a replacement for it.

### 4. "You cannot use Kustomize with Helm charts."

**Misconception.** Kustomize v4.1+ has a `helmChart` field that renders Helm
charts inline:

```yaml
helmChart:
  name: redis
  repo: https://charts.bitnami.com/bitnami
  version: 17.0.0
  valuesInline:
    architecture: standalone
```

You can also pipe `helm template` output into `kustomize build`:

```bash
helm template my-release bitnami/redis | kustomize build
```

### 5. "Kustomize overlays create copy-paste drift over time."

**Valid concern, but solvable.** If team members create overlays by copying
entire YAML files instead of using patches, drift accumulates. The solution is:
- Use `patches` with `target` selectors instead of full file copies
- Use `components` for shared pieces
- Enforce review standards that flag duplicated YAML in PRs
- Use `kustomize build` output diffing to detect drift

---

## Common Mistakes

1. **Choosing Helm for simple use cases.** If you only need value changes
   across environments, Helm adds unnecessary complexity (chart packaging,
   values files, template debugging).

2. **Choosing Kustomize for complex dependency trees.** If you need to compose
   5+ independent components with versioned releases, Kustomize's file-based
   references become unwieldy.

3. **Treating the tools as competitors.** They solve different problems.
   Kustomize is for customizing YAML. Helm is for packaging and distributing
   applications. Use them together when the situation calls for it.

4. **Ignoring the GitOps implications.** Kustomize's deterministic output is a
   major advantage for GitOps because the Git diff always reflects the actual
   change. Helm's template rendering can obscure what changed between versions.
