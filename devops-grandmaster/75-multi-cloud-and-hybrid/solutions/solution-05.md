# Solution 05: Build a Cloud-Agnostic Application Platform

## Platform Architecture

The platform uses Kubernetes as the abstraction layer, with Helm for
packaging, External Secrets Operator for secrets portability, and a
unified CI/CD pipeline that deploys to any cloud.

```
                       +---------------------------+
                       |     GitHub Actions        |
                       |   (build + deploy)        |
                       +--------+---------+--------+
                                |         |
                    +-----------+         +-----------+
                    v                                  v
          +------------------+              +------------------+
          | AWS EKS          |              | GCP GKE          |
          |                  |              |                  |
          | +--Nginx Ingress |              | +--Nginx Ingress |
          | +--Ext. Secrets  |              | +--Ext. Secrets  |
          | +--Prometheus    |              | +--Prometheus    |
          | +--App Pods      |              | +--App Pods      |
          | +--GP3 Storage   |              | +--PD-SSD Stor   |
          +------------------+              +------------------+
                    |                                  |
          +---------+---------+            +----------+---------+
          | AWS Secrets Mgr   |            | GCP Secret Manager |
          | RDS PostgreSQL    |            | Cloud SQL          |
          | S3                |            | GCS                |
          +-------------------+            +--------------------+
```

## Helm Chart

### chart/Chart.yaml

```yaml
apiVersion: v2
name: cloud-agnostic-platform
description: A cloud-agnostic application platform
version: 1.0.0
appVersion: "1.0.0"
dependencies:
  - name: external-secrets
    version: "0.9.x"
    repository: "https://charts.external-secrets.io"
    condition: external-secrets.enabled
```

### chart/values.yaml (defaults)

```yaml
platform:
  cloudProvider: ""  # Must be set: aws, azure, gcp
  clusterName: ""
  region: ""

app:
  name: "datavault-app"
  image:
    repository: ""  # Set per cloud
    tag: "latest"
  replicas: 3
  port: 8080
  resources:
    requests:
      cpu: 250m
      memory: 256Mi
    limits:
      cpu: 1000m
      memory: 512Mi
  env:
    LOG_LEVEL: "info"
    DATABASE_SSL_MODE: "require"

ingress:
  enabled: true
  className: "nginx"
  host: "app.example.com"
  tls:
    enabled: true
    secretName: "app-tls"

storage:
  defaultClass: ""  # Set per cloud
  reclaimPolicy: Retain

monitoring:
  enabled: true
  prometheus:
    retention: 15d
    scrapeInterval: 15s
  grafana:
    enabled: true

secrets:
  enabled: true
  provider: "external-secrets"
  refreshInterval: "1h"
  secretStoreRef: "cloud-secret-store"

external-secrets:
  enabled: true
```

### chart/values-aws.yaml

```yaml
platform:
  cloudProvider: aws
  region: us-east-1

app:
  image:
    repository: 123456789.dkr.ecr.us-east-1.amazonaws.com/datavault-app

ingress:
  annotations:
    service.beta.kubernetes.io/aws-load-balancer-type: nlb
    service.beta.kubernetes.io/aws-load-balancer-scheme: internet-facing

storage:
  defaultClass: gp3-encrypted

secrets:
  store:
    type: aws
    region: us-east-1
    # Service account annotation for IRSA
    serviceAccount:
      annotations:
        eks.amazonaws.com/role-arn: arn:aws:iam::123456789:role/external-secrets
```

### chart/values-azure.yaml

```yaml
platform:
  cloudProvider: azure
  region: eastus

app:
  image:
    repository: datavaultregistry.azurecr.io/datavault-app

ingress:
  annotations:
    service.beta.kubernetes.io/azure-load-balancer-internal: "false"

storage:
  defaultClass: managed-premium-encrypted

secrets:
  store:
    type: azure
    vaultUrl: "https://datavault-kv.vault.azure.net/"
    # Managed identity for workload identity
    serviceAccount:
      annotations:
        azure.workload.identity/client-id: "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
```

