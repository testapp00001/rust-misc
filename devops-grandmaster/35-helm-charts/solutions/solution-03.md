# Solution 03: Environment-Specific Values Files

## Directory structure

```
webapp-chart/
  Chart.yaml
  values.yaml              # base defaults
  values/
    dev.yaml
    staging.yaml
    production.yaml
  templates/
    _helpers.tpl
    deployment.yaml
    service.yaml
    configmap.yaml
    ingress.yaml
    hpa.yaml
    NOTES.txt
```

## values/dev.yaml

```yaml
replicaCount: 1

image:
  tag: "latest"

service:
  type: ClusterIP

ingress:
  enabled: false

resources:
  requests:
    cpu: 50m
    memory: 64Mi
  limits:
    cpu: 100m
    memory: 128Mi

env:
  APP_ENV: development

autoscaling:
  enabled: false
```

## values/staging.yaml

```yaml
replicaCount: 2

image:
  tag: "1.0.0"

service:
  type: ClusterIP

ingress:
  enabled: true
  hosts:
    - host: staging.webapp.local
      paths:
        - path: /
          pathType: Prefix

resources:
  requests:
    cpu: 100m
    memory: 128Mi
  limits:
    cpu: 250m
    memory: 256Mi

env:
  APP_ENV: staging

autoscaling:
  enabled: false
```

## values/production.yaml

```yaml
replicaCount: 5

image:
  tag: "1.0.0"

service:
  type: LoadBalancer

ingress:
  enabled: true
  hosts:
    - host: webapp.example.com
      paths:
        - path: /
          pathType: Prefix

resources:
  requests:
    cpu: 250m
    memory: 256Mi
  limits:
    cpu: 500m
    memory: 512Mi

env:
  APP_ENV: production

autoscaling:
  enabled: true
  minReplicas: 3
  maxReplicas: 10
  targetCPU: 80
```

## templates/hpa.yaml

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

## Validation commands

```bash
# Dev: 1 replica, no ingress, no HPA
helm template myrelease . -f values/dev.yaml

# Staging: 2 replicas, ingress enabled, no HPA
helm template myrelease . -f values/staging.yaml

# Production: 5 replicas, ingress enabled, HPA enabled
helm template myrelease . -f values/production.yaml

# Side-by-side diff
diff <(helm template r . -f values/dev.yaml) \
     <(helm template r . -f values/production.yaml)
```

## Value precedence explanation

Helm merges values in this order, from lowest to highest priority:

1. **Chart's `values.yaml`** -- the base defaults bundled with the chart.
2. **Parent chart's values** -- if this chart is a subchart (dependency),
   the parent's values block for this chart takes precedence.
3. **Values files (`-f`)** -- files passed on the command line. When
   multiple `-f` flags are given, later files override earlier ones.
4. **`--set` / `--set-string` / `--set-file`** -- highest priority.
   These override everything.

In this exercise, the environment files (step 3) override the base
`values.yaml` (step 1). Any key not present in the environment file
falls back to the base default. For example, `image.repository` is
`nginx` in the base and is never overridden, so all environments use
`nginx`.
