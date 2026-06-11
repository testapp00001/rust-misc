# Solution 01: Chart Structure and Templating

## Part A -- File purposes

After running `helm create demo`, the directory tree looks like:

```
demo/
  Chart.yaml
  Chart.lock
  values.yaml
  charts/
  templates/
    NOTES.txt
    _helpers.tpl
    deployment.yaml
    hpa.yaml
    ingress.yaml
    service.yaml
    serviceaccount.yaml
    tests/
      test-connection.yaml
  .helmignore
```

File descriptions:

| File | Purpose |
|------|---------|
| `Chart.yaml` | Declares chart metadata: name, version, apiVersion, description, appVersion, and dependencies. |
| `values.yaml` | Provides default configuration values that templates consume via `.Values`. |
| `templates/deployment.yaml` | Templated Deployment resource; parameterized with `.Values` for replicas, image, ports, and probes. |
| `templates/service.yaml` | Templated Service resource; wire type and port from values. |
| `templates/_helpers.tpl` | Contains `define` blocks that produce reusable template snippets (partials) for naming, labels, and selectors. |
| `templates/NOTES.txt` | Post-install text displayed to the user after `helm install`. |
| `.helmignore` | Lists patterns (like `.git`, `*.md`) to exclude when packaging the chart into a `.tgz` archive. |

## Part B -- Template expressions

Typical `deployment.yaml` template expressions and their value paths:

| Expression | Values path |
|-----------|-------------|
| `{{ .Values.replicaCount }}` | `replicaCount` |
| `{{ .Values.image.repository }}` | `image.repository` |
| `{{ .Values.image.tag }}` | `image.tag` |
| `{{ .Values.image.pullPolicy }}` | `image.pullPolicy` |
| `{{ .Values.serviceAccount.create }}` | `serviceAccount.create` |
| `{{ .Values.serviceAccount.name }}` | `serviceAccount.name` |
| `{{ .Values.podSecurityContext }}` | `podSecurityContext` |
| `{{ .Values.securityContext }}` | `securityContext` |
| `{{ .Values.resources }}` | `resources` |
| `{{ .Values.nodeSelector }}` | `nodeSelector` |
| `{{ .Values.tolerations }}` | `tolerations` |
| `{{ .Values.affinity }}` | `affinity` |

Changing `replicaCount` from `1` to `3` in `values.yaml` and re-running
`helm template myrelease demo/` produces `replicas: 3` in the rendered
Deployment output.

## Part C -- include vs template

**`template`** inserts a named template inline. It writes directly into
the output stream and cannot be piped through other functions.

```yaml
{{ template "demo.labels" . }}
```

**`include`** renders a named template and returns the result as a string.
Because it returns a string, you can pipe it through functions like
`nindent`, `quote`, or `trim`.

```yaml
{{ include "demo.labels" . | nindent 4 }}
```

In practice, `include` is preferred because indentation control is almost
always needed.

### Helpers (_helpers.tpl)

The `_helpers.tpl` file contains `define` blocks. Each block creates a
named, reusable template:

```yaml
{{- define "demo.labels" -}}
helm.sh/chart: {{ include "demo.chart" . }}
app.kubernetes.io/name: {{ include "demo.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}
```

Other templates reference these with `include` or `template`, keeping
DRY (Don't Repeat Yourself).

### Adding a custom helper

```yaml
{{- define "demo.fullname" -}}
{{- printf "%s-%s" .Release.Name "demo" | trunc 63 | trimSuffix "-" }}
{{- end }}
```

Then in `deployment.yaml`:

```yaml
metadata:
  name: {{ include "demo.fullname" . }}
```

Running `helm template myrelease demo/` shows `name: myrelease-demo`.

## Part D -- Built-in objects

**`.Release`** contains information about the current release. Key
properties include `.Release.Name` (the name passed to `helm install`),
`.Release.Namespace` (the target namespace), `.Release.Revision` (the
current revision number), and `.Release.IsUpgrade` (true during an
upgrade, false during initial install).

**`.Chart`** contains the metadata from `Chart.yaml`. You reference it
when you need the chart name, version, or appVersion -- for example, in
labels or annotations: `app.kubernetes.io/version: {{ .Chart.AppVersion }}`.

**`.Capabilities`** describes the Kubernetes cluster's capabilities. The
most useful property is `.Capabilities.KubeVersion`, which lets you
conditionally enable features based on the cluster version. For example,
you might use an Ingress API version that differs between Kubernetes 1.18
and 1.22.