### chart/values-gcp.yaml

```yaml
platform:
  cloudProvider: gcp
  region: us-central1

app:
  image:
    repository: gcr.io/my-project/datavault-app

ingress:
  annotations:
    # No special annotation needed for GCP

storage:
  defaultClass: premium-rwo

secrets:
  store:
    type: gcp
    projectId: "my-gcp-project"
    # Workload identity binding
    serviceAccount:
      annotations:
        iam.gke.io/gcp-service-account: "external-secrets@my-project.iam.gserviceaccount.com"
```

### chart/templates/namespace.yaml

```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: {{ .Values.app.name }}
  labels:
    app.kubernetes.io/managed-by: Helm
    cloud: {{ .Values.platform.cloudProvider }}
```

### chart/templates/secret-store.yaml

```yaml
{{- if .Values.secrets.enabled }}
{{- if eq .Values.platform.cloudProvider "aws" }}
apiVersion: external-secrets.io/v1beta1
kind: SecretStore
metadata:
  name: {{ .Values.secrets.secretStoreRef }}
  namespace: {{ .Values.app.name }}
spec:
  provider:
    aws:
      service: SecretsManager
      region: {{ .Values.platform.region }}
      auth:
        jwt:
          serviceAccountRef:
            name: external-secrets-sa
---
{{- else if eq .Values.platform.cloudProvider "gcp" }}
apiVersion: external-secrets.io/v1beta1
kind: SecretStore
metadata:
  name: {{ .Values.secrets.secretStoreRef }}
  namespace: {{ .Values.app.name }}
spec:
  provider:
    gcpsm:
      projectID: {{ .Values.secrets.store.projectId }}
      auth:
        workloadIdentity:
          clusterLocation: {{ .Values.platform.region }}
          clusterName: {{ .Values.platform.clusterName }}
          serviceAccountRef:
            name: external-secrets-sa
---
{{- else if eq .Values.platform.cloudProvider "azure" }}
apiVersion: external-secrets.io/v1beta1
kind: SecretStore
metadata:
  name: {{ .Values.secrets.secretStoreRef }}
  namespace: {{ .Values.app.name }}
spec:
  provider:
    azurekv:
      vaultUrl: {{ .Values.secrets.store.vaultUrl }}
      authType: WorkloadIdentity
      serviceAccountRef:
        name: external-secrets-sa
---
{{- end }}
{{- end }}
```

### chart/templates/external-secret.yaml

```yaml
{{- if .Values.secrets.enabled }}
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: app-secrets
  namespace: {{ .Values.app.name }}
spec:
  refreshInterval: {{ .Values.secrets.refreshInterval }}
  secretStoreRef:
    name: {{ .Values.secrets.secretStoreRef }}
    kind: SecretStore
  target:
    name: app-secrets
    creationPolicy: Owner
  data:
    - secretKey: DATABASE_URL
      remoteRef:
        key: {{ .Values.app.name }}/database-url
    - secretKey: API_KEY
      remoteRef:
        key: {{ .Values.app.name }}/api-key
    - secretKey: JWT_SECRET
      remoteRef:
        key: {{ .Values.app.name }}/jwt-secret
{{- end }}
```

### chart/templates/storage-class.yaml

```yaml
apiVersion: storage.k8s.io/v1
kind: StorageClass
metadata:
  name: {{ .Values.storage.defaultClass }}
  annotations:
    storageclass.kubernetes.io/is-default-class: "true"
provisioner: {{ include "platform.storageProvisioner" . }}
reclaimPolicy: {{ .Values.storage.reclaimPolicy }}
volumeBindingMode: WaitForFirstConsumer
parameters:
  {{- include "platform.storageParameters" . | nindent 2 }}
```

### chart/templates/_helpers.tpl

