# Module 35: Helm Charts — Package Management for Kubernetes

## The Problem: YAML Sprawl

A real production Kubernetes application isn't one YAML file. It's dozens:

```
my-app/
├── namespace.yaml
├── configmap.yaml
├── secret.yaml
├── deployment.yaml
├── service.yaml
├── ingress.yaml
├── hpa.yaml
├── pdb.yaml
├── serviceaccount.yaml
├── networkpolicy.yaml
└── ... (20 more files)
```

Now multiply by environments:

```
my-app-dev/          (different image tag, fewer replicas, no ingress)
my-app-staging/      (medium replicas, staging ingress)
my-app-prod/         (full replicas, production ingress, HPA, PDB)
```

And add third-party software:

```
Installing Redis:
  1. Find the official Redis k8s manifests
  2. Copy 15 YAML files
  3. Modify for your cluster
  4. Apply them in the right order
  5. Repeat for every environment
  6. When Redis updates, find new manifests, diff, merge...
```

**Problems:**
- No templating (can't change image tag in one place)
- No versioning (which version of Redis did we deploy?)
- No rollback (kubectl apply doesn't track history)
- No dependency management (app depends on Redis, Redis depends on PVC)
- Sharing and reusing is manual copy-paste

## The Naive Way: Shell Scripts and Copy-Paste

```bash
#!/bin/bash
# deploy.sh — the "DevOps" script
ENV=$1
NAMESPACE=$2

if [ "$ENV" == "dev" ]; then
  REPLICAS=1
  IMAGE_TAG="latest"
  INGRESS_HOST="dev.myapp.local"
elif [ "$ENV" == "staging" ]; then
  REPLICAS=3
  IMAGE_TAG="staging-abc123"
  INGRESS_HOST="staging.myapp.com"
elif [ "$ENV" == "prod" ]; then
  REPLICAS=5
  IMAGE_TAG="v1.2.3"
  INGRESS_HOST="myapp.com"
fi

# sed-based templating (fragile, unreadable)
sed "s/REPLICAS/$REPLICAS/g; s/IMAGE_TAG/$IMAGE_TAG/g; s/INGRESS_HOST/$INGRESS_HOST/g" \
  deployment.yaml.template | kubectl apply -f -
```

**Why this fails:**
- Bash scripts are fragile and hard to test
- No rollback mechanism
- No dependency tracking
- Can't share with other teams
- Error-prone string replacement

## The Right Way: Helm

Helm is the **package manager** for Kubernetes, like apt for Debian or brew for macOS.

```
┌──────────────────────────────────────────────────────────┐
│                     Helm Concepts                         │
├──────────────────────────────────────────────────────────┤
│                                                           │
│  Chart      = A package of Kubernetes manifests           │
│               (like a .deb or .rpm file)                  │
│                                                           │
│  Values     = Configuration that customizes the chart     │
│               (like a config file for the package)        │
│                                                           │
│  Release    = A specific deployment of a chart            │
│               (like an installed package instance)        │
│                                                           │
│  Repository = A collection of charts for download         │
│               (like an apt repository)                    │
│                                                           │
└──────────────────────────────────────────────────────────┘
```

### Installing Helm

```bash
# Linux
curl https://raw.githubusercontent.com/helm/helm/main/scripts/get-helm-3 | bash

# macOS
brew install helm

# Verify
helm version
```

### Chart Structure

```
my-chart/
├── Chart.yaml              # Chart metadata (name, version, description)
├── Chart.lock              # Dependency lock file
├── values.yaml             # Default configuration values
├── values.schema.json      # Optional: validate values
├── templates/              # Kubernetes manifest templates
│   ├── deployment.yaml
│   ├── service.yaml
│   ├── ingress.yaml
│   ├── configmap.yaml
│   ├── _helpers.tpl        # Template helpers (partial templates)
│   ├── NOTES.txt           # Post-install notes shown to user
│   └── tests/              # Test pods to verify the chart works
│       └── test-connection.yaml
├── charts/                 # Subchart dependencies
├── crds/                   # Custom Resource Definitions
└── .helmignore             # Files to exclude when packaging
```

### values.yaml: The Configuration Layer

```yaml
# values.yaml — default values for the chart
replicaCount: 3

image:
  repository: myapp
  tag: "1.0.0"
  pullPolicy: IfNotPresent

service:
  type: ClusterIP
  port: 80

ingress:
  enabled: false
  host: myapp.local
  tls: false

resources:
  requests:
    cpu: 100m
    memory: 128Mi
  limits:
    cpu: 500m
    memory: 512Mi

autoscaling:
  enabled: false
  minReplicas: 2
  maxReplicas: 10
  targetCPUUtilization: 80

env:
  LOG_LEVEL: info
  DATABASE_HOST: postgres.default.svc.cluster.local
```

### Templates: Go Template Language

Templates use Go's `text/template` syntax with Helm-specific functions:

```yaml
# templates/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: {{ include "mychart.fullname" . }}
  labels:
    {{- include "mychart.labels" . | nindent 4 }}
spec:
  {{- if not .Values.autoscaling.enabled }}
  replicas: {{ .Values.replicaCount }}
  {{- end }}
  selector:
    matchLabels:
      {{- include "mychart.selectorLabels" . | nindent 6 }}
  template:
    metadata:
      labels:
        {{- include "mychart.selectorLabels" . | nindent 8 }}
    spec:
      containers:
      - name: {{ .Chart.Name }}
        image: "{{ .Values.image.repository }}:{{ .Values.image.tag }}"
        imagePullPolicy: {{ .Values.image.pullPolicy }}
        ports:
        - containerPort: {{ .Values.service.port }}
        env:
        {{- range $key, $value := .Values.env }}
        - name: {{ $key }}
          value: {{ $value | quote }}
        {{- end }}
        resources:
          {{- toYaml .Values.resources | nindent 10 }}
```

### Template Helpers (_helpers.tpl)

```yaml
# templates/_helpers.tpl
{{/*
Expand the name of the chart.
*/}}
{{- define "mychart.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
*/}}
{{- define "mychart.fullname" -}}
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
Common labels
*/}}
{{- define "mychart.labels" -}}
helm.sh/chart: {{ .Chart.Name }}-{{ .Chart.Version }}
app.kubernetes.io/name: {{ include "mychart.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/*
Selector labels
*/}}
{{- define "mychart.selectorLabels" -}}
app.kubernetes.io/name: {{ include "mychart.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}
```

## The Production Way: Helm in Practice

### Installing Charts from Repositories

```bash
# Add a chart repository
helm repo add bitnami https://charts.bitnami.com/bitnami
helm repo add ingress-nginx https://kubernetes.github.io/ingress-nginx
helm repo add prometheus-community https://prometheus-community.github.io/helm-charts

# Update local repo cache
helm repo update

# Search for charts
helm search repo nginx
helm search hub redis    # Search Artifact Hub

# Install a chart
helm install my-redis bitnami/redis \
  --namespace redis \
  --create-namespace \
  --set auth.password=mypassword \
  --set master.persistence.size=10Gi

# Install with a values file
helm install my-nginx ingress-nginx/ingress-nginx \
  --namespace ingress-nginx \
  --create-namespace \
  -f nginx-values.yaml

# List installed releases
helm list -A

# Get release status
helm status my-redis -n redis

# Get the values used
helm get values my-redis -n redis
```

### Upgrading and Rollback

```bash
# Upgrade a release (change image tag)
helm upgrade my-redis bitnami/redis \
  --namespace redis \
  --set auth.password=mypassword \
  --set image.tag=7.2

# Upgrade with a values file
helm upgrade my-redis bitnami/redis \
  --namespace redis \
  -f redis-prod-values.yaml

# View release history
helm history my-redis -n redis
# REVISION  STATUS      DESCRIPTION
# 1         superseded  Install complete
# 2         superseded  Upgrade complete
# 3         deployed    Upgrade complete

# Rollback to a previous revision
helm rollback my-redis 2 -n redis

# Rollback to the immediately previous revision
helm rollback my-redis 0 -n redis
```

### Creating Your Own Chart

```bash
# Scaffold a new chart
helm create my-app

# The generated structure:
# my-app/
# ├── Chart.yaml
# ├── values.yaml
# ├── templates/
# │   ├── deployment.yaml
# │   ├── service.yaml
# │   ├── ingress.yaml
# │   ├── serviceaccount.yaml
# │   ├── hpa.yaml
# │   ├── NOTES.txt
# │   └── tests/
# └── charts/

# Edit values.yaml for your app
# Edit templates/ for your specific manifests

# Test the chart rendering (dry-run)
helm template my-release ./my-app -f values.yaml

# Lint the chart for errors
helm lint ./my-app

# Install your chart
helm install my-app ./my-app -n my-namespace --create-namespace

# Package the chart for distribution
helm package ./my-app
# Creates: my-app-0.1.0.tgz
```

### Chart.yaml: Metadata

```yaml
apiVersion: v2
name: my-app
description: A Helm chart for My Application
type: application        # or "library" for reusable charts
version: 0.1.0           # Chart version (SemVer)
appVersion: "1.2.3"      # Version of the app being deployed
maintainers:
- name: DevOps Team
  email: devops@mycompany.com
dependencies:
- name: postgresql
  version: "12.x.x"
  repository: "https://charts.bitnami.com/bitnami"
  condition: postgresql.enabled
- name: redis
  version: "17.x.x"
  repository: "https://charts.bitnami.com/bitnami"
  condition: redis.enabled
```

### Chart Dependencies

```bash
# Update dependency charts
helm dependency update ./my-app
# Downloads charts/ to my-app/charts/

# List dependencies
helm dependency list ./my-app
```

### Environment-Specific Values

```bash
# values.yaml          — base defaults
# values-dev.yaml      — dev overrides
# values-staging.yaml  — staging overrides
# values-prod.yaml     — production overrides

# Dev: 1 replica, latest image, no ingress
helm install my-app ./my-app -f values.yaml -f values-dev.yaml -n dev

# Prod: 5 replicas, pinned image, ingress, HPA
helm install my-app ./my-app -f values.yaml -f values-prod.yaml -n production
```

```yaml
# values-prod.yaml
replicaCount: 5
image:
  tag: "1.2.3"
ingress:
  enabled: true
  host: myapp.com
  tls: true
autoscaling:
  enabled: true
  minReplicas: 5
  maxReplicas: 20
resources:
  requests:
    cpu: 500m
    memory: 512Mi
  limits:
    cpu: "2"
    memory: 2Gi
```

### Hooks: Lifecycle Events

```yaml
# templates/pre-install-job.yaml
apiVersion: batch/v1
kind: Job
metadata:
  name: {{ include "mychart.fullname" . }}-db-migrate
  annotations:
    "helm.sh/hook": pre-install,pre-upgrade
    "helm.sh/hook-weight": "0"
    "helm.sh/hook-delete-policy": before-hook-creation
spec:
  template:
    spec:
      containers:
      - name: migrate
        image: "{{ .Values.image.repository }}:{{ .Values.image.tag }}"
        command: ["./migrate.sh"]
      restartPolicy: Never
```

Available hooks:
```
pre-install      — before resources are created
post-install     — after resources are created
pre-upgrade      — before upgrade
post-upgrade     — after upgrade
pre-delete       — before deletion
post-delete      — after deletion
pre-rollback     — before rollback
post-rollback    — after rollback
```

### Helm Diff and Dry-Run

```bash
# Render templates locally (no cluster needed)
helm template my-release ./my-app -f values.yaml

# Dry-run against the cluster (validates but doesn't apply)
helm upgrade my-release ./my-app -f values.yaml --dry-run

# Install helm-diff plugin for safe upgrades
helm plugin install https://github.com/databus23/helm-diff
helm diff upgrade my-release ./my-app -f values.yaml
```

## Hands-On Lab: Install and Create Charts

### Lab 1: Install NGINX Ingress with Helm

```bash
# Add the repository
helm repo add ingress-nginx https://kubernetes.github.io/ingress-nginx
helm repo update

# Install with default values
helm install nginx-ingress ingress-nginx/ingress-nginx \
  --namespace ingress-nginx \
  --create-namespace

# Check the release
helm status nginx-ingress -n ingress-nginx
helm list -A

# View the deployed resources
kubectl get pods -n ingress-nginx
kubectl get svc -n ingress-nginx

# Upgrade with custom values
helm upgrade nginx-ingress ingress-nginx/ingress-nginx \
  --namespace ingress-nginx \
  --set controller.replicaCount=2 \
  --set controller.metrics.enabled=true

# View history
helm history nginx-ingress -n ingress-nginx

# Rollback if needed
helm rollback nginx-ingress 1 -n ingress-nginx
```

### Lab 2: Install Redis with Custom Values

```bash
# Create a custom values file
cat <<EOF > redis-values.yaml
architecture: standalone
auth:
  enabled: true
  password: "my-secure-password"
master:
  persistence:
    size: 2Gi
  resources:
    requests:
      cpu: 100m
      memory: 128Mi
    limits:
      cpu: 250m
      memory: 256Mi
metrics:
  enabled: true
EOF

# Install Redis
helm install my-redis bitnami/redis \
  --namespace redis \
  --create-namespace \
  -f redis-values.yaml

# Connect to Redis
kubectl exec -it my-redis-master-0 -n redis -- redis-cli -a my-secure-password

# Upgrade (change password)
helm upgrade my-redis bitnami/redis \
  --namespace redis \
  -f redis-values.yaml \
  --set auth.password="new-secure-password"
```

### Lab 3: Create a Custom Chart from Scratch

```bash
# Create the chart
helm create webapp

# Edit Chart.yaml
cat <<EOF > webapp/Chart.yaml
apiVersion: v2
name: webapp
description: A simple web application
type: application
version: 0.1.0
appVersion: "1.0.0"
EOF

# Edit values.yaml
cat <<EOF > webapp/values.yaml
replicaCount: 2
image:
  repository: nginx
  tag: "alpine"
  pullPolicy: IfNotPresent
service:
  type: ClusterIP
  port: 80
ingress:
  enabled: false
  host: webapp.local
resources:
  requests:
    cpu: 50m
    memory: 64Mi
  limits:
    cpu: 100m
    memory: 128Mi
EOF

# Simplify the deployment template
cat <<EOF > webapp/templates/deployment.yaml
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
        - containerPort: {{ .Values.service.port }}
        resources:
          {{- toYaml .Values.resources | nindent 10 }}
EOF

# Simplify the service template
cat <<EOF > webapp/templates/service.yaml
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
    targetPort: {{ .Values.service.port }}
  selector:
    {{- include "webapp.selectorLabels" . | nindent 4 }}
EOF

# Lint the chart
helm lint webapp/

# Render templates locally
helm template my-webapp webapp/

# Install the chart
helm install my-webapp webapp/ -n webapp --create-namespace

# Verify
kubectl get pods -n webapp
helm list -n webapp

# Upgrade with new values
helm upgrade my-webapp webapp/ -n webapp --set replicaCount=3

# Package the chart
helm package webapp/
# Creates: webapp-0.1.0.tgz
```

### Lab 4: Cleanup

```bash
helm uninstall nginx-ingress -n ingress-nginx
helm uninstall my-redis -n redis
helm uninstall my-webapp -n webapp
kubectl delete namespace ingress-nginx redis webapp
```

## Limitation: Helm Is Powerful, But Templates Can Be Complex

Helm gives you packaging, versioning, rollback, and templating. But:
- Go templates are hard to read and debug
- Overriding deep values requires understanding the template structure
- For simple per-environment patches, Helm is overkill
- Some teams prefer "plain YAML + patches" over templating

**Next problem:** Is there a simpler way to manage environment-specific configurations without templates?

→ **Next module:** [36-kustomize](../36-kustomize/) — Configuration management without templating

## Checklist

- [ ] I understand what Helm is and why it exists (package manager for k8s)
- [ ] I can explain charts, values, releases, and repositories
- [ ] I can install a chart from a repository with custom values
- [ ] I can upgrade and rollback a Helm release
- [ ] I can create a custom chart from scratch
- [ ] I understand template syntax and helper functions
- [ ] I can manage chart dependencies
- [ ] I know how to use environment-specific values files
