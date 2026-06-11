# Exercise 02: Create a Helm Chart for a Web Application (Guided)

## Objective

Build a complete Helm chart from scratch for a simple web application.
You will create a Deployment, Service, ConfigMap, and Ingress, and
parameterize everything through `values.yaml`.

## Application Details

- **Name:** `webapp`
- **Container image:** `nginx:1.25-alpine`
- **Port:** 80
- **Replicas:** 2
- **Environment variable:** `APP_ENV` set to `production`

## Instructions

### Step 1 -- Scaffold the chart

```bash
mkdir webapp-chart && cd webapp-chart
```

Create the standard directory structure manually (do **not** use
`helm create`):

```
webapp-chart/
  Chart.yaml
  values.yaml
  templates/
    _helpers.tpl
    deployment.yaml
    service.yaml
    configmap.yaml
    ingress.yaml
    NOTES.txt
```

### Step 2 -- Write Chart.yaml

Fill in the metadata:

| Field | Value |
|-------|-------|
| apiVersion | v2 |
| name | webapp |
| description | A Helm chart for the demo web application |
| version | 0.1.0 |
| appVersion | "1.0.0" |

### Step 3 -- Write values.yaml

Create a well-structured `values.yaml` that exposes:

- `replicaCount` (default `2`)
- `image.repository` (default `nginx`)
- `image.tag` (default `1.25-alpine`)
- `image.pullPolicy` (default `IfNotPresent`)
- `service.type` (default `ClusterIP`)
- `service.port` (default `80`)
- `ingress.enabled` (default `false`)
- `ingress.hosts` -- a list with one entry: `host: webapp.local`, paths: `/`
- `env.APP_ENV` (default `production`)
- `resources.limits.cpu` and `resources.limits.memory`
- `resources.requests.cpu` and `resources.requests.memory`

### Step 4 -- Write _helpers.tpl

Define the following named templates:

- `webapp.name` -- returns the chart name
- `webapp.fullname` -- returns `<release-name>-webapp` (truncated to 63
  chars)
- `webapp.labels` -- returns standard labels (`helm.sh/chart`,
  `app.kubernetes.io/name`, `app.kubernetes.io/instance`,
  `app.kubernetes.io/version`, `app.kubernetes.io/managed-by`)
- `webapp.selectorLabels` -- returns `app.kubernetes.io/name` and
  `app.kubernetes.io/instance`

### Step 5 -- Write templates

**deployment.yaml**
- Use `webapp.fullname` for the Deployment name.
- Use `webapp.labels` and `webapp.selectorLabels`.
- Wire `replicaCount`, image, resources, and `APP_ENV` from values.
- Inject `APP_ENV` via a ConfigMap reference.

**service.yaml**
- Use `webapp.fullname` for the Service name.
- Wire `service.type` and `service.port`.

**configmap.yaml**
- Create a ConfigMap containing the `APP_ENV` key.

**ingress.yaml**
- Wrap the entire template in `{{- if .Values.ingress.enabled }}`.
- Iterate over `.Values.ingress.hosts` to produce Ingress rules.

**NOTES.txt**
- Print the application URL based on service type and ingress settings.

### Step 6 -- Validate and render

```bash
# Lint the chart
helm lint .

# Render templates to stdout
helm template myrelease .

# If you have a cluster, do a dry-run install
helm install myrelease . --dry-run
```

### Step 7 -- Install (optional, requires a cluster)

```bash
helm install myrelease . --namespace webapp --create-namespace
kubectl get all -n webapp
```

## Success Criteria

- [ ] `helm lint` passes with no errors.
- [ ] `helm template` produces valid YAML for Deployment, Service, and
      ConfigMap.
- [ ] Ingress is omitted from rendered output when
      `ingress.enabled=false`.
- [ ] Changing `replicaCount` in values changes the rendered Deployment.
- [ ] The `APP_ENV` environment variable is sourced from a ConfigMap.
- [ ] Standard labels appear on every resource.

## Hints

<details>
<summary>Hint: helpers.tpl structure</summary>

```yaml
{{- define "webapp.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name "webapp" | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
```

</details>

<details>
<summary>Hint: Iterating ingress hosts</summary>

```yaml
{{- range .Values.ingress.hosts }}
  - host: {{ .host | quote }}
    http:
      paths:
        {{- range .paths }}
        - path: {{ . }}
          pathType: Prefix
          backend:
            service:
              name: {{ include "webapp.fullname" $ }}
              port:
                number: {{ $.Values.service.port }}
        {{- end }}
{{- end }}
```

Note the `$` to access the root context inside `range`.

</details>

<details>
<summary>Hint: ConfigMap for env</summary>

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: {{ include "webapp.fullname" . }}
data:
  APP_ENV: {{ .Values.env.APP_ENV | quote }}
```

Reference it in the container spec:

```yaml
envFrom:
  - configMapRef:
      name: {{ include "webapp.fullname" . }}
```

</details>