```yaml
{{- define "platform.storageProvisioner" -}}
{{- if eq .Values.platform.cloudProvider "aws" }}ebs.csi.aws.com
{{- else if eq .Values.platform.cloudProvider "azure" }}disk.csi.azure.com
{{- else if eq .Values.platform.cloudProvider "gcp" }}pd.csi.storage.gke.io
{{- end }}
{{- end }}

{{- define "platform.storageParameters" -}}
{{- if eq .Values.platform.cloudProvider "aws" }}
type: gp3
encrypted: "true"
{{- else if eq .Values.platform.cloudProvider "azure" }}
skuName: Premium_LRS
caching: ReadOnly
{{- else if eq .Values.platform.cloudProvider "gcp" }}
type: pd-ssd
replication-type: none
{{- end }}
{{- end }}
```

## Application

### app/Dockerfile

```dockerfile
FROM golang:1.21-alpine AS builder
WORKDIR /app
COPY go.mod go.sum ./
RUN go mod download
COPY . .
RUN CGO_ENABLED=0 go build -o /app/server .

FROM alpine:3.19
RUN apk --no-cache add ca-certificates
COPY --from=builder /app/server /server
EXPOSE 8080
HEALTHCHECK --interval=10s --timeout=3s \
  CMD wget -qO- http://localhost:8080/health || exit 1
ENTRYPOINT ["/server"]
```

### app/k8s/deployment.yaml

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: {{ .Values.app.name }}
  namespace: {{ .Values.app.name }}
spec:
  replicas: {{ .Values.app.replicas }}
  selector:
    matchLabels:
      app: {{ .Values.app.name }}
  template:
    metadata:
      labels:
        app: {{ .Values.app.name }}
    spec:
      containers:
        - name: {{ .Values.app.name }}
          image: "{{ .Values.app.image.repository }}:{{ .Values.app.image.tag }}"
          ports:
            - containerPort: {{ .Values.app.port }}
          envFrom:
            - secretRef:
                name: app-secrets
          env:
            {{- range $key, $value := .Values.app.env }}
            - name: {{ $key }}
              value: {{ $value | quote }}
            {{- end }}
          resources:
            {{- toYaml .Values.app.resources | nindent 12 }}
          livenessProbe:
            httpGet:
              path: /health
              port: {{ .Values.app.port }}
            initialDelaySeconds: 10
            periodSeconds: 15
          readinessProbe:
            httpGet:
              path: /ready
              port: {{ .Values.app.port }}
            initialDelaySeconds: 5
            periodSeconds: 10
```

### app/k8s/hpa.yaml

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: {{ .Values.app.name }}
  namespace: {{ .Values.app.name }}
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: {{ .Values.app.name }}
  minReplicas: {{ .Values.app.replicas }}
  maxReplicas: 50
  metrics:
    - type: Resource
      resource:
        name: cpu
        target:
          type: Utilization
          averageUtilization: 70
    - type: Resource
      resource:
        name: memory
        target:
          type: Utilization
          averageUtilization: 80
```

## CI/CD Pipeline

### pipeline/deploy.yml (GitHub Actions)

