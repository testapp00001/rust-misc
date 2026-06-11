# Exercise 04: Helm Library Chart for Shared Templates (Challenge)

## Objective

Create a **Helm library chart** that contains reusable template
definitions, then consume it from two different application charts. This
exercise teaches you how to eliminate template duplication across multiple
charts in an organization.

## Background

A **library chart** (apiVersion: v2, type: library) contains only
template definitions (in `_helpers.tpl` or other files) and no
installable resources. Application charts declare it as a dependency and
use `{{ include }}` to pull in the shared templates.

## Architecture

```
shared-library/          # library chart
  Chart.yaml             # type: library
  templates/
    _helpers.tpl         # shared template definitions

app-a/                   # application chart A
  Chart.yaml             # depends on shared-library
  values.yaml
  templates/
    ...

app-b/                   # application chart B
  Chart.yaml             # depends on shared-library
  values.yaml
  templates/
    ...
```

## Instructions

### Step 1 -- Create the library chart

Create `shared-library/` with this structure:

```
shared-library/
  Chart.yaml
  templates/
    _helpers.tpl
```

**Chart.yaml:**

| Field | Value |
|-------|-------|
| apiVersion | v2 |
| name | shared-library |
| description | Common templates for all team charts |
| version | 0.1.0 |
| type | library |

### Step 2 -- Define shared templates in _helpers.tpl

Create the following reusable templates. Each must accept the root
context (`.`) as its argument.

**`shared-library.name`** -- Returns chart name or `.Values.nameOverride`.

**`shared-library.fullname`** -- Returns a unique name based on release
and chart name. Handle `.Values.fullnameOverride`.

**`shared-library.labels`** -- Returns a complete label set:

```yaml
helm.sh/chart: <chart>-<version>
app.kubernetes.io/name: <name>
app.kubernetes.io/instance: <release>
app.kubernetes.io/version: <appVersion>
app.kubernetes.io/managed-by: Helm
```

**`shared-library.selectorLabels`** -- Returns:

```yaml
app.kubernetes.io/name: <name>
app.kubernetes.io/instance: <release>
```

**`shared-library.serviceAccountName`** -- Returns the service account
name from `.Values.serviceAccount.name`, or falls back to the fullname.

**`shared-library.deployment`** -- A template that renders a full
Deployment resource. It should:

- Accept context `.` as argument.
- Use `shared-library.fullname` for the name.
- Use `shared-library.labels` and `shared-library.selectorLabels`.
- Wire `replicaCount`, image, ports, resources, env, and
  `serviceAccountName` from values.
- Support optional liveness/readiness probes from values.

**`shared-library.service`** -- A template that renders a Service.
Wire type and port from values.

### Step 3 -- Create app-a

Create `app-a/`:

**Chart.yaml** -- name: `app-a`, version `0.1.0`, appVersion `"1.0.0"`.
Add a dependency on `shared-library` at version `0.1.0` from the local
path `../shared-library`.

**values.yaml:**

```yaml
nameOverride: ""
fullnameOverride: ""
replicaCount: 2
image:
  repository: myorg/app-a
  tag: "1.0.0"
  pullPolicy: IfNotPresent
service:
  type: ClusterIP
  port: 8080
resources:
  requests:
    cpu: 100m
    memory: 128Mi
  limits:
    cpu: 250m
    memory: 256Mi
serviceAccount:
  name: ""
probes:
  liveness:
    httpGet:
      path: /healthz
      port: 8080
    initialDelaySeconds: 10
  readiness:
    httpGet:
      path: /readyz
      port: 8080
    initialDelaySeconds: 5
```

**templates/deployment.yaml** -- Use the library:

```yaml
{{- include "shared-library.deployment" . }}
```

**templates/service.yaml:**

```yaml
{{- include "shared-library.service" . }}
```

### Step 4 -- Create app-b

Create `app-b/` following the same pattern as `app-a` but with:

- `image.repository: myorg/app-b`
- `service.port: 3000`
- Different probe paths (`/api/health` and `/api/ready`)

### Step 5 -- Build dependencies and validate

```bash
cd app-a && helm dependency update && cd ..
cd app-b && helm dependency update && cd ..

helm lint app-a/
helm lint app-b/

helm template release-a app-a/
helm template release-b app-b/
```

### Step 6 -- Install (optional)

```bash
helm install app-a app-a/ --namespace apps --create-namespace
helm install app-b app-b/ --namespace apps
```

## Success Criteria

- [ ] `shared-library` has `type: library` in `Chart.yaml`.
- [ ] Library chart contains no Kubernetes resources of its own -- only
      `define` blocks.
- [ ] Both `app-a` and `app-b` declare `shared-library` as a dependency.
- [ ] `helm lint` passes for all three charts.
- [ ] `helm template` for both apps produces correct Deployment and
      Service resources.
- [ ] The two apps share identical label logic, naming conventions, and
      Deployment structure without copy-pasting templates.
- [ ] Updating a template in the library propagates to both apps after
      `helm dependency update`.

## Hints

<details>
<summary>Hint: Library Chart.yaml</summary>

```yaml
apiVersion: v2
name: shared-library
description: Shared Helm templates
version: 0.1.0
type: library
```

The key field is `type: library`. Without it Helm treats it as an
application chart.

</details>

<details>
<summary>Hint: Local path dependency</summary>

In `app-a/Chart.yaml`:

```yaml
dependencies:
  - name: shared-library
    version: 0.1.0
    repository: file://../shared-library
```

After adding the dependency, run:

```bash
helm dependency update app-a/
```

This creates `app-a/charts/shared-library-0.1.0.tgz`.

</details>

<details>
<summary>Hint: Deployment template in library</summary>

```yaml
{{- define "shared-library.deployment" -}}
apiVersion: apps/v1
kind: Deployment
metadata:
  name: {{ include "shared-library.fullname" . }}
  labels:
    {{- include "shared-library.labels" . | nindent 4 }}
spec:
  replicas: {{ .Values.replicaCount }}
  selector:
    matchLabels:
      {{- include "shared-library.selectorLabels" . | nindent 6 }}
  template:
    metadata:
      labels:
        {{- include "shared-library.selectorLabels" . | nindent 8 }}
    spec:
      serviceAccountName: {{ include "shared-library.serviceAccountName" . }}
      containers:
        - name: {{ .Chart.Name }}
          image: "{{ .Values.image.repository }}:{{ .Values.image.tag }}"
          imagePullPolicy: {{ .Values.image.pullPolicy }}
          ports:
            - containerPort: {{ .Values.service.port }}
          resources:
            {{- toYaml .Values.resources | nindent 12 }}
          {{- if .Values.probes }}
          {{- if .Values.probes.liveness }}
          livenessProbe:
            {{- toYaml .Values.probes.liveness | nindent 12 }}
          {{- end }}
          {{- if .Values.probes.readiness }}
          readinessProbe:
            {{- toYaml .Values.probes.readiness | nindent 12 }}
          {{- end }}
          {{- end }}
{{- end }}
```

</details>

<details>
<summary>Hint: Using library templates from app charts</summary>

In `app-a/templates/deployment.yaml`:

```yaml
{{- include "shared-library.deployment" . }}
```

That single line renders the entire Deployment. The library templates
access `.Values` from the consuming chart's context because you pass `.`.

</details>
