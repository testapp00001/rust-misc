# Exercise 05: Build a Cloud-Agnostic Application Platform

**Type:** Integration
**Difficulty:** Expert
**Estimated Time:** 6-8 hours

## Objective

Design and implement a cloud-agnostic application platform that can run the
same application workloads on AWS, Azure, or GCP without code changes. The
platform uses Kubernetes as the abstraction layer, with a unified CI/CD
pipeline, cloud-agnostic service mesh, and portable observability stack.

## Background

The highest level of multi-cloud maturity is a platform where the application
is truly portable. Kubernetes provides compute portability, but a complete
platform must also address:

- **Service discovery and networking** across clouds
- **Secrets management** that works regardless of provider
- **Storage abstraction** for persistent workloads
- **Observability** (metrics, logs, traces) aggregated from all clouds
- **CI/CD** that deploys to any target cluster

This exercise builds such a platform from scratch.

## Architecture Overview

```
                    +--------------------------+
                    |    CI/CD Pipeline        |
                    |  (GitHub Actions)        |
                    +------+----------+--------+
                           |          |
                    +------v--+  +----v------+
                    | AWS EKS |  | GCP GKE   |
                    | Cluster |  | Cluster   |
                    +---------+  +-----------+
                         |            |
                    +----v------------v----+
                    |   Service Mesh       |
                    |   (Linkerd/Istio)    |
                    +---------------------+
                         |
                    +----v----------------+
                    |  Observability      |
                    |  (Grafana/Prometheus)|
                    +---------------------+
```

## Instructions

### Step 1: Create a Helm Chart for the Application Platform (120 minutes)

Create a Helm chart that packages all platform components. The chart must
parameterize cloud-specific values so the same chart installs on any cloud.

```
exercise-05/
  chart/
    Chart.yaml
    values.yaml
    values-aws.yaml
    values-azure.yaml
    values-gcp.yaml
    templates/
      namespace.yaml
      service-mesh.yaml
      ingress.yaml
      secrets-external.yaml
      monitoring.yaml
      storage-class.yaml
  app/
    Dockerfile
    main.go (or main.py)
    k8s/
      deployment.yaml
      service.yaml
      hpa.yaml
  pipeline/
    deploy.yml
  README.md
```

**values.yaml** should define all cloud-agnostic defaults. Provider-specific
value files override only the things that differ:

```yaml
platform:
  cloudProvider: "aws"  # aws, azure, gcp
  clusterName: "app-cluster"
  region: "us-east-1"

ingress:
  enabled: true
  className: "nginx"
  annotations: {}
  host: "app.example.com"

storage:
  defaultClass: "gp3"  # Overridden per cloud
  reclaimPolicy: Retain

monitoring:
  enabled: true
  prometheus:
    retention: 15d
  grafana:
    enabled: true

secrets:
  provider: "external-secrets"  # Uses External Secrets Operator
  store: {}  # Overridden per cloud (AWS SSM, Azure Key Vault, GCP Secret Manager)
```

### Step 2: Implement Cloud-Agnostic Secrets Management (60 minutes)

Use the External Secrets Operator to abstract secrets management. Create a
Helm template that configures the SecretStore based on the target cloud:

**AWS**: Uses AWS Systems Manager Parameter Store or Secrets Manager
**Azure**: Uses Azure Key Vault
**GCP**: Uses Google Secret Manager

Create templates for:

1. A `SecretStore` resource that configures the cloud-specific backend
2. An `ExternalSecret` resource that syncs application secrets from the
   cloud store into Kubernetes Secrets
3. A sample application Deployment that mounts the synced secrets

The application should reference secrets by logical name only -- the cloud
provider mapping is handled entirely by the platform configuration.

### Step 3: Implement Cloud-Agnostic Storage (45 minutes)

Create StorageClass templates for each cloud:

```yaml
# AWS
apiVersion: storage.k8s.io/v1
kind: StorageClass
metadata:
  name: {{ .Values.storage.defaultClass }}
provisioner: ebs.csi.aws.com
parameters:
  type: gp3
  encrypted: "true"
reclaimPolicy: {{ .Values.storage.reclaimPolicy }}
```

Create equivalent StorageClass resources for Azure (Azure Disk CSI) and GCP
(Persistent Disk CSI). Use Helm conditionals to include only the relevant
provisioner for the target cloud.

### Step 4: Create a Unified CI/CD Pipeline (90 minutes)

Create a GitHub Actions workflow that:

