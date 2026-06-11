# Module 06: Image Registries — Sharing and Storing Docker Images

> **Previous Module:** [05-image-caching](../05-image-caching/README.md) — Speeding up builds with layer caching
> **Next Module:** [07-docker-compose](../07-docker-compose/README.md) — Orchestrating multi-container applications

---

## The Problem

In the previous module, you learned how to cache Docker image layers to avoid rebuilding the same code over and over. Your builds are fast now. But there is a glaring problem: **your images exist only on your machine**.

When your teammate needs to run the same application, they must:
1. Clone the repository
2. Build the Docker image from scratch
3. Wait for the entire build process (even with caching, the first build is slow)

When you deploy to a server, you must:
1. SSH into the server
2. Copy your Dockerfile and source code
3. Build the image on the server
4. Hope the server has the same build environment as your laptop

When you scale to multiple servers, you must:
1. Repeat the above on every single server
2. Pray that all servers build identical images
3. Deal with inconsistent versions when something goes wrong

This is like writing a book, printing one copy, and then expecting everyone in the world to come to your house to read it. You need a library — a central place where images are stored and distributed.

**This is what an image registry does.**

---

## The Naive Way — Copying Images Manually

The simplest approach is to export images as tar files and transfer them manually.

### Step 1: Save an image to a file

```bash
# Build your image
docker build -t myapp:1.0 .

# Save it to a tar archive
docker save myapp:1.0 -o myapp-1.0.tar

# Check the size
ls -lh myapp-1.0.tar
# -rw------- 1 user user 847M Jun 11 10:00 myapp-1.0.tar
```

### Step 2: Transfer the file

```bash
# Copy to another machine
scp myapp-1.0.tar user@server:/tmp/

# Or use rsync for better performance
rsync -avz --progress myapp-1.0.tar user@server:/tmp/
```

### Step 3: Load the image on the target machine

```bash
# SSH into the server
ssh user@server

# Load the image
docker load -i /tmp/myapp-1.0.tar

# Verify it loaded
docker images | grep myapp
```

### Why This Fails

| Problem | Impact |
|---------|--------|
| **No versioning** | Which tar file is the latest? `myapp-final.tar`? `myapp-final-v2-REAL.tar`? |
| **No access control** | Anyone with the file has the image, including secrets baked into it |
| **No scanning** | No way to check for vulnerabilities before deployment |
| **Manual process** | Every deployment requires human intervention |
| **No deduplication** | Each tar file contains full layers, wasting storage |
| **Slow transfers** | 847MB per image, multiplied by every team member and every server |
| **No automation** | CI/CD pipelines cannot push or pull images |
| **Inconsistency** | Different machines may end up with different versions |

This approach works for a single developer with a single server. It breaks down immediately when you have a team.

---

## The Right Way — Using Docker Hub

Docker Hub is the default public registry. It is the GitHub of container images. Every Docker installation is pre-configured to pull from Docker Hub.

### Step 1: Create a Docker Hub Account

```bash
# Sign up at https://hub.docker.com
# Then log in from your terminal
docker login
# Enter your username and password
```

### Step 2: Tag Your Image for Docker Hub

Docker Hub requires images to follow the naming convention: `<username>/<repository>:<tag>`

```bash
# Wrong tag — Docker Hub will reject this
docker tag myapp:1.0 myapp:1.0

# Correct tag — includes your Docker Hub username
docker tag myapp:1.0 yourusername/myapp:1.0

# Also tag as latest
docker tag myapp:1.0 yourusername/myapp:latest
```

### Step 3: Push the Image

```bash
# Push the versioned tag
docker push yourusername/myapp:1.0

# Push latest
docker push yourusername/myapp:latest
```

Output:
```
The push refers to repository [docker.io/yourusername/myapp]
5f70bf18a086: Pushed
e12ab88d14dd: Pushed
1.0: digest: sha256:3a7b8f... size: 1345
```

### Step 4: Pull from Any Machine

```bash
# On any machine with Docker installed
docker pull yourusername/myapp:1.0

# Run it
docker run -d -p 8080:80 yourusername/myapp:1.0
```

### Docker Hub Limits (Free Tier)

| Limit | Value |
|-------|-------|
| Public repositories | Unlimited |
| Private repositories | 1 |
| Image pulls per 6 hours (anonymous) | 100 |
| Image pulls per 6 hours (authenticated) | 200 |
| Image size | No hard limit, but large images are slow |

For teams and production, you need more than the free tier provides.

---

## Tag Strategies — Naming Images Properly

Tags are labels that point to a specific image. A single image can have multiple tags. Choosing the right tagging strategy is critical for managing images in production.

### The `latest` Tag Trap

```bash
# This is ambiguous and dangerous
docker pull yourusername/myapp:latest
```

The `latest` tag is **not** automatically updated. It is just a default tag name. If you push `myapp:1.0` without also pushing `myapp:latest`, the `latest` tag still points to whatever was previously pushed with that tag.

```bash
# WRONG: Only tagging as latest
docker tag myapp:1.0 yourusername/myapp:latest
docker push yourusername/myapp:latest

# PROBLEM: Six months later, what version is "latest"?
# Nobody knows. There is no audit trail.
```

### Semantic Versioning (Recommended for Releases)

