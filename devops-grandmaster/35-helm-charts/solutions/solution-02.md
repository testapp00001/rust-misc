# Solution 02: Create a Helm Chart for a Web Application

## Chart structure

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

## Chart.yaml

```yaml
apiVersion: v2
name: webapp
description: A Helm chart for the demo web application
type: application
version: 0.1.0
appVersion: "1.0.0"
```

## values.yaml

```yaml
replicaCount: 2

image:
  repository: nginx
  tag: "1.25-alpine"
  pullPolicy: IfNotPresent

service:
  type: ClusterIP
  port: 80

ingress:
  enabled: false
  className: ""
  annotations: {}
  hosts:
    - host: webapp.local
      paths:
        - path: /
          pathType: Prefix
  tls: []

env:
  APP_ENV: production

resources:
  limits:
    cpu: 200m
    memory: 256Mi
  requests:
    cpu: 100m
    memory: 128Mi

nameOverride: ""
fullnameOverride: ""
```

## templates/_helpers.tpl

```yaml
{{/*
Expand the name of the chart.
*/}}
{{- define "webapp.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
*/}}
{{- define "webapp.fullname" -}}
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
Create chart name and version as used by the chart label.
*/}}
{{- define "webapp.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "webapp.labels" -}}
helm.sh/chart: {{ include "webapp.chart" . }}
{{ include "webapp.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/*
Selector labels
*/}}
{{- define "webapp.selectorLabels" -}}
app.kubernetes.io/name: {{ include "webapp.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}
```

## templates/configmap.yaml

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: {{ include "webapp.fullname" . }}
  labels:
    {{- include "webapp.labels" . | nindent 4 }}
data:
  APP_ENV: {{ .Values.env.APP_ENV | quote }}
```

## templates/deployment.yaml

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: {{ include "webapp.fullname" . }}
  labels:
    {{- include "webapp.labels" . | nindent 4 }}
spec:
  replicas: {{ .Values.replicaCount }}
  selector:
    matchLabels:
      {{- include "webapp.selectorLabels" . | nindent 6 }}
  template:
    metadata:
      labels:
        {{- include "webapp.selectorLabels" . | nindent 8 }}
    spec:
      containers:
        - name: {{ .Chart.Name }}
          image: "{{ .Values.image.repository }}:{{ .Values.image.tag }}"
          imagePullPolicy: {{ .Values.image.pullPolicy }}
          ports:
            - name: http
              containerPort: {{ .Values.service.port }}
              protocol: TCP
          envFrom:
            - configMapRef:
                name: {{ include "webapp.fullname" . }}
          resources:
            {{- toYaml .Values.resources | nindent 12 }}
```

## templates/service.yaml

```yaml
apiVersion: v1
kind: Service
metadata:
  name: {{ include "webapp.fullname" . }}
  labels:
    {{- include "webapp.labels" . | nindent 4 }}
spec:
  type: {{ .Values.service.type }}
  ports:
    - port: {{ .Values.service.port }}
      targetPort: http
      protocol: TCP
      name: http
  selector:
    {{- include "webapp.selectorLabels" . | nindent 4 }}
```

## templates/ingress.yaml

```yaml
{{- if .Values.ingress.enabled -}}
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: {{ include "webapp.fullname" . }}
  labels:
    {{- include "webapp.labels" . | nindent 4 }}
  {{- with .Values.ingress.annotations }}
  annotations:
    {{- toYaml . | nindent 4 }}
  {{- end }}
spec:
  {{- if .Values.ingress.className }}
  ingressClassName: {{ .Values.ingress.className }}
  {{- end }}
  {{- if .Values.ingress.tls }}
  tls:
    {{- range .Values.ingress.tls }}
    - hosts:
        {{- range .hosts }}
        - {{ . | quote }}
        {{- end }}
      secretName: {{ .secretName }}
    {{- end }}
  {{- end }}
  rules:
    {{- range .Values.ingress.hosts }}
    - host: {{ .host | quote }}
      http:
        paths:
          {{- range .paths }}
          - path: {{ .path }}
            pathType: {{ .pathType }}
            backend:
              service:
                name: {{ include "webapp.fullname" $ }}
                port:
                  number: {{ $.Values.service.port }}
          {{- end }}
    {{- end }}
{{- end }}
```

## templates/NOTES.txt

```
Thank you for installing {{ include "webapp.fullname" . }}!

{{- if .Values.ingress.enabled }}

The application is available at:
{{- range .Values.ingress.hosts }}
  http{{ if $.Values.ingress.tls }}s{{ end }}://{{ .host }}
{{- end }}

{{- else if eq .Values.service.type "LoadBalancer" }}

Get the application URL by running:
  export SERVICE_IP=$(kubectl get svc {{ include "webapp.fullname" . }} \
    --namespace {{ .Release.Namespace }} \
    -o jsonpath='{.status.loadBalancer.ingress[0].ip}')
  echo http://$SERVICE_IP:{{ .Values.service.port }}

{{- else }}

Access the application by running:
  kubectl port-forward svc/{{ include "webapp.fullname" . }} \
    8080:{{ .Values.service.port }} --namespace {{ .Release.Namespace }}
  echo http://localhost:8080

{{- end }}
```

## Validation

```bash
$ helm lint .
==> Linting .
[INFO] Chart.yaml: icon is recommended
1 chart(s) linted, 0 chart(s) failed

$ helm template myrelease .
# Renders all four resources (ConfigMap, Deployment, Service, NOTES).
# Ingress is absent because ingress.enabled=false by default.
```
