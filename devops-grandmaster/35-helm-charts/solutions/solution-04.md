# Solution 04: Helm Library Chart for Shared Templates

## Directory structure

```
shared-library/
  Chart.yaml
  templates/
    _helpers.tpl

app-a/
  Chart.yaml
  values.yaml
  templates/
    deployment.yaml
    service.yaml

app-b/
  Chart.yaml
  values.yaml
  templates/
    deployment.yaml
    service.yaml
```

## shared-library/Chart.yaml

```yaml
apiVersion: v2
name: shared-library
description: Common templates for all team charts
version: 0.1.0
type: library
```

The `type: library` field is what makes this a library chart. Without it
Helm would treat it as a regular application chart and expect installable
resources.

## shared-library/templates/_helpers.tpl

```yaml
{{/*
Expand the name of the chart.
*/}}
{{- define "shared-library.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Fully qualified app name.
*/}}
{{- define "shared-library.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{/*
Chart label value.
*/}}
{{- define "shared-library.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels.
*/}}
{{- define "shared-library.labels" -}}
helm.sh/chart: {{ include "shared-library.chart" . }}
{{ include "shared-library.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/*
Selector labels.
*/}}
{{- define "shared-library.selectorLabels" -}}
app.kubernetes.io/name: {{ include "shared-library.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Service account name.
*/}}
{{- define "shared-library.serviceAccountName" -}}
{{- if .Values.serviceAccount.name }}
{{- .Values.serviceAccount.name }}
{{- else }}
{{- include "shared-library.fullname" . }}
{{- end }}
{{- end }}

{{/*
Deployment resource.
*/}}
{{- define "shared-library.deployment" -}}
apiVersion: apps/v1
kind: Deployment
metadata:
  name: {{ include "shared-library.fullname" . }}
  labels:
    {{- include "shared-library.labels" . | nindent 4 }}
spec:
  {{- if not .Values.autoscaling }}
  replicas: {{ .Values.replicaCount }}
  {{- end }}
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
            - name: http
              containerPort: {{ .Values.service.port }}
              protocol: TCP
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
          resources:
            {{- toYaml .Values.resources | nindent 12 }}
{{- end }}

{{/*
Service resource.
*/}}
{{- define "shared-library.service" -}}
apiVersion: v1
kind: Service
metadata:
  name: {{ include "shared-library.fullname" . }}
  labels:
    {{- include "shared-library.labels" . | nindent 4 }}
spec:
  type: {{ .Values.service.type }}
  ports:
    - port: {{ .Values.service.port }}
      targetPort: http
      protocol: TCP
      name: http
  selector:
    {{- include "shared-library.selectorLabels" . | nindent 4 }}
{{- end }}
```

## app-a/Chart.yaml

```yaml
apiVersion: v2
name: app-a
description: Application A
version: 0.1.0
appVersion: "1.0.0"
dependencies:
  - name: shared-library
    version: 0.1.0
    repository: file://../shared-library
```

## app-a/values.yaml

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
    periodSeconds: 10
  readiness:
    httpGet:
      path: /readyz
      port: 8080
    initialDelaySeconds: 5
    periodSeconds: 5
```

## app-a/templates/deployment.yaml

```yaml
{{- include "shared-library.deployment" . }}
```

## app-a/templates/service.yaml

```yaml
{{- include "shared-library.service" . }}
```

## app-b/Chart.yaml

```yaml
apiVersion: v2
name: app-b
description: Application B
version: 0.1.0
appVersion: "1.0.0"
dependencies:
  - name: shared-library
    version: 0.1.0
    repository: file://../shared-library
```

## app-b/values.yaml

```yaml
nameOverride: ""
fullnameOverride: ""
replicaCount: 2

image:
  repository: myorg/app-b
  tag: "1.0.0"
  pullPolicy: IfNotPresent

service:
  type: ClusterIP
  port: 3000

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
      path: /api/health
      port: 3000
    initialDelaySeconds: 10
    periodSeconds: 10
  readiness:
    httpGet:
      path: /api/ready
      port: 3000
    initialDelaySeconds: 5
    periodSeconds: 5
```

## app-b/templates/deployment.yaml

```yaml
{{- include "shared-library.deployment" . }}
```

## app-b/templates/service.yaml

```yaml
{{- include "shared-library.service" . }}
```

## Build and validate

```bash
cd app-a && helm dependency update && cd ..
cd app-b && helm dependency update && cd ..

helm lint app-a/   # Should pass
helm lint app-b/   # Should pass

helm template release-a app-a/
# Shows Deployment on port 8080 with /healthz and /readyz probes

helm template release-b app-b/
# Shows Deployment on port 3000 with /api/health and /api/ready probes
```

## How the dependency resolution works

When you run `helm dependency update`, Helm reads `Chart.yaml`, finds the
`file://../shared-library` reference, packages the library chart into
`charts/shared-library-0.1.0.tgz`, and writes `Chart.lock`. The consuming
chart's templates can then `include` any template defined in the library.

If you update a template in `shared-library`, you must re-run
`helm dependency update` in both `app-a` and `app-b` to pick up the
change. Alternatively, during development, use a symlink:

```bash
ln -s ../../shared-library app-a/charts/shared-library
ln -s ../../shared-library app-b/charts/shared-library
```

This lets you iterate without re-packaging after every edit.