```yaml
name: Deploy to Cloud

on:
  workflow_dispatch:
    inputs:
      cloud_provider:
        description: 'Target cloud (aws, azure, gcp)'
        required: true
        type: choice
        options: [aws, azure, gcp]
      environment:
        description: 'Target environment'
        required: true
        type: choice
        options: [dev, staging, prod]
      image_tag:
        description: 'Image tag to deploy'
        required: true

env:
  IMAGE_NAME: datavault-app

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Build image
        run: |
          docker build -t ${{ env.IMAGE_NAME }}:${{ inputs.image_tag }} ./app

      - name: Push to AWS ECR
        if: inputs.cloud_provider == 'aws'
        uses: aws-actions/amazon-ecr-login@v2
        run: |
          aws ecr get-login-password | docker login --username AWS --password-stdin $ECR_REGISTRY
          docker tag ${{ env.IMAGE_NAME }}:${{ inputs.image_tag }} $ECR_REGISTRY/${{ env.IMAGE_NAME }}:${{ inputs.image_tag }}
          docker push $ECR_REGISTRY/${{ env.IMAGE_NAME }}:${{ inputs.image_tag }}

      - name: Push to Azure ACR
        if: inputs.cloud_provider == 'azure'
        run: |
          az acr login --name datavaultregistry
          docker tag ${{ env.IMAGE_NAME }}:${{ inputs.image_tag }} \
            datavaultregistry.azurecr.io/${{ env.IMAGE_NAME }}:${{ inputs.image_tag }}
          docker push datavaultregistry.azurecr.io/${{ env.IMAGE_NAME }}:${{ inputs.image_tag }}

      - name: Push to GCP GCR
        if: inputs.cloud_provider == 'gcp'
        uses: google-github-actions/auth@v2
        with:
          credentials_json: ${{ secrets.GCP_SA_KEY }}
        run: |
          gcloud auth configure-docker
          docker tag ${{ env.IMAGE_NAME }}:${{ inputs.image_tag }} \
            gcr.io/${{ secrets.GCP_PROJECT }}/${{ env.IMAGE_NAME }}:${{ inputs.image_tag }}
          docker push gcr.io/${{ secrets.GCP_PROJECT }}/${{ env.IMAGE_NAME }}:${{ inputs.image_tag }}

  deploy:
    needs: build
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Configure AWS credentials
        if: inputs.cloud_provider == 'aws'
        uses: aws-actions/configure-aws-credentials@v4
        with:
          role-to-assume: ${{ secrets.AWS_DEPLOY_ROLE_ARN }}
          aws-region: us-east-1

      - name: Configure Azure credentials
        if: inputs.cloud_provider == 'azure'
        uses: azure/login@v1
        with:
          creds: ${{ secrets.AZURE_CREDENTIALS }}

      - name: Configure GCP credentials
        if: inputs.cloud_provider == 'gcp'
        uses: google-github-actions/auth@v2
        with:
          credentials_json: ${{ secrets.GCP_SA_KEY }}

      - name: Set kubeconfig
        run: |
          if [ "${{ inputs.cloud_provider }}" == "aws" ]; then
            aws eks update-kubeconfig --name datavault-cluster --region us-east-1
          elif [ "${{ inputs.cloud_provider }}" == "azure" ]; then
            az aks get-credentials --resource-group datavault-rg --name datavault-cluster
          elif [ "${{ inputs.cloud_provider }}" == "gcp" ]; then
            gcloud container clusters get-credentials datavault-cluster --region us-central1
          fi

      - name: Deploy with Helm
        run: |
          helm upgrade --install ${{ env.IMAGE_NAME }} ./chart \
            -f ./chart/values-${{ inputs.cloud_provider }}.yaml \
            --set app.image.tag=${{ inputs.image_tag }} \
            --namespace ${{ env.IMAGE_NAME }} \
            --create-namespace \
            --wait \
            --timeout=300s

      - name: Smoke test
        run: |
          kubectl wait --for=condition=available deployment/${{ env.IMAGE_NAME }} \
            -n ${{ env.IMAGE_NAME }} --timeout=120s
          # Port-forward and test
          kubectl port-forward svc/${{ env.IMAGE_NAME }} 8080:8080 \
            -n ${{ env.IMAGE_NAME }} &
          sleep 5
          HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:8080/health)
          if [ "$HTTP_CODE" != "200" ]; then
            echo "Smoke test failed: HTTP $HTTP_CODE"
            exit 1
          fi
          echo "Smoke test passed."

  deploy-multi:
    # Optional: deploy to all clouds in parallel
    if: inputs.cloud_provider == 'all'
    strategy:
      matrix:
        cloud: [aws, azure, gcp]
    uses: ./.github/workflows/deploy-single.yml
    with:
      cloud_provider: ${{ matrix.cloud }}
      environment: ${{ inputs.environment }}
      image_tag: ${{ inputs.image_tag }}
```

## Validation Script