Follow [Semantic Versioning](https://semver.org/): `MAJOR.MINOR.PATCH`

```bash
# Major version — breaking changes
docker tag myapp:1.0 yourusername/myapp:1.0.0
docker tag myapp:1.0 yourusername/myapp:1.0    # Minor version alias
docker tag myapp:1.0 yourusername/myapp:1       # Major version alias

# When you release 1.0.1 (bug fix)
docker tag myapp:1.0.1 yourusername/myapp:1.0.1
docker tag myapp:1.0.1 yourusername/myapp:1.0   # Updated
docker tag myapp:1.0.1 yourusername/myapp:1     # Updated

# When you release 1.1.0 (new feature)
docker tag myapp:1.1.0 yourusername/myapp:1.1.0
docker tag myapp:1.1.0 yourusername/myapp:1.1
docker tag myapp:1.1.0 yourusername/myapp:1     # Updated

# When you release 2.0.0 (breaking change)
docker tag myapp:2.0.0 yourusername/myapp:2.0.0
docker tag myapp:2.0.0 yourusername/myapp:2.0
docker tag myapp:2.0.0 yourusername/myapp:2     # Updated
# myapp:1 still points to 1.1.0 — safe for existing users
```

This allows users to pin at different levels:
- `myapp:1.0.0` — exact version, never changes
- `myapp:1.0` — latest patch of 1.0.x
- `myapp:1` — latest minor of 1.x.x
- `myapp:latest` — latest major (use with caution)

### Git SHA Tagging (Recommended for CI/CD)

Tag every image with the Git commit that produced it.

```bash
# In your CI/CD pipeline
GIT_SHA=$(git rev-parse --short HEAD)
IMAGE_NAME="yourusername/myapp"

# Build and tag with git SHA
docker build -t ${IMAGE_NAME}:${GIT_SHA} .

# Also tag with branch name
BRANCH=$(git rev-parse --abbrev-ref HEAD)
docker tag ${IMAGE_NAME}:${GIT_SHA} ${IMAGE_NAME}:${BRANCH}

# Push both tags
docker push ${IMAGE_NAME}:${GIT_SHA}
docker push ${IMAGE_NAME}:${BRANCH}
```

Benefits:
- Every image is traceable to the exact code that produced it
- No ambiguity about what is deployed
- Easy rollback: just redeploy the previous SHA
- Audit trail for compliance

### Combined Strategy (Best Practice)

Use both semantic versioning and git SHA together.

```bash
# In CI/CD
GIT_SHA=$(git rev-parse --short HEAD)
VERSION="1.2.3"
IMAGE_NAME="yourusername/myapp"

docker build -t ${IMAGE_NAME}:${GIT_SHA} .

# Tag with version if this is a release
if [ "$IS_RELEASE" = "true" ]; then
    docker tag ${IMAGE_NAME}:${GIT_SHA} ${IMAGE_NAME}:${VERSION}
    docker tag ${IMAGE_NAME}:${GIT_SHA} ${IMAGE_NAME}:latest
    docker push ${IMAGE_NAME}:${VERSION}
    docker push ${IMAGE_NAME}:latest
fi

docker push ${IMAGE_NAME}:${GIT_SHA}
```

### Tags to Avoid

```bash
# NEVER do these
docker tag myapp:1.0 yourusername/myapp:production    # Environment names in tags
docker tag myapp:1.0 yourusername/myapp:test           # Status names in tags
docker tag myapp:1.0 yourusername/myapp:final          # Subjective labels
docker tag myapp:1.0 yourusername/myapp:v2-fixed       # Descriptive chaos
docker tag myapp:1.0 yourusername/myapp:DO-NOT-DELETE  # Begging
```

Tags should describe **what** the image contains, not **where** it goes or **how** it feels.

---

## Private Registries — Controlling Access

Docker Hub public repositories are visible to everyone. For proprietary code, you need a private registry.

### Option 1: Docker Hub Private Repositories

```bash
# Create a private repository on Docker Hub (web UI)
# Then push to it
docker login
docker tag myapp:1.0 yourusername/myapp:1.0
docker push yourusername/myapp:1.0
# This repository is now private — only you and collaborators can pull it
```

### Option 2: Amazon Elastic Container Registry (ECR)

```bash
# Install AWS CLI and configure credentials
aws configure

# Create a repository
aws ecr create-repository \
    --repository-name myapp \
    --image-scanning-configuration scanOnPush=true \
    --encryption-configuration encryptionType=AES256

# Authenticate Docker to ECR
aws ecr get-login-password --region us-east-1 | \
    docker login --username AWS --password-stdin \
    123456789012.dkr.ecr.us-east-1.amazonaws.com

# Tag and push
docker tag myapp:1.0 123456789012.dkr.ecr.us-east-1.amazonaws.com/myapp:1.0
docker push 123456789012.dkr.ecr.us-east-1.amazonaws.com/myapp:1.0

# Pull from another machine (after authenticating)
docker pull 123456789012.dkr.ecr.us-east-1.amazonaws.com/myapp:1.0
```

ECR pricing:
- Storage: $0.10 per GB per month
- Data transfer: Standard AWS rates
- Image scanning: $0.09 per image scan

### Option 3: Google Container Registry (GCR) / Artifact Registry

```bash
# Authenticate with Google Cloud
gcloud auth configure-docker

# Tag and push to GCR
docker tag myapp:1.0 gcr.io/my-project-id/myapp:1.0
docker push gcr.io/my-project-id/myapp:1.0

# For Artifact Registry (newer, recommended)
gcloud auth configure-docker us-docker.pkg.dev
docker tag myapp:1.0 us-docker.pkg.dev/my-project-id/my-repo/myapp:1.0
docker push us-docker.pkg.dev/my-project-id/my-repo/myapp:1.0
```

### Option 4: Azure Container Registry (ACR)

```bash
# Create a registry
az acr create \
    --resource-group myResourceGroup \
    --name myregistry \
    --sku Basic

# Login
az acr login --name myregistry

# Tag and push
docker tag myapp:1.0 myregistry.azurecr.io/myapp:1.0
docker push myregistry.azurecr.io/myapp:1.0
```

### Option 5: GitHub Container Registry (GHCR)

```bash
# Authenticate with a personal access token
echo $GITHUB_TOKEN | docker login ghcr.io -u USERNAME --password-stdin

# Tag and push
docker tag myapp:1.0 ghcr.io/yourusername/myapp:1.0
docker push ghcr.io/yourusername/myapp:1.0
```

### Registry Comparison

| Feature | Docker Hub | ECR | GCR/AR | ACR | GHCR | Harbor |
|---------|-----------|-----|--------|-----|------|--------|
| **Hosting** | Cloud | AWS | GCP | Azure | GitHub | Self-hosted |
| **Free tier** | 1 private repo | None | None | None | Unlimited public | Free (OSS) |
| **Geo-replication** | Paid | Per region | Multi-region | Premium | Global | Manual |
| **Vulnerability scanning** | Basic | Enhanced | On-demand | Basic | Dependabot | Trivy built-in |
| **Access control** | Basic | IAM | IAM | AAD/RBAC | GitHub teams | RBAC + LDAP |
| **Best for** | Open source | AWS shops | GCP shops | Azure shops | GitHub projects | Enterprise |

---

## Self-Hosted Registries — Running Harbor

For enterprise environments, you may need full control over your registry. [Harbor](https://goharbor.io/) is the most popular open-source registry.

### Why Self-Host?

- **Compliance**: Data must stay within your network
- **Cost**: No per-GB cloud storage fees
- **Control**: Full control over access, scanning, and retention
- **Integration**: LDAP/AD authentication, custom policies
- **Air-gapped environments**: No internet access required

### Installing Harbor with Docker Compose

```bash
# Download the Harbor installer
HARBOR_VERSION="v2.10.0"
wget https://github.com/goharbor/harbor/releases/download/${HARBOR_VERSION}/harbor-offline-installer-${HARBOR_VERSION}.tgz

# Extract
tar xzf harbor-offline-installer-${HARBOR_VERSION}.tgz
cd harbor

# Copy the configuration template
cp harbor.yml.tmpl harbor.yml
```

Edit `harbor.yml`:

```yaml
# hostname: The hostname of your Harbor instance
hostname: harbor.yourcompany.com

# http configuration
http:
  port: 80

# https configuration (recommended for production)
https:
  port: 443
  certificate: /data/cert/harbor.crt
  private_key: /data/cert/harbor.key

# Harbor admin password (CHANGE THIS)
harbor_admin_password: ChangeMeNow123!

# Database configuration
database:
  password: ChangeMeNow123!
  max_idle_conns: 100
  max_open_conns: 900

# Storage configuration
storage_service:
  s3:
    accesskey: YOUR_ACCESS_KEY
    secretkey: YOUR_SECRET_KEY
    region: us-east-1
    bucket: harbor-registry
```

```bash
# Run the installer
sudo ./install.sh --with-trivy --with-chartmuseum

# Verify Harbor is running
docker compose ps
```

### Configuring Docker to Trust Harbor

```bash
# If using self-signed certificates
sudo mkdir -p /etc/docker/certs.d/harbor.yourcompany.com
sudo cp ca.crt /etc/docker/certs.d/harbor.yourcompany.com/

# Restart Docker
sudo systemctl restart docker

# Login to Harbor
docker login harbor.yourcompany.com
# Username: admin
# Password: (the password you set)

# Push an image
docker tag myapp:1.0 harbor.yourcompany.com/library/myapp:1.0
docker push harbor.yourcompany.com/library/myapp:1.0
```

### Harbor Key Features

- **Projects**: Logical grouping of repositories with access control
- **Robot accounts**: Service accounts for CI/CD pipelines
- **Replication**: Sync images between Harbor instances
- **Retention policies**: Automatically delete old tags
- **Quotas**: Limit storage per project
- **Webhook notifications**: Trigger actions on push/pull events

---

## Image Scanning in Registries — Finding Vulnerabilities

Docker images contain operating system packages and application dependencies. These may have known vulnerabilities (CVEs). Registries can scan images automatically.

### Trivy — The Most Popular Scanner

```bash
# Install Trivy
sudo apt-get install trivy   # Debian/Ubuntu
brew install trivy            # macOS

# Scan a local image
trivy image yourusername/myapp:1.0
```

Output:
```
yourusername/myapp:1.0 (debian 12.0)
=====================================
Total: 42 (UNKNOWN: 0, LOW: 15, MEDIUM: 18, HIGH: 7, CRITICAL: 2)

+------------------+-----------+----------+-----------+---------+-----------+
|     LIBRARY      | INSTALLED |  FIXED   |   TYPE    | VULN ID | SEVERITY  |
+------------------+-----------+----------+-----------+---------+-----------+
| openssl          | 3.0.9-1   | 3.0.10-1 | debian    | CVE-2023| CRITICAL  |
| libssl3          | 3.0.9-1   | 3.0.10-1 | debian    | CVE-2023| CRITICAL  |
| curl             | 7.88.1    | 7.88.1-1 | debian    | CVE-2023| HIGH      |
+------------------+-----------+----------+-----------+---------+-----------+
```

### Scanning on Push (ECR Example)

```bash
# ECR scans automatically on push (if enabled)
aws ecr create-repository \
    --repository-name myapp \
    --image-scanning-configuration scanOnPush=true

# Check scan results
aws ecr describe-image-scan-findings \
    --repository-name myapp \
    --image-id imageTag=1.0
```

### Scanning in CI/CD Pipelines

```yaml
# GitHub Actions example
name: Build and Scan
on:
  push:
    branches: [main]

jobs:
  build-and-scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Build image
        run: docker build -t myapp:${{ github.sha }} .

      - name: Run Trivy vulnerability scanner
        uses: aquasecurity/trivy-action@master
        with:
          image-ref: 'myapp:${{ github.sha }}'
          format: 'sarif'
          output: 'trivy-results.sarif'
          severity: 'CRITICAL,HIGH'
          exit-code: '1'  # Fail the build on critical/high vulnerabilities

      - name: Upload Trivy scan results
        uses: github/codeql-action/upload-sarif@v2
        with:
          sarif_file: 'trivy-results.sarif'
```

### Scanning Policies in Harbor

```bash
# Harbor has built-in Trivy integration
# Configure via the Harbor UI:

# 1. Go to Projects > myapp > Configuration
# 2. Enable "Automatically scan images on push"
# 3. Set vulnerability severity threshold to "High"
# 4. Enable "Prevent vulnerable images from running" (optional)
```

When an image fails the scan policy:
```
$ docker push harbor.yourcompany.com/myapp:1.0
# Image pushed successfully, but scan finds CRITICAL vulnerabilities
# If "prevent" policy is active:
$ docker pull harbor.yourcompany.com/myapp:1.0
Error: image has vulnerabilities that exceed the configured threshold
```

---

## Registry Garbage Collection — Reclaiming Storage

Every push creates new layers. Over time, registries accumulate orphaned layers from deleted tags, overwritten images, and failed pushes. Garbage collection reclaims this storage.

### How Docker Images Store Data

```
Image: myapp:1.0
  Layer 1: ubuntu:22.04 base         (shared, 77MB)
  Layer 2: apt-get install packages   (shared, 45MB)
  Layer 3: COPY application code      (unique, 2MB)
  Layer 4: RUN npm build              (unique, 15MB)

Image: myapp:1.1
  Layer 1: ubuntu:22.04 base         (shared, 77MB)  -- same as 1.0
  Layer 2: apt-get install packages   (shared, 45MB)  -- same as 1.0
  Layer 3: COPY application code      (unique, 2.1MB) -- NEW layer
  Layer 4: RUN npm build              (unique, 16MB)  -- NEW layer
```

When you delete `myapp:1.0`, layers 3 and 4 from that version become orphaned. They are no longer referenced by any tag but still occupy disk space.

### Docker Hub Garbage Collection

Docker Hub automatically manages storage. You cannot run garbage collection manually. To free space:
- Delete unused repositories via the web UI
- Remove old tags via the API

```bash
# List tags in a repository
curl -s "https://hub.docker.com/v2/repositories/yourusername/myapp/tags/?page_size=100" | \
    jq '.results[].name'

# Delete a specific tag
curl -X DELETE \
    -H "Authorization: Bearer ${TOKEN}" \
    "https://hub.docker.com/v2/repositories/yourusername/myapp/tags/old-tag/"
```

### Harbor Garbage Collection

```bash
# Via Harbor CLI (docker exec)
docker exec -it harbor-core gc start

# Via Harbor API
curl -X POST "https://harbor.yourcompany.com/api/v2.0/system/gc/schedule" \
    -H "Authorization: Basic $(echo -n admin:password | base64)" \
    -H "Content-Type: application/json" \
    -d '{
        "parameters": {
            "delete_untagged": true,
            "dry_run": false
        },
        "schedule": {
            "type": "Weekly",
            "cron": "0 0 0 * * 0"
        }
    }'

# Check GC status
curl "https://harbor.yourcompany.com/api/v2.0/system/gc" \
    -H "Authorization: Basic $(echo -n admin:password | base64)"
```

### Retention Policies (Automated Cleanup)

Harbor can automatically delete old tags based on rules:

```bash
# Via Harbor UI: Projects > myapp > Tag Retention
# Rule 1: Keep the 10 most recently pushed images
# Rule 2: Keep images pushed within the last 30 days
# Rule 3: Always keep images tagged with semver pattern (v*.*.*)
```

### Best Practices for Storage Management

```bash
# 1. Always delete tags you no longer need
# Don't just push new tags and forget old ones

# 2. Use multi-stage builds to reduce image size
# (Module 04 covered this)

# 3. Schedule regular garbage collection
# Weekly for active registries, monthly for small ones

# 4. Monitor registry storage usage
# Set up alerts when storage exceeds 80% capacity

# 5. Use storage backends with lifecycle policies
# S3 with lifecycle rules can auto-delete old data
```

---

## Multi-Architecture Images — ARM64 and AMD64

Modern infrastructure includes both x86 (AMD64) and ARM (ARM64) servers. Apple Silicon Macs are ARM64. AWS Graviton instances are ARM64. Raspberry Pi is ARM64. You need images that work on all of them.

### The Problem

```bash
# Build on an AMD64 machine
docker build -t myapp:1.0 .
docker push yourusername/myapp:1.0

# Try to pull on an ARM64 machine (Apple M1, Graviton)
docker pull yourusername/myapp:1.0
# May work via emulation (slow) or fail entirely
```

### Building Multi-Architecture Images with Buildx

```bash
# Create a buildx builder that supports multiple platforms
docker buildx create --name multiarch --driver docker-container --use
docker buildx inspect --bootstrap

# Build and push for multiple architectures in one command
docker buildx build \
    --platform linux/amd64,linux/arm64,linux/arm/v7 \
    -t yourusername/myapp:1.0 \
    --push \
    .

# Verify the manifest
docker buildx imagetools inspect yourusername/myapp:1.0
```

Output of `imagetools inspect`:
```
Name:      docker.io/yourusername/myapp:1.0
MediaType: application/vnd.docker.distribution.manifest.list.v2+json
Digest:    sha256:abc123...

Manifests:
  Name:      docker.io/yourusername/myapp:1.0@sha256:def456...
  MediaType: application/vnd.docker.distribution.manifest.v2+json
  Platform:  linux/amd64

  Name:      docker.io/yourusername/myapp:1.0@sha256:ghi789...
  MediaType: application/vnd.docker.distribution.manifest.v2+json
  Platform:  linux/arm64

  Name:      docker.io/yourusername/myapp:1.0@sha256:jkl012...
  MediaType: application/vnd.docker.distribution.manifest.v2+json
  Platform:  linux/arm/v7
```

### Multi-Architecture Dockerfile

Some base images support multiple architectures natively. Others require conditional logic.

```dockerfile
# This Dockerfile works on both AMD64 and ARM64
FROM --platform=$BUILDPLATFORM node:20-alpine AS builder

# $BUILDPLATFORM is the platform of the machine running the build
# $TARGETPLATFORM is the platform the image will run on
ARG TARGETPLATFORM
ARG BUILDPLATFORM

WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production
COPY . .
RUN npm run build

FROM --platform=$TARGETPLATFORM node:20-alpine
WORKDIR /app
COPY --from=builder /app/dist ./dist
COPY --from=builder /app/node_modules ./node_modules
COPY --from=builder /app/package.json ./

EXPOSE 3000
CMD ["node", "dist/server.js"]
```

### Multi-Architecture in CI/CD

```yaml
# GitHub Actions for multi-arch builds
name: Build Multi-Arch
on:
  push:
    tags: ['v*']

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Set up QEMU
        uses: docker/setup-qemu-action@v3

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3

      - name: Login to Docker Hub
        uses: docker/login-action@v3
        with:
          username: ${{ secrets.DOCKER_USERNAME }}
          password: ${{ secrets.DOCKER_TOKEN }}

      - name: Extract version from tag
        id: version
        run: echo "VERSION=${GITHUB_REF#refs/tags/v}" >> $GITHUB_OUTPUT

      - name: Build and push
        uses: docker/build-push-action@v5
        with:
          context: .
          platforms: linux/amd64,linux/arm64
          push: true
          tags: |
            yourusername/myapp:${{ steps.version.outputs.VERSION }}
            yourusername/myapp:latest
          cache-from: type=gha
          cache-to: type=gha,mode=max
```

### Performance Considerations

Building for ARM64 on an AMD64 machine uses QEMU emulation, which is **slow**.

| Build Type | Time (approximate) |
|-----------|-------------------|
| Native AMD64 on AMD64 | 2 minutes |
| Native ARM64 on ARM64 | 2 minutes |
| ARM64 via QEMU on AMD64 | 10-20 minutes |

Solutions:
- Use GitHub Actions with native ARM64 runners (available since 2024)
- Build on separate machines and merge manifests
- Use `--platform` only for the final stage, not intermediate build stages

---

## Cost Optimization — Storage and Bandwidth

Registry costs can grow unexpectedly. Understanding the cost model is essential.

### Storage Costs

```
Scenario: 50 microservices, each with 20 tagged versions

Image size: ~200MB per image (optimistic)
Total storage: 50 services x 20 versions x 200MB = 200GB

Monthly costs:
  Docker Hub Pro:     $7/month (unlimited private repos, but pull limits)
  ECR (us-east-1):    $0.10/GB x 200GB = $20/month
  GCR (us-central1):  $0.10/GB x 200GB = $20/month
  ACR Basic:          $0.167/day = ~$5/month (but limited storage)
  ACR Standard:       $0.334/day = ~$10/month (100GB included)
  Harbor (self-hosted): $0 (but you pay for servers and maintenance)
```

### Bandwidth Costs

```
Scenario: CI/CD pulls images 100 times per day, 200MB each

Daily bandwidth: 100 pulls x 200MB = 20GB
Monthly bandwidth: 20GB x 30 = 600GB

Cost comparison:
  ECR (same region):  $0.00 (free within AWS)
  ECR (cross-region): $0.02/GB x 600GB = $12/month
  ECR (to internet):  $0.09/GB x 600GB = $54/month
  Docker Hub:         Included in plan (but rate limited)
  Self-hosted (same DC): $0.00 (internal network)
```

### Optimization Strategies

**1. Use Multi-Stage Builds to Reduce Image Size**

```dockerfile
# BEFORE: 1.2GB image
FROM node:20
WORKDIR /app
COPY . .
RUN npm install
RUN npm run build
EXPOSE 3000
CMD ["node", "dist/server.js"]

# AFTER: 180MB image (85% reduction)
FROM node:20-alpine AS builder
WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production
COPY . .
RUN npm run build

FROM node:20-alpine
WORKDIR /app
COPY --from=builder /app/dist ./dist
COPY --from=builder /app/node_modules ./node_modules
EXPOSE 3000
CMD ["node", "dist/server.js"]
```

**2. Use Shared Base Images**

```dockerfile
# Each service shares the same base
FROM yourcompany/base:node20-alpine
# Base already has security patches, CA certs, etc.
COPY . .
RUN npm ci --only=production && npm run build
```

**3. Implement Tag Retention Policies**

```bash
# Keep only recent tags, not every version ever built
# Harbor: Tag Retention Rules
# - Keep last 10 tags
# - Keep tags matching "v*.*.*"
# - Delete everything else after 90 days
```

**4. Use Registry Mirrors and Caches**

```json
// /etc/docker/daemon.json
{
  "registry-mirrors": ["https://mirror.yourcompany.com"]
}
```

```bash
# Harbor can act as a pull-through cache for Docker Hub
# This reduces external bandwidth and avoids rate limits
# Configure: Registries > New Endpoint > Docker Hub
# Then: Projects > New Project > Proxy Cache > Docker Hub
```

**5. Deduplicate with Content-Addressable Storage**

All major registries use content-addressable storage by default. Layers with identical content are stored once, regardless of how many images reference them. Design your Dockerfiles to maximize layer sharing:

```dockerfile
# GOOD: Dependencies change rarely, code changes often
COPY package*.json ./
RUN npm ci
COPY . .        # Only this layer changes on code updates

# BAD: Everything in one layer
COPY . .
RUN npm ci      # Re-runs even if only code changed
```

---

## Hands-On Lab — Image Registry Workflow

Let's build, push, pull, and scan a real application using Docker Hub.

### Prerequisites

```bash
# You need:
# 1. A Docker Hub account (free at https://hub.docker.com)
# 2. Docker installed
# 3. A terminal

# Login to Docker Hub
docker login
```

### Lab 1: Build, Tag, and Push

```bash
# Create a working directory
mkdir -p /tmp/registry-lab && cd /tmp/registry-lab

# Create a simple application
cat > server.js << 'EOF'
const http = require('http');
const os = require('os');

const server = http.createServer((req, res) => {
  res.writeHead(200, { 'Content-Type': 'application/json' });
  res.end(JSON.stringify({
    message: 'Hello from the registry lab!',
    version: process.env.APP_VERSION || 'unknown',
    hostname: os.hostname(),
    arch: os.arch(),
    platform: os.platform(),
    timestamp: new Date().toISOString()
  }));
});

server.listen(3000, () => {
  console.log('Server running on port 3000');
});
EOF

# Create a Dockerfile
cat > Dockerfile << 'EOF'
FROM node:20-alpine
WORKDIR /app
COPY server.js .
ENV APP_VERSION=1.0.0
EXPOSE 3000
CMD ["node", "server.js"]
EOF

# Build the image
docker build -t registry-lab:1.0.0 .

# Test locally
docker run -d --name lab-test -p 3000:3000 registry-lab:1.0.0
curl http://localhost:3000
docker stop lab-test && docker rm lab-test
```

```bash
# Replace YOUR_USERNAME with your Docker Hub username
export DOCKER_USER="YOUR_USERNAME"

# Tag for Docker Hub
docker tag registry-lab:1.0.0 ${DOCKER_USER}/registry-lab:1.0.0
docker tag registry-lab:1.0.0 ${DOCKER_USER}/registry-lab:latest

# Push to Docker Hub
docker push ${DOCKER_USER}/registry-lab:1.0.0
docker push ${DOCKER_USER}/registry-lab:latest

# Verify on Docker Hub
echo "Check your image at: https://hub.docker.com/r/${DOCKER_USER}/registry-lab"
```

### Lab 2: Pull and Run from Registry

```bash
# Remove the local image completely
docker rmi ${DOCKER_USER}/registry-lab:1.0.0
docker rmi ${DOCKER_USER}/registry-lab:latest
docker rmi registry-lab:1.0.0

# Verify it's gone
docker images | grep registry-lab
# Should show nothing

# Pull from Docker Hub
docker pull ${DOCKER_USER}/registry-lab:1.0.0

# Run it
docker run -d --name lab-pulled -p 3000:3000 ${DOCKER_USER}/registry-lab:1.0.0
curl http://localhost:3000
# Notice: it works exactly the same as when built locally

docker stop lab-pulled && docker rm lab-pulled
```

### Lab 3: Semantic Version Tags

```bash
# Simulate releasing version 1.0.1
cat > server.js << 'EOF'
const http = require('http');
const os = require('os');

const server = http.createServer((req, res) => {
  res.writeHead(200, { 'Content-Type': 'application/json' });
  res.end(JSON.stringify({
    message: 'Hello from the registry lab!',
    version: process.env.APP_VERSION || 'unknown',
    changelog: 'Fixed a critical bug',
    hostname: os.hostname(),
    arch: os.arch(),
    timestamp: new Date().toISOString()
  }));
});

server.listen(3000, () => {
  console.log('Server running on port 3000');
});
EOF

# Build 1.0.1
docker build --build-arg APP_VERSION=1.0.1 -t registry-lab:1.0.1 .

# Actually, the ENV is hardcoded in the Dockerfile, so let's update it
cat > Dockerfile << 'EOF'
FROM node:20-alpine
WORKDIR /app
COPY server.js .
ARG APP_VERSION=1.0.0
ENV APP_VERSION=${APP_VERSION}
EXPOSE 3000
CMD ["node", "server.js"]
EOF

docker build --build-arg APP_VERSION=1.0.1 -t registry-lab:1.0.1 .

# Tag with semantic versioning
docker tag registry-lab:1.0.1 ${DOCKER_USER}/registry-lab:1.0.1
docker tag registry-lab:1.0.1 ${DOCKER_USER}/registry-lab:1.0
docker tag registry-lab:1.0.1 ${DOCKER_USER}/registry-lab:1
docker tag registry-lab:1.0.1 ${DOCKER_USER}/registry-lab:latest

# Push all tags
docker push ${DOCKER_USER}/registry-lab:1.0.1
docker push ${DOCKER_USER}/registry-lab:1.0
docker push ${DOCKER_USER}/registry-lab:1
docker push ${DOCKER_USER}/registry-lab:latest

# Now users can pull at different precision levels
# docker pull ${DOCKER_USER}/registry-lab:1.0.1  # Exact version
# docker pull ${DOCKER_USER}/registry-lab:1.0     # Latest patch
# docker pull ${DOCKER_USER}/registry-lab:1       # Latest minor
```

### Lab 4: Git SHA Tags

```bash
# Simulate a CI/CD pipeline
cd /tmp/registry-lab

# Initialize a git repo
git init
git add .
git commit -m "Initial commit"

# Get the short SHA
GIT_SHA=$(git rev-parse --short HEAD)
echo "Git SHA: ${GIT_SHA}"

# Build and tag with git SHA
docker build --build-arg APP_VERSION=${GIT_SHA} -t ${DOCKER_USER}/registry-lab:${GIT_SHA} .

# Push
docker push ${DOCKER_USER}/registry-lab:${GIT_SHA}

# This image is now traceable to this exact commit
```

### Lab 5: Vulnerability Scanning

```bash
# Install Trivy
# macOS: brew install trivy
# Linux: sudo apt-get install trivy
# Or use Docker:
docker run --rm \
    -v /var/run/docker.sock:/var/run/docker.sock \
    aquasec/trivy image ${DOCKER_USER}/registry-lab:1.0.0

# Scan and output as table
trivy image --severity HIGH,CRITICAL ${DOCKER_USER}/registry-lab:1.0.0

# Scan with exit code (useful in CI)
trivy image --exit-code 1 --severity CRITICAL ${DOCKER_USER}/registry-lab:1.0.0
# Exit code 1 means critical vulnerabilities were found
```

### Lab 6: Inspect Registry Metadata

```bash
# View image layers and metadata
docker inspect ${DOCKER_USER}/registry-lab:1.0.0

# View image history (layer-by-layer commands)
docker history ${DOCKER_USER}/registry-lab:1.0.0

# View the image manifest (multi-arch info)
docker manifest inspect ${DOCKER_USER}/registry-lab:1.0.0

# Check image size
docker images ${DOCKER_USER}/registry-lab --format "table {{.Tag}}\t{{.Size}}"
```

### Lab 7: Multi-Architecture Build

```bash
# Create a buildx builder
docker buildx create --name lab-builder --use
docker buildx inspect --bootstrap

# Build for multiple architectures (push to registry)
docker buildx build \
    --platform linux/amd64,linux/arm64 \
    --build-arg APP_VERSION=1.0.0 \
    -t ${DOCKER_USER}/registry-lab:1.0.0-multiarch \
    --push \
    .

# Inspect the multi-arch manifest
docker buildx imagetools inspect ${DOCKER_USER}/registry-lab:1.0.0-multiarch

# Clean up the builder
docker buildx rm lab-builder
```

### Lab 8: Cleanup

```bash
# Remove local images
docker rmi registry-lab:1.0.0
docker rmi registry-lab:1.0.1
docker rmi ${DOCKER_USER}/registry-lab:1.0.0
docker rmi ${DOCKER_USER}/registry-lab:1.0.1
docker rmi ${DOCKER_USER}/registry-lab:1.0
docker rmi ${DOCKER_USER}/registry-lab:1
docker rmi ${DOCKER_USER}/registry-lab:latest
docker rmi ${DOCKER_USER}/registry-lab:${GIT_SHA}
docker rmi ${DOCKER_USER}/registry-lab:1.0.0-multiarch

# Remove the lab directory
rm -rf /tmp/registry-lab

# Remove tags from Docker Hub via API (optional)
# Go to https://hub.docker.com/r/${DOCKER_USER}/registry-lab/tags
# and delete tags manually, or use the API:

# List tags
curl -s "https://hub.docker.com/v2/repositories/${DOCKER_USER}/registry-lab/tags/" | \
    jq '.results[].name'

# Delete repository entirely (if desired)
# This requires a Docker Hub API token
# curl -X DELETE -H "Authorization: Bearer ${TOKEN}" \
#     "https://hub.docker.com/v2/repositories/${DOCKER_USER}/registry-lab/"
```

---

## Quick Reference — Registry Commands

```bash
# Authentication
docker login                              # Docker Hub
docker login registry.example.com         # Custom registry
aws ecr get-login-password | docker login --username AWS --password-stdin <ecr-url>

# Tagging
docker tag SOURCE_IMAGE TARGET_IMAGE:TAG
docker tag myapp:1.0 user/myapp:1.0
docker tag myapp:1.0 registry.example.com/project/myapp:1.0

# Push
docker push user/myapp:1.0
docker push registry.example.com/project/myapp:1.0

# Pull
docker pull user/myapp:1.0
docker pull registry.example.com/project/myapp:1.0

# Search
docker search nginx
docker search --filter stars=100 nginx

# Inspect remote image (without pulling)
docker manifest inspect user/myapp:1.0
docker buildx imagetools inspect user/myapp:1.0

# Multi-arch build
docker buildx create --name builder --use
docker buildx build --platform linux/amd64,linux/arm64 -t user/myapp:1.0 --push .

# Image scanning
trivy image user/myapp:1.0
```

---

## Common Mistakes

### 1. Pushing Without Authentication

```bash
# WRONG: Forgot to login
docker push yourusername/myapp:1.0
# error: denied: requested access to the resource is denied

# FIX: Login first
docker login
docker push yourusername/myapp:1.0
```

### 2. Using `latest` in Production

```bash
# WRONG: No version control
docker run -d yourusername/myapp:latest

# RIGHT: Pin to specific version
docker run -d yourusername/myapp:1.2.3
```

### 3. Pushing Large Images

```bash
# WRONG: Pushing a 2GB image
docker push yourusername/myapp:1.0   # Takes 10 minutes, wastes storage

# RIGHT: Use multi-stage builds
# See Module 04 for details
```

### 4. Not Scanning Before Push

```bash
# WRONG: Push first, scan never
docker push yourusername/myapp:1.0

# RIGHT: Scan before pushing to production
trivy image --exit-code 1 --severity CRITICAL yourusername/myapp:1.0
docker push yourusername/myapp:1.0
```

### 5. Hardcoding Registry URLs

```bash
# WRONG: Registry URL in every script
docker pull 123456789012.dkr.ecr.us-east-1.amazonaws.com/myapp:1.0

# RIGHT: Use environment variables
REGISTRY=${REGISTRY:-docker.io}
IMAGE=${REGISTRY}/myapp:1.0
docker pull ${IMAGE}
```

---

## What You Learned

- **Image registries** are centralized storage for Docker images, enabling sharing across teams and servers
- **Docker Hub** is the default public registry; **ECR, GCR, ACR, GHCR** are cloud-specific private registries
- **Harbor** is the leading self-hosted, open-source registry for enterprise environments
- **Tag strategies** matter: semantic versioning for releases, git SHA for traceability, avoid `latest` in production
- **Image scanning** (Trivy) finds vulnerabilities before they reach production
- **Garbage collection** reclaims storage from orphaned layers and deleted tags
- **Multi-architecture images** support AMD64, ARM64, and other platforms from a single tag
- **Cost optimization** requires small images, retention policies, and registry mirrors

---

## Limitation

You can now share Docker images across teams and servers. But deploying a real application requires running multiple containers together — a web server, a database, a cache, a message queue. Managing all of these manually with `docker run` commands is error-prone:

```bash
# This is already painful with 3 containers
docker network create myapp
docker run -d --name postgres --network myapp -e POSTGRES_PASSWORD=secret postgres:16
docker run -d --name redis --network myapp redis:7-alpine
docker run -d --name app --network myapp -p 8080:3000 \
    -e DATABASE_URL=postgresql://postgres:secret@postgres:5432/mydb \
    -e REDIS_URL=redis://redis:6379 \
    myapp:1.0

# Starting, stopping, scaling, and updating these is a nightmare
# What if postgres starts after the app? The app crashes.
# What if you need to scale the app to 3 instances?
# What if you want to see logs from all services together?
```

**Next Module:** [07-docker-compose](../07-docker-compose/README.md) — Define and run multi-container applications with a single declarative file.