1. Builds a container image from the application source
2. Pushes it to a container registry (ECR, ACR, or GCR based on target)
3. Deploys to the target Kubernetes cluster using Helm
4. Runs smoke tests against the deployed application
5. Supports deploying to multiple clouds in parallel (matrix strategy)

The workflow should accept inputs for:
- Target cloud provider
- Target region
- Environment (dev, staging, prod)
- Image tag

Key design decisions to document:
- How do you handle cloud-specific authentication in the pipeline?
- How do you manage Helm values per environment per cloud?
- How do you handle database migrations that may differ per cloud?

### Step 5: Implement Observability (60 minutes)

Create templates for a portable observability stack:

1. **Prometheus**: For metrics collection. Configure cloud-specific
   service monitors:
   - AWS: Scrape CloudWatch metrics via a CloudWatch exporter
   - Azure: Scrape Azure Monitor metrics via an exporter
   - GCP: Scrape Cloud Monitoring metrics via an exporter

2. **Loki**: For log aggregation. Configure log shipping from each cloud's
   native logging:
   - AWS: Ship CloudWatch Logs to Loki
   - Azure: Ship Azure Monitor Logs to Loki
   - GCP: Ship Cloud Logging to Loki

3. **Grafana**: Unified dashboards that work regardless of which cloud the
   data comes from. Create a dashboard JSON that shows:
   - Request rate and latency
   - Error rate
   - Resource utilization (CPU, memory, disk)
   - Cloud-specific cost metrics

### Step 6: Validate Portability (30 minutes)

Create a validation script that:

1. Checks all Helm templates render correctly for each cloud provider:
   ```bash
   for provider in aws azure gcp; do
     helm template ./chart -f ./chart/values-${provider}.yaml > /dev/null
     echo "${provider}: OK"
   done
   ```

2. Verifies no cloud-specific resources leak into the wrong provider's
   output (e.g., no AWS ARNs in the Azure template).

3. Validates that the application image runs identically on all three
   clusters using a kind/minikube local test.

## Success Criteria

- [ ] The Helm chart installs successfully on at least two different cloud
      providers (or local clusters simulating them).
- [ ] The same application container image runs on all target clouds without
      modification.
- [ ] Secrets are managed through External Secrets Operator with
      cloud-specific backends configured via Helm values only.
- [ ] StorageClasses are correctly provisioned for each cloud.
- [ ] The CI/CD pipeline can deploy to any target cloud with a single
      parameter change.
- [ ] Observability dashboards show data regardless of which cloud is
      running the workload.
- [ ] You can articulate the performance and feature trade-offs of this
      cloud-agnostic approach vs using native cloud services directly.
- [ ] The README documents how to add support for a fourth cloud provider.

## Hints

<details>
<summary>Hint 1: Handling cloud-specific ingress</summary>

Different clouds have different ingress controllers and load balancer
implementations. Use the NGINX Ingress Controller on all clouds as it is
truly portable. For the LoadBalancer service type, each cloud automatically
provisions the correct type of load balancer. Annotate the Service
differently per cloud:

- AWS: `service.beta.kubernetes.io/aws-load-balancer-type: nlb`
- Azure: `service.beta.kubernetes.io/azure-load-balancer-internal: "false"`
- GCP: No special annotation needed (default is external)

</details>

<details>
<summary>Hint 2: External Secrets Operator setup</summary>

Install External Secrets Operator via Helm:

```bash
helm repo add external-secrets https://charts.external-secrets.io
helm install external-secrets external-secrets/external-secrets
```

Then create a SecretStore per cloud. The key insight is that the
ExternalSecret resource references a SecretStore by name -- you create
different SecretStores per cloud and the ExternalSecret always references
the same logical name. The mapping is in the SecretStore configuration.

</details>

<details>
<summary>Hint 3: Multi-cloud registry strategy</summary>

For the CI/CD pipeline, push the same image to all three registries
(ECR, ACR, GCR) so it is available regardless of which cloud is active.
Use the same image digest (not tag) across all registries to ensure
byte-for-byte identical images. In the pipeline:

```bash
DIGEST=$(docker push aws-ecr/image:tag --format '{{.RepoDigests}}')
docker tag aws-ecr/image@${DIGEST} acr.azurecr.io/image:tag
docker tag aws-ecr/image@${DIGEST} gcr.io/project/image:tag
docker push acr.azurecr.io/image:tag
docker push gcr.io/project/image:tag
```

</details>