```bash
#!/bin/bash
# validate-portability.sh

set -euo pipefail

echo "=== Validating Helm chart portability ==="

PROVIDERS=("aws" "azure" "gcp")
ERRORS=0

for provider in "${PROVIDERS[@]}"; do
  echo ""
  echo "--- Testing ${provider} ---"

  # 1. Template rendering
  echo -n "  Template rendering: "
  if helm template ./chart -f ./chart/values-${provider}.yaml > /dev/null 2>&1; then
    echo "OK"
  else
    echo "FAILED"
    helm template ./chart -f ./chart/values-${provider}.yaml 2>&1 | head -20
    ERRORS=$((ERRORS + 1))
  fi

  # 2. No cross-cloud resource leaks
  echo -n "  No cross-cloud leaks: "
  RENDERED=$(helm template ./chart -f ./chart/values-${provider}.yaml 2>/dev/null)
  LEAKS=0

  if [ "$provider" != "aws" ]; then
    if echo "$RENDERED" | grep -q "arn:aws"; then
      echo "FAILED (found AWS ARN in ${provider} output)"
      LEAKS=$((LEAKS + 1))
    fi
  fi
  if [ "$provider" != "azure" ]; then
    if echo "$RENDERED" | grep -q "azurecr.io"; then
      echo "FAILED (found Azure registry in ${provider} output)"
      LEAKS=$((LEAKS + 1))
    fi
  fi
  if [ "$provider" != "gcp" ]; then
    if echo "$RENDERED" | grep -q "gcr.io"; then
      echo "FAILED (found GCR in ${provider} output)"
      LEAKS=$((LEAKS + 1))
    fi
  fi
  if [ "$LEAKS" -eq 0 ]; then
    echo "OK"
  else
    ERRORS=$((ERRORS + LEAKS))
  fi

  # 3. Correct provisioner
  echo -n "  Storage provisioner: "
  case $provider in
    aws)   EXPECTED="ebs.csi.aws.com" ;;
    azure) EXPECTED="disk.csi.azure.com" ;;
    gcp)   EXPECTED="pd.csi.storage.gke.io" ;;
  esac
  if echo "$RENDERED" | grep -q "$EXPECTED"; then
    echo "OK ($EXPECTED)"
  else
    echo "FAILED (expected $EXPECTED)"
    ERRORS=$((ERRORS + 1))
  fi

  # 4. Correct secret store type
  echo -n "  Secret store: "
  case $provider in
    aws)   EXPECTED="SecretsManager" ;;
    azure) EXPECTED="azurekv" ;;
    gcp)   EXPECTED="gcpsm" ;;
  esac
  if echo "$RENDERED" | grep -q "$EXPECTED"; then
    echo "OK ($EXPECTED)"
  else
    echo "FAILED (expected $EXPECTED)"
    ERRORS=$((ERRORS + 1))
  fi
done

echo ""
if [ "$ERRORS" -eq 0 ]; then
  echo "=== All validations passed ==="
  exit 0
else
  echo "=== ${ERRORS} validation(s) failed ==="
  exit 1
fi
```

## Trade-Offs: Cloud-Agnostic vs Native

| Aspect | Cloud-Agnostic | Cloud-Native |
|--------|---------------|-------------|
| Portability | High -- any K8s cluster | Low -- tied to one provider |
| Performance | Good -- misses optimization | Best -- uses native features |
| Feature set | Common subset only | Full provider feature set |
| Operational cost | Higher (abstraction maintenance) | Lower (single expertise) |
| Vendor leverage | High -- can switch | Low -- switching is expensive |
| Debugging | Harder (extra layers) | Easier (direct access) |
| Team skill req | Broad (all clouds + K8s) | Deep (one cloud) |

## Adding a Fourth Cloud Provider

To support a new cloud (e.g., Oracle Cloud Infrastructure):

1. Create `chart/values-oci.yaml` with OCI-specific configuration.
2. Add OCI image registry configuration to the CI/CD pipeline.
3. Create an OCI-specific SecretStore template (if External Secrets
   supports OCI Vault, or use a generic Kubernetes secret).
4. Add OCI StorageClass parameters to `_helpers.tpl`.
5. Update the validation script to include `oci` in the providers list.
6. Set up a Kubernetes cluster on OCI (OKE).
7. Test the full deployment pipeline end-to-end.

Estimated effort: 1-2 days for an engineer familiar with the platform.
