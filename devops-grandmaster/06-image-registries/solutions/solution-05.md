# Solution 05: Design a Registry Strategy for a Multi-Team Organization

## Part A: Registry Design Document

### 1. Registry Selection

**Primary registry: Amazon ECR (Elastic Container Registry)**

Justification:
- NovaCorp already runs workloads on AWS (EKS), so ECR integrates natively
  with IAM, VPC, and EKS. Pulling from ECR within the same region is free.
- ECR supports image scanning on push (via Amazon Inspector), lifecycle
  policies for automated cleanup, and cross-region replication.
- For 45GB of storage (estimated), ECR costs approximately $4.50/month.

**Secondary registry: Harbor (self-hosted, on-premises)**

Justification:
- NovaCorp has on-premises workloads that cannot depend on internet access
  to pull images.
- Harbor acts as a pull-through cache for ECR, reducing bandwidth costs
  and providing offline capability.
- Harbor is open-source, so there is no per-GB licensing cost. The cost is
  the server that runs it.
- Harbor provides built-in Trivy scanning, RBAC, and project-based access
  control.

**Why not other options:**
- Docker Hub: Rate limits (200 pulls/6 hours) are too restrictive for CI/CD.
  No IAM integration.
- GCR/Artifact Registry: NovaCorp is not a GCP shop.
- ACR: NovaCorp is not an Azure shop.
- GHCR: Good for open-source, but lacks the enterprise features (lifecycle
  policies, cross-region replication) that NovaCorp needs.

### 2. Repository Naming Convention

```
<org>/<team>/<app>:<tag>
```

Examples:

```
nova/platform/base-node20:20.11        # Shared base image
nova/platform/base-python312:3.12.1    # Shared base image
nova/backend/user-service:abc1234      # Microservice
nova/backend/order-service:1.2.3       # Released microservice
nova/frontend/web-app:main-def5678     # Frontend app
nova/frontend/bff:1.0.0                # Backend for frontend
nova/data/ml-serving:v2.0.0            # ML model server
nova/data/etl-pipeline:main-ghi9012    # Data pipeline
```

**Hybrid cloud handling:**

All images are built and pushed to ECR (us-east-1). Harbor on-premises is
configured as a pull-through cache that mirrors the ECR repositories. On-
premises workloads pull from Harbor; AWS workloads pull from ECR directly.

```
Build Server -> ECR (us-east-1) -> Harbor (on-premises mirror)
                     |                       |
                 AWS workloads        On-premises workloads
```

### 3. Access Control Model

**Team-level isolation:**

| Team | ECR Repositories | Push | Pull | Delete |
|------|-----------------|------|------|--------|
| Platform | `nova/platform/*` | Yes | Yes | Yes |
| Backend | `nova/backend/*` | Yes | Yes | Yes |
| Frontend | `nova/frontend/*` | Yes | Yes | Yes |
| Data | `nova/data/*` | Yes | Yes | Yes |
| All teams | `nova/platform/*` | No | Yes | No |

**Key rules:**
- Each team can push only to their own namespace.
- All teams can pull shared base images from `nova/platform/*`.
- Only the Platform team can push to `nova/platform/*`.
- CI/CD service accounts (robot accounts) have push access only to their
  team's namespace.

**Implementation (ECR IAM policies):**

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Sid": "AllowPullAllRepos",
      "Effect": "Allow",
      "Action": [
        "ecr:BatchGetImage",
        "ecr:GetDownloadUrlForLayer",
        "ecr:BatchCheckLayerAvailability"
      ],
      "Resource": "arn:aws:ecr:us-east-1:123456789012:repository/nova/*"
    },
    {
      "Sid": "AllowPushToBackendOnly",
      "Effect": "Allow",
      "Action": [
        "ecr:PutImage",
        "ecr:InitiateLayerUpload",
        "ecr:UploadLayerPart",
        "ecr:CompleteLayerUpload"
      ],
      "Resource": "arn:aws:ecr:us-east-1:123456789012:repository/nova/backend/*"
    }
  ]
}
```

Create separate IAM roles for each team's CI/CD pipeline, each with push
access limited to their namespace.

**Harbor project structure:**

```
Projects:
  nova-platform:   (Platform team: admin, others: read-only)
  nova-backend:    (Backend team: admin, Platform: admin)
  nova-frontend:   (Frontend team: admin, Platform: admin)
  nova-data:       (Data team: admin, Platform: admin)
