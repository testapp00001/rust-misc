# Exercise 03: Environment-Specific Values Files (Independent)

## Objective

Extend the `webapp-chart` from Exercise 02 (or create a fresh one) so
that it supports three environments -- `dev`, `staging`, and `production`
-- each with its own values file and appropriate settings.

## Requirements

Create three values files that override the base `values.yaml` for each
environment:

| Setting | dev | staging | production |
|---------|-----|---------|------------|
| `replicaCount` | 1 | 2 | 5 |
| `image.tag` | latest | 1.0.0 | 1.0.0 |
| `service.type` | ClusterIP | ClusterIP | LoadBalancer |
| `ingress.enabled` | false | true | true |
| `resources.requests.cpu` | 50m | 100m | 250m |
| `resources.requests.memory` | 64Mi | 128Mi | 256Mi |
| `resources.limits.cpu` | 100m | 250m | 500m |
| `resources.limits.memory` | 128Mi | 256Mi | 512Mi |
| `env.APP_ENV` | development | staging | production |
| `ingress.hosts[0].host` | -- | staging.webapp.local | webapp.example.com |
| `autoscaling.enabled` | false | false | true |
| `autoscaling.minReplicas` | -- | -- | 3 |
| `autoscaling.maxReplicas` | -- | -- | 10 |
| `autoscaling.targetCPU` | -- | -- | 80 |

## Instructions

### Step 1 -- Create the values overlay files

```
webapp-chart/
  values.yaml              # base defaults
  values/
    dev.yaml
    staging.yaml
    production.yaml
```

### Step 2 -- Add a HorizontalPodAutoscaler template

Create `templates/hpa.yaml`:

- Wrap the entire file in `{{- if .Values.autoscaling.enabled }}`.
- Set `minReplicas` and `maxReplicas` from values.
- Target CPU utilization from `autoscaling.targetCPU`.
- Reference the Deployment name via the `webapp.fullname` helper.

### Step 3 -- Update values.yaml

Add the `autoscaling` block to the base `values.yaml`:

```yaml
autoscaling:
  enabled: false
  minReplicas: 1
  maxReplicas: 10
  targetCPU: 80
```

### Step 4 -- Validate rendering for each environment

```bash
helm template myrelease . -f values/dev.yaml
helm template myrelease . -f values/staging.yaml
helm template myrelease . -f values/production.yaml
```

For each render, confirm:

- The correct replica count appears.
- Ingress is present or absent as expected.
- HPA is present or absent as expected.
- Resource requests and limits match the table above.

### Step 5 -- Install and verify (optional)

```bash
helm install webapp-dev . -f values/dev.yaml --namespace dev --create-namespace
helm install webapp-staging . -f values/staging.yaml --namespace staging --create-namespace
helm install webapp-prod . -f values/production.yaml --namespace prod --create-namespace
```

Verify the running resources match expectations.

## Success Criteria

- [ ] Three environment values files exist and each overrides the base
      `values.yaml` correctly.
- [ ] `helm template` with `-f values/<env>.yaml` produces correct output
      for all three environments.
- [ ] The HPA template is rendered only when `autoscaling.enabled=true`.
- [ ] Values not overridden in an environment file fall back to the base
      `values.yaml` defaults.
- [ ] You can explain the Helm value merge precedence (base values are
      overridden by file overlays, which are overridden by `--set` flags).

## Hints

<details>
<summary>Hint: Value precedence</summary>

Helm merges values in this order (lowest to highest priority):

1. `values.yaml` inside the chart
2. Parent chart's `values.yaml` (if this is a subchart)
3. Values files passed with `-f` (in order; last wins)
4. `--set` and `--set-string` flags on the command line

</details>

<details>
<summary>Hint: HPA template skeleton</summary>

```yaml
{{- if .Values.autoscaling.enabled }}
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: {{ include "webapp.fullname" . }}
  labels:
    {{- include "webapp.labels" . | nindent 4 }}
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: {{ include "webapp.fullname" . }}
  minReplicas: {{ .Values.autoscaling.minReplicas }}
  maxReplicas: {{ .Values.autoscaling.maxReplicas }}
  metrics:
    - type: Resource
      resource:
        name: cpu
        target:
          type: Utilization
          averageUtilization: {{ .Values.autoscaling.targetCPU }}
{{- end }}
```

</details>

<details>
<summary>Hint: Verifying environment diffs</summary>

Render two environments side by side:

```bash
diff <(helm template r . -f values/dev.yaml) \
     <(helm template r . -f values/production.yaml)
```

This makes it easy to spot differences in replica counts, resource limits,
ingress rules, and HPA presence.

</details>