```

Each project has a robot account for CI/CD:

```bash
# Create robot account for backend team CI/CD
# Harbor UI: Projects > nova-backend > Robot Accounts > New Robot Account
# Permissions: Push, Pull (scoped to nova-backend project only)
```

### 4. Tag Strategy

| Context | Tag Format | Example | Auto-Deploy? |
|---------|-----------|---------|--------------|
| Feature branch | `<branch>-<sha>` | `feature-auth-abc1234` | No |
| Main branch (staging) | `main-<sha>` + `staging` | `main-def5678`, `staging` | Auto-deploy to staging |
| Release | `<semver>` + `<major.minor>` + `<major>` | `1.2.3`, `1.2`, `1` | Manual promotion to production |
| Production | `production` (rolling) | `production` | Only via promotion script |

**Semantic versioning rules:**
- Release tags are created only from git tags (e.g., `v1.2.3`).
- The leading `v` is stripped for the image tag (`v1.2.3` -> `1.2.3`).
- When a new patch is released (`1.2.4`), the `1.2` and `1` aliases are
  updated to point to `1.2.4`.

**`latest` tag policy:** We do not use `latest`. The `staging` tag serves
the "give me the latest from main" role for staging environments only.

### 5. Vulnerability Scanning Policy

**When scanning happens:**
1. **Build time:** Trivy runs in CI before pushing. If CRITICAL or HIGH
   vulnerabilities are found, the push is blocked. This prevents vulnerable
   images from ever entering the registry.
2. **Push time:** ECR scan-on-push is enabled for all repositories. This
   provides a second scan using Amazon Inspector, which may catch
   vulnerabilities that Trivy missed (different vulnerability databases).

**Severity thresholds:**

| Severity | Build Time | Push Time | Deployment |
|----------|-----------|-----------|------------|
| CRITICAL | Block push | Alert | Block deployment |
| HIGH | Block push | Alert | Block deployment |
| MEDIUM | Warn (allow push) | Log | Allow deployment |
| LOW | Ignore | Log | Allow deployment |

**Handling false positives:**
- Maintain a `.trivyignore` file in each repository with documented
  exceptions. Each entry must include the CVE ID, the reason for the
  exception, and an expiration date.
- The Platform team reviews `.trivyignore` files monthly.
- Expired exceptions cause the build to fail, forcing re-evaluation.

```
# .trivyignore
# CVE-2023-12345 - Not exploitable in our usage (no network access to affected endpoint)
# Expires: 2024-06-01
CVE-2023-12345
```

### 6. Image Retention Policy

**ECR lifecycle rules:**

```json
{
  "rules": [
    {
      "rulePriority": 1,
      "description": "Keep last 20 images with any tag",
      "selection": {
        "tagStatus": "any",
        "countType": "imageCountMoreThan",
        "countNumber": 20
      },
      "action": { "type": "expire" }
    },
    {
      "rulePriority": 2,
      "description": "Keep release tags (v*) for 90 days",
      "selection": {
        "tagStatus": "tagged",
        "tagPrefixList": ["v"],
        "countType": "sinceImagePushed",
        "countUnit": "days",
        "countNumber": 90
      },
      "action": { "type": "expire" }
    },
    {
      "rulePriority": 3,
      "description": "Delete untagged images after 7 days",
      "selection": {
        "tagStatus": "untagged",
        "countType": "sinceImagePushed",
        "countUnit": "days",
        "countNumber": 7
      },
      "action": { "type": "expire" }
    }
  ]
}
```

**Harbor retention rules (per project):**
- Keep the 20 most recently pushed images.
- Keep images pushed within the last 90 days.
- Always keep images tagged with semver pattern (`*.*.*`).
- Delete everything else.

**Exempt images:**
- Images tagged with semver patterns (`1.2.3`) are protected by tag
  immutability rules in Harbor.
- The Platform team's base images (`nova/platform/*`) are exempt from
  automatic cleanup and managed manually.

### 7. Cost Optimization Strategy

**Storage estimate:**

```
Services:           ~22 (12 backend + 4 frontend + 1 BFF + 4 data + 1 base)
Tags per service:   ~15 (recent commits + releases)
Average image size: ~150MB (with multi-stage builds)
Total storage:      22 x 15 x 150MB = ~50GB

ECR storage cost:   50GB x $0.10/GB = $5.00/month
```

**Bandwidth estimate:**

```
CI/CD pulls per day:     ~500 (builds + deploys)
Average image size:      150MB
Daily bandwidth:         500 x 150MB = 75GB/day
Monthly bandwidth:       75GB x 30 = 2,250GB

Same-region ECR pulls:   $0.00 (free within AWS)
Cross-region ECR pulls:  $0.02/GB x 2,250GB = $45/month
On-premises (via mirror): $0.00 (Harbor caches locally)
```

**Total estimated monthly cost:**

```
ECR storage:              $5.00
ECR bandwidth (same region): $0.00
Harbor server (t3.medium): ~$30.00
Amazon Inspector scans:   ~$0.09 x 500 scans = $45.00
─────────────────────────────────────────
Total:                    ~$80/month (well under $500 budget)
```

**Cost optimization tactics:**
1. Multi-stage builds reduce image size from ~500MB to ~150MB (70%
   reduction in storage and bandwidth).
2. Shared base images mean common layers are stored once.
3. Harbor pull-through cache eliminates on-premises bandwidth costs.
4. Lifecycle policies prevent unbounded storage growth.
5. CI/CD caches base image layers to avoid redundant pulls.

---

## Part B: Access Control Implementation

### ECR Repository Creation

```bash
#!/usr/bin/env bash
set -euo pipefail

AWS_ACCOUNT="123456789012"
REGION="us-east-1"

REPOS=(
  "nova/platform/base-node20"
  "nova/platform/base-python312"
  "nova/backend/user-service"
  "nova/backend/order-service"
  "nova/backend/payment-service"
  # ... other repos
  "nova/frontend/web-app"
  "nova/frontend/bff"
  "nova/data/ml-serving"
  "nova/data/etl-pipeline"
)

for REPO in "${REPOS[@]}"; do
  echo "Creating repository: ${REPO}"
  aws ecr create-repository \
    --repository-name "${REPO}" \
    --region "${REGION}" \
    --image-scanning-configuration scanOnPush=true \
    --encryption-configuration encryptionType=AES256 \
    --image-tag-mutability IMMUTABLE \
    2>/dev/null || echo "  Already exists, skipping"

  # Apply lifecycle policy
  aws ecr put-lifecycle-policy \
    --repository-name "${REPO}" \
    --region "${REGION}" \
    --lifecycle-policy-text file://lifecycle-policy.json \
    2>/dev/null
done

echo "Done."
```

### Harbor Setup

```bash
# Harbor projects are created via the UI or API
# Example: create the nova-backend project

curl -X POST "https://harbor.novacorp.com/api/v2.0/projects" \
  -H "Authorization: Basic $(echo -n admin:password | base64)" \
  -H "Content-Type: application/json" \
  -d '{
    "project_name": "nova-backend",
    "public": false,
    "storage_limit": -1,
    "auto_scan": true,
    "severity": "high"
  }'
```

---

## Part C: CI/CD Pipeline

```yaml
# .github/workflows/build-and-push.yml
name: Build, Scan, and Push

on:
  push:
    branches: [main, "feature/**"]
    tags: ["v*"]

permissions:
  id-token: write    # For OIDC auth to AWS
  contents: read
  security-events: write  # For uploading SARIF scan results

env:
  ECR_REGISTRY: 123456789012.dkr.ecr.us-east-1.amazonaws.com
  IMAGE_NAME: nova/backend/user-service

jobs:
  # ─── Job 1: Build ──────────────────────────────────────────────────
  build:
    runs-on: ubuntu-latest
    outputs:
      image_tag: ${{ steps.meta.outputs.tags }}
      git_sha: ${{ steps.git.outputs.sha }}
    steps:
      - uses: actions/checkout@v4

      - name: Detect Git info
        id: git
        run: |
          echo "sha=$(git rev-parse --short HEAD)" >> "$GITHUB_OUTPUT"
          echo "branch=$(git rev-parse --abbrev-ref HEAD)" >> "$GITHUB_OUTPUT"
          GIT_TAG=$(git describe --tags --exact-match 2>/dev/null || echo "")
          echo "tag=${GIT_TAG}" >> "$GITHUB_OUTPUT"

      - name: Determine image tags
        id: meta
        run: |
          SHA="${{ steps.git.outputs.sha }}"
          BRANCH="${{ steps.git.outputs.branch }}"
          GIT_TAG="${{ steps.git.outputs.tag }}"
          IMAGE="${{ env.ECR_REGISTRY }}/${{ env.IMAGE_NAME }}"
          TAGS=""

          if [ -n "$GIT_TAG" ]; then
            VERSION="${GIT_TAG#v}"
            MAJOR=$(echo "$VERSION" | cut -d. -f1)
            MINOR=$(echo "$VERSION" | cut -d. -f1-2)
            TAGS="${IMAGE}:${SHA},${IMAGE}:${VERSION},${IMAGE}:${MINOR},${IMAGE}:${MAJOR}"
          elif [ "$BRANCH" = "main" ]; then
            TAGS="${IMAGE}:main-${SHA},${IMAGE}:staging"
          else
            SAFE_BRANCH=$(echo "$BRANCH" | tr '/_' '-' | tr '[:upper:]' '[:lower:]')
            TAGS="${IMAGE}:${SAFE_BRANCH}-${SHA}"
          fi

          echo "tags=${TAGS}" >> "$GITHUB_OUTPUT"

      - name: Configure AWS credentials
        uses: aws-actions/configure-aws-credentials@v4
        with:
          role-to-assume: arn:aws:iam::123456789012:role/ci-user-service
          aws-region: us-east-1

      - name: Login to ECR
        uses: aws-actions/amazon-ecr-login@v2

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3

      - name: Build and push
        uses: docker/build-push-action@v5
        with:
          context: .
          push: true
          tags: ${{ steps.meta.outputs.tags }}
          build-args: |
            APP_VERSION=${{ steps.git.outputs.sha }}
            GIT_SHA=${{ steps.git.outputs.sha }}
          cache-from: type=gha
          cache-to: type=gha,mode=max

  # ─── Job 2: Scan ───────────────────────────────────────────────────
  scan:
    needs: build
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Configure AWS credentials
        uses: aws-actions/configure-aws-credentials@v4
        with:
          role-to-assume: arn:aws:iam::123456789012:role/ci-user-service
          aws-region: us-east-1

      - name: Login to ECR
        uses: aws-actions/amazon-ecr-login@v2

      - name: Run Trivy vulnerability scanner
        uses: aquasecurity/trivy-action@master
        with:
          image-ref: "${{ env.ECR_REGISTRY }}/${{ env.IMAGE_NAME }}:${{ needs.build.outputs.git_sha }}"
          format: "sarif"
          output: "trivy-results.sarif"
          severity: "CRITICAL,HIGH"
          exit-code: "1"  # Fail the build on CRITICAL or HIGH vulns

      - name: Upload Trivy results to GitHub Security
        if: always()
        uses: github/codeql-action/upload-sarif@v2
        with:
          sarif_file: "trivy-results.sarif"

  # ─── Job 3: Deploy Staging ─────────────────────────────────────────
  deploy-staging:
    needs: [build, scan]
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'
    environment: staging
    steps:
      - uses: actions/checkout@v4

      - name: Configure AWS credentials
        uses: aws-actions/configure-aws-credentials@v4
        with:
          role-to-assume: arn:aws:iam::123456789012:role/deploy-staging
          aws-region: us-east-1

      - name: Deploy to staging EKS
        run: |
          aws eks update-kubeconfig --name staging-cluster --region us-east-1
          kubectl set image deployment/user-service \
            user-service="${{ env.ECR_REGISTRY }}/${{ env.IMAGE_NAME }}:${{ needs.build.outputs.git_sha }}" \
            -n staging
          kubectl rollout status deployment/user-service -n staging --timeout=300s

  # ─── Job 4: Deploy Production ──────────────────────────────────────
  deploy-production:
    needs: [build, scan, deploy-staging]
    runs-on: ubuntu-latest
    if: startsWith(github.ref, 'refs/tags/v')
    environment: production  # Requires manual approval
    steps:
      - uses: actions/checkout@v4

      - name: Configure AWS credentials
        uses: aws-actions/configure-aws-credentials@v4
        with:
          role-to-assume: arn:aws:iam::123456789012:role/deploy-production
          aws-region: us-east-1

      - name: Login to ECR
        uses: aws-actions/amazon-ecr-login@v2

      - name: Promote staging to production
        run: |
          IMAGE="${{ env.ECR_REGISTRY }}/${{ env.IMAGE_NAME }}"
          # Tag the staging image as production
          MANIFEST=$(aws ecr batch-get-image \
            --repository-name "${{ env.IMAGE_NAME }}" \
            --image-id imageTag=staging \
            --query 'images[0].imageManifest' \
            --output text)
          aws ecr put-image \
            --repository-name "${{ env.IMAGE_NAME }}" \
            --image-tag production \
            --image-manifest "$MANIFEST"

      - name: Deploy to production EKS
        run: |
          aws eks update-kubeconfig --name production-cluster --region us-east-1
          kubectl set image deployment/user-service \
            user-service="${{ env.ECR_REGISTRY }}/${{ env.IMAGE_NAME }}:production" \
            -n production
          kubectl rollout status deployment/user-service -n production --timeout=300s
```

### Why This Pipeline Works

**Four separate jobs:** Build, scan, staging deploy, and production deploy are
independent jobs. This means:
- Scan failures block deployment without affecting the build.
- Staging deployment happens automatically on main-branch pushes.
- Production deployment requires manual approval (via GitHub's `environment`
  feature) and only triggers on version tags.

**Trivy blocks on CRITICAL/HIGH:** The `exit-code: "1"` setting causes Trivy
to exit with code 1 if it finds CRITICAL or HIGH vulnerabilities. GitHub
Actions treats any non-zero exit code as a failure, so the scan job fails and
blocks downstream jobs.

**Git SHA embedded in image:** The `APP_VERSION` and `GIT_SHA` build args are
passed to the Dockerfile, which sets them as environment variables. Any
running container can report its source commit.

**OIDC authentication:** The pipeline uses GitHub's OIDC provider to assume
AWS IAM roles, avoiding long-lived credentials. Each job assumes a different
role with the minimum permissions it needs.

---

## Part D: Retention Policies

### ECR Lifecycle Policy

Save as `lifecycle-policy.json`:

```json
{
  "rules": [
    {
      "rulePriority": 1,
      "description": "Keep last 20 images per repository",
      "selection": {
        "tagStatus": "any",
        "countType": "imageCountMoreThan",
        "countNumber": 20
      },
      "action": {
        "type": "expire"
      }
    },
    {
      "rulePriority": 2,
      "description": "Expire non-release images after 90 days",
      "selection": {
        "tagStatus": "tagged",
        "tagPrefixList": ["main-", "feature-", "staging"],
        "countType": "sinceImagePushed",
        "countUnit": "days",
        "countNumber": 90
      },
      "action": {
        "type": "expire"
      }
    },
    {
      "rulePriority": 3,
      "description": "Delete untagged images after 7 days",
      "selection": {
        "tagStatus": "untagged",
        "countType": "sinceImagePushed",
        "countUnit": "days",
        "countNumber": 7
      },
      "action": {
        "type": "expire"
      }
    }
  ]
}
```

Apply to all repositories:

```bash
for REPO in $(aws ecr describe-repositories --query 'repositories[].repositoryName' --output text); do
  aws ecr put-lifecycle-policy \
    --repository-name "$REPO" \
    --lifecycle-policy-text file://lifecycle-policy.json
done
```

### Harbor Retention Policy

Configure via Harbor UI or API:

```bash
# Create a tag retention policy for the nova-backend project
curl -X POST "https://harbor.novacorp.com/api/v2.0/projects/nova-backend/tag-retention/rules" \
  -H "Authorization: Basic $(echo -n admin:password | base64)" \
  -H "Content-Type: application/json" \
  -d '{
    "algorithm": "or",
    "rules": [
      {
        "disabled": false,
        "action": "retain",
        "template": "latestPushedK",
        "params": {"latestPushedK": 20},
        "tag_selectors": [{"kind": "wildcard", "decoration": "matches", "pattern": "**"}],
        "scope_selectors": [{"kind": "regularExpression", "decoration": "matches", "pattern": ".*"}]
      },
      {
        "disabled": false,
        "action": "retain",
        "template": "ncpus",
        "params": {"nDays": 90},
        "tag_selectors": [{"kind": "wildcard", "decoration": "matches", "pattern": "**"}],
        "scope_selectors": [{"kind": "regularExpression", "decoration": "matches", "pattern": ".*"}]
      }
    ],
    "trigger": {
      "kind": "scheduled",
      "settings": {"cron": "0 0 0 * * *"}
    }
  }'
```

---

## Part E: Registry Mirroring (Bonus)

### Harbor as a Pull-Through Cache for ECR

The on-premises Harbor instance is configured to proxy ECR. When an on-
premises workload pulls `nova/backend/user-service:1.2.3`, Harbor checks
its local cache first. If the image is not cached, Harbor pulls it from ECR,
caches it locally, and serves it to the requester. Subsequent pulls are
served from the local cache.

**Harbor configuration:**

```
1. Add ECR as a registry endpoint:
   Harbor UI: Registries > New Endpoint
   Name: ecr-primary
   Provider: Aws ECR
   Endpoint URL: https://123456789012.dkr.ecr.us-east-1.amazonaws.com
   Access Key: (IAM access key with ECR pull permissions)
   Access Secret: (IAM secret key)

2. Create a proxy cache project:
   Harbor UI: Projects > New Project
   Project Name: nova
   Project Type: Proxy Cache
   Registry Endpoint: ecr-primary
```

**Docker daemon configuration for on-premises servers:**

```json
{
  "registry-mirrors": ["https://harbor.novacorp.com"]
}
```

With this configuration:
- On-premises servers pull from Harbor by default.
- Harbor caches images locally after the first pull.
- No internet access is required for subsequent pulls.
- If ECR is unreachable, Harbor still serves cached images.
- Bandwidth from AWS to on-premises is minimized (each image is
  transferred once, then served from local cache).

### Alternative: Harbor Replication

Instead of pull-through caching, configure Harbor to replicate images from
ECR on a schedule. This is useful when you want explicit control over which
images are available on-premises.

```
Harbor UI: Administration > Replications > New Replication
Name: ecr-to-onprem
Source: ecr-primary (pull-based)
Trigger: Scheduled (every 6 hours)
Filters:
  - Repository: nova/**
  - Tag: *.*.* (only semver tags)
Destination: local registry
```

This approach replicates only release images, not every development build.
It reduces storage requirements and ensures on-premises only has production-
ready images.

---

## Common Mistakes

1. **Using a single ECR repository for all services.** This makes lifecycle
   policies and access control impossible. Each service needs its own
   repository so that retention rules and IAM policies can be scoped
   correctly.

2. **Not enabling scan-on-push.** If scanning only happens in CI, a
   developer who builds and pushes locally bypasses the scan. Enable ECR
   scan-on-push as a safety net.

3. **Using `latest` in the CI/CD pipeline.** The pipeline should always
   produce tags with the git SHA. Using `latest` in a pipeline makes it
   impossible to know which commit is deployed.

4. **Granting CI/CD pipelines too many permissions.** The CI role should only
   be able to push to its team's repositories. It should not have access to
   other teams' repositories or the ability to delete images.

5. **Forgetting to set up pull-through caching for on-premises.** Without a
   mirror, every on-premises pull goes over the internet to ECR. This is
   slow, expensive, and breaks when the internet connection is down.

6. **Not setting tag immutability on release tags.** Without immutability
   rules, someone could accidentally push a different image with the same
   tag (e.g., overwriting `1.2.3`). ECR and Harbor both support tag
   immutability -- enable it for semver tags.

7. **Underestimating storage growth.** Without lifecycle policies, storage
   grows linearly with every push. 22 services pushing 5 times per day
   produces 110 images per day, or ~3,300 per month. At 150MB each, that
   is ~500GB/month. Lifecycle policies keep this bounded.
