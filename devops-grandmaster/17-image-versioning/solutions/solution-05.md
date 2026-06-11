# Solution 05: Design a Versioning and Release Strategy

## Part A: Tagging Strategy Design

### Tag Types Summary

| Tag Type | Format | Trigger | Mutable? | Created By | Primary Use |
|----------|--------|---------|----------|------------|-------------|
| Semantic version | `v1.2.3` | Git tag push (`v*`) | No | Release manager / CI | Production deployment, human communication |
| Git SHA (short) | `a1b2c3d` | Every CI build | No | CI pipeline | Traceability to source code |
| Git SHA (full) | `a1b2c3d4e5f6...` | Every CI build | No | CI pipeline | Signing, audit trails |
| Combined | `v1.2.3-a1b2c3d` | Git tag push | No | CI pipeline | Best of both: readable + traceable |
| Environment | `v1.2.3-staging`, `v1.2.3-production` | Promotion step | No | Promotion pipeline | Track which environment approved the image |
| Branch | `main`, `feature-auth` | Every CI build | Yes (overwrites) | CI pipeline | Development convenience only |
| `latest` | `latest` | Push to `main` | Yes (overwrites) | CI pipeline | Local development convenience |
| Date-based | `20240115`, `20240115-a1b2c3d` | Nightly builds | No | Scheduled CI | Nightly/scheduled builds |
| Base image | `node:18.19.0-alpine3.19@sha256:...` | Base image update | No (by convention) | Dependabot / Renovate | Dockerfile `FROM` line |

### Why This Works

Each tag type serves a distinct purpose. Semantic version tags communicate intent to humans. Git SHA tags provide traceability to source code. Combined tags give both. Environment tags track promotion. Branch tags and `latest` are convenience tags for development. Date tags handle scheduled builds. Base image tags with digests prevent silent drift.

### Common Mistakes

- **Using only one tag type.** Each type serves a different purpose. Use multiple tags per image.
- **Allowing `latest` in production.** `latest` is mutable and should be restricted to development.
- **Not pinning base images.** `FROM node:18-alpine` silently changes when new Alpine versions are released.

## Part B: Release Workflow Design

### Complete Flow

```
Developer                    CI Pipeline                    Release                    Promotion
---------                    -----------                    -------                    ---------
  |                             |                             |                          |
  |-- git push feature -------->|                             |                          |
  |                             |-- build + test              |                          |
  |                             |-- tag: feature-abc          |                          |
  |<-- status report -----------|                             |                          |
  |                             |                             |                          |
  |-- merge to main ----------->|                             |                          |
  |                             |-- build + test + scan       |                          |
  |                             |-- tag: main, <sha>          |                          |
  |                             |-- deploy to staging ------->|                          |
  |                             |                             |-- smoke tests            |
  |                             |                             |                          |
  |-- create git tag v1.2.3 --->|                             |                          |
  |                             |-- build + test + scan       |                          |
  |                             |-- tag: v1.2.3, <sha>        |                          |
  |                             |-- combined: v1.2.3-<sha>    |                          |
  |                             |-- deploy to staging ------->|                          |
  |                             |                             |-- integration tests      |
  |                             |                             |-- approve? ------------->|
  |                             |                             |                          |
  |                             |                             |        tag: v1.2.3-prod -|
  |                             |                             |        deploy prod ------>|
  |                             |                             |                          |-- health check
  |                             |                             |                          |
  |  HOTFIX: branch from v1.3  |                             |                          |
  |-- fix + git tag v1.3.1 --->|                             |                          |
  |                             |-- expedited pipeline ------>|                          |
  |                             |                             |                          |
```

### 1. Development Flow

- **Trigger:** Developer pushes to a feature branch.
- **CI steps:** Build image, run unit tests, run linting. Do NOT push to registry.
- **Tags created:** None (or branch-based tags for preview).
- **Environments affected:** None.
- **Approval gates:** None -- automated only.

### 2. CI Flow (Main Branch)

- **Trigger:** Merge to `main`.
- **CI steps:** Build image, run tests, run security scan, push to registry, deploy to staging.
- **Tags created:** `main`, `<short-sha>`, `<full-sha>`.
- **Environments affected:** Staging.
- **Approval gates:** None -- automatic deployment to staging.

### 3. Release Flow

- **Trigger:** Release manager creates a Git tag (`v1.2.3`).
- **CI steps:** Build image, run full test suite, security scan, sign image, push to registry.
- **Tags created:** `v1.2.3`, `<sha>`, `v1.2.3-<sha>`, `latest` (if this is the highest version).
- **Environments affected:** Staging (automatic), production (after approval).
- **Approval gates:** Manual approval required for production promotion.

### 4. Promotion Flow

- **Trigger:** Manual approval after staging verification.
- **Steps:** Pull image by digest, deploy to production, run health checks, record in deployment log.
- **Tags created:** `v1.2.3-production` (environment tag).
- **Environments affected:** Production.
- **Approval gates:** Required -- environment protection rules in GitHub.

### 5. Hotfix Flow

- **Trigger:** Critical bug found in production. Developer branches from the release tag.
- **CI steps:** Same as release flow, but expedited (shorter test suite for non-affected areas).
- **Tags created:** `v1.2.4` (patch increment), `<sha>`, `v1.2.4-<sha>`.
- **Environments affected:** Staging (automatic), production (expedited approval).
- **Approval gates:** Expedited -- single approver instead of full review.

### Why This Works

The flow ensures that every production deployment goes through staging first. Release tags trigger a separate pipeline with additional security gates. Hotfixes follow the same pipeline but with expedited approvals. The promotion step deploys the exact same image (by digest) that was tested in staging.

### Common Mistakes

- **Deploying directly to production.** Every image must go through staging first, even hotfixes.
- **Not using Git tags for releases.** Without Git tags, there is no trigger for the release pipeline.
- **Automated production deployment without approval.** Production should always have a human gate.

## Part C: Environment Promotion Strategy

### Promotion Workflow

```yaml
name: Promote to Production

on:
  workflow_dispatch:
    inputs:
      image-digest:
        required: true
        type: string
        description: The image digest to promote (from staging deployment)
      version:
        required: true
        type: string
        description: The semantic version (e.g., v1.2.3)

concurrency:
  group: production-deploy
  cancel-in-progress: false

jobs:
  promote:
    runs-on: ubuntu-latest
    environment:
      name: production  # Requires approval
    steps:
      - name: Verify image passed staging
        run: |
          # Check that this digest was deployed to staging
          if ! grep -q "${{ inputs.image-digest }}" deployments-staging.log; then
            echo "ERROR: This image was not deployed to staging"
            exit 1
          fi

      - name: Pull image by digest
        run: |
          docker pull ghcr.io/${{ github.repository }}@${{ inputs.image-digest }}

      - name: Tag for production
        run: |
          docker tag \
            ghcr.io/${{ github.repository }}@${{ inputs.image-digest }} \
            ghcr.io/${{ github.repository }}:${{ inputs.version }}-production

      - name: Deploy to production
        run: |
          export IMAGE_REF="ghcr.io/${{ github.repository }}@${{ inputs.image-digest }}"
          docker compose -f docker-compose.yml -f docker-compose.production.yml up -d

      - name: Health check
        run: |
          for i in $(seq 1 30); do
            if curl -sf http://localhost:3000/health > /dev/null 2>&1; then
              echo "Production is healthy"
              exit 0
            fi
            sleep 2
          done
          echo "ERROR: Production health check failed"
          exit 1

      - name: Record deployment
        run: |
          echo "{\"timestamp\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\",\"environment\":\"production\",\"digest\":\"${{ inputs.image-digest }}\",\"version\":\"${{ inputs.version }}\",\"deployer\":\"${{ github.actor }}\",\"status\":\"success\"}" >> deployments-production.log
```

### Key Principles

1. **Build once, deploy everywhere.** The image is built once in CI. Promotion deploys the same image (by digest) with different configuration.
2. **Digest-based promotion.** The promotion step references the image by digest, not tag. This guarantees the exact same bits that were tested in staging.
3. **Approval gate.** The `environment: name: production` setting in GitHub Actions requires approval from designated reviewers.
4. **Environment-specific configuration.** Use docker-compose override files or Kubernetes ConfigMaps for environment-specific settings (database URLs, API keys). The image itself is identical.
5. **Independent rollback.** Each environment has its own deployment log. Rolling back staging does not affect production.

### Why This Works

"Build once, deploy everywhere" eliminates the risk of rebuilding introducing differences between environments. Digest-based promotion guarantees that the production image is bit-for-bit identical to the staging image. The approval gate ensures a human reviews before production deployment. Environment-specific configuration is injected at deploy time, not baked into the image.

### Common Mistakes

- **Rebuilding for production.** If you rebuild, you might get different dependency versions, different timestamps, or different layers. Always promote the same image.
- **Using tags for promotion.** Tags can be reassigned. Use digests.
- **Baking environment config into the image.** This means you need a different image per environment, which defeats "build once."

## Part D: Base Image Management

### Base Image Policy

1. **Pin to a specific version and digest.** Production Dockerfiles must use `FROM node:18.19.0-alpine3.19@sha256:abc123...`. Floating tags like `node:18-alpine` are prohibited.

2. **Update base images weekly.** Use Dependabot or Renovate to propose base image updates on a weekly schedule. Updates go through the normal CI pipeline (build, test, scan).

3. **Expedite security patches.** When a CVE is found in a base image, the update bypasses the weekly schedule and enters an expedited pipeline with a 24-hour SLA.

4. **Test before merging.** Base image updates must pass the full test suite before being merged. A base image update is treated like any other code change.

5. **Document the base image.** Each Dockerfile must have a comment explaining why this specific base image was chosen (e.g., "Alpine for small image size, pinned to 3.19 for glibc compatibility").

6. **One base image per service.** Each service can choose its own base image, but the choice must be documented and approved by the team.

7. **Audit base images quarterly.** Review all base images for EOL status, known CVEs, and available updates. Remove unused base images from the registry.

8. **Use multi-stage builds.** Build dependencies in a full image, copy artifacts to a minimal runtime image. This reduces the attack surface.

### Why This Works

Pinning to a digest prevents silent base image drift. Weekly updates keep the base current without overwhelming the team with PRs. Expedited security patches address CVEs quickly. Testing before merging catches incompatibilities. Documentation ensures the team understands why specific base images were chosen.

### Common Mistakes

- **Pinning to a tag without a digest.** Tags can be reassigned. The digest is the only guarantee.
- **Never updating base images.** Outdated base images accumulate CVEs. Automate updates.
- **Updating base images without testing.** A base image update can change system libraries, OpenSSL versions, or glibc versions. Always test.

## Part E: Rollback and Disaster Recovery

### Scenario 1: Bad Application Code

**Detection:** Monitoring alerts (error rate spike, latency increase, health check failures). Automated health checks in the CI/CD pipeline.

**Rollback target:** The previous image digest from `deployments-production.log`. The second-to-last entry is the last known good deployment.

**Rollback procedure:**

```bash
./rollback.sh production
# Reads deployments-production.log
# Finds the second-to-last entry
# Pulls that image by digest
# Deploys it
# Runs health check
```

**Verification:** Health checks pass, error rates return to normal, monitoring confirms stability.

**Prevention:** Require staging deployment before production. Require health checks after every deployment. Maintain deployment history with digests.

### Scenario 2: Bad Base Image Update

**Detection:** Subtle failures that appear after a base image update -- library incompatibilities, changed behavior in system utilities, OpenSSL version mismatches.

**Rollback target:** The previous base image digest from the Dockerfile's git history. Use `git log` to find the previous `FROM` line.

**Rollback procedure:**

```bash
# Find the previous base image digest
git log --oneline --all -- Dockerfile | head -5
git show <previous-commit>:Dockerfile | grep FROM

# Update the Dockerfile to use the previous base image
# Rebuild and deploy through the normal pipeline
./version.sh patch  # Bump to 1.2.4
./build.sh          # Rebuild with previous base image
./deploy.sh myapp:v1.2.4 production
```

**Verification:** Application behaves as before the base image update. Run the full test suite against the rebuilt image.

**Prevention:** Always test base image updates through the full CI pipeline before merging. Use Dependabot to propose updates as PRs, which run the full test suite.

### Scenario 3: Registry Outage

**Detection:** `docker pull` fails with connection errors. CI pipeline fails at the push step.

**Rollback target:** Locally cached images on production servers. If the image is already pulled, the container can be restarted from the local cache.

**Rollback procedure:**

1. **If the image is already on the production server:** Restart the container from the local image. Do not attempt to pull.
2. **If the image is not on the production server:** Use a mirror registry (if configured). Pull from the mirror.
3. **If no mirror exists:** Use `docker save`/`docker load` to transfer images between servers.
4. **As a last resort:** Rebuild from source on a server with access to the source code.

```bash
# Check if the image is locally available
docker image inspect myapp@sha256:abc123 > /dev/null 2>&1 && echo "Available locally"

# Save an image to a tar file (do this proactively for critical images)
docker save myapp@sha256:abc123 -o myapp-abc123.tar

# Load an image from a tar file
docker load -i myapp-abc123.tar
```

**Verification:** Application starts and passes health checks using the local image.

**Prevention:**

- Configure a mirror registry (e.g., AWS ECR pull-through cache, Harbor mirror).
- Proactively pull and cache critical images on production servers.
- Use `docker save` to create offline backups of release images.
- Configure retry logic in CI pipelines for transient registry failures.

### Why This Works

Each scenario requires a different rollback strategy because the failure mode is different. Bad application code is fixed by deploying a previous image. Bad base images require rebuilding with the previous base. Registry outages require local caches or mirrors. The common thread is: always have a fallback, and always test the fallback.

### Common Mistakes

- **Treating all rollbacks the same.** Bad code, bad base images, and infrastructure failures require different strategies.
- **Not having a mirror registry.** A registry outage should not block all deployments.
- **Not caching images locally.** If the only copy of an image is in the registry, a registry outage makes rollback impossible.

## Part F: Audit and Compliance

### Audit Requirements and Data Sources

| Question | Data Source | How to Query |
|----------|-----------|-------------|
| What exact image (by digest) is running in production? | `deployments-production.log` | `tail -1 deployments-production.log \| jq -r '.digest'` |
| What code (by Git SHA) produced that image? | OCI label on the image | `docker inspect --format='{{index .Config.Labels "org.opencontainers.image.revision"}}' <image>` |
| When was it deployed, and by whom? | `deployments-production.log` | `tail -1 deployments-production.log \| jq -r '.timestamp, .deployer'` |
| What base image was used? | Dockerfile in the Git commit | `git show <sha>:Dockerfile \| grep FROM` |
| What vulnerabilities were known at deployment time? | CI/CD scan results (GitHub Actions logs, SARIF) | GitHub Security tab, workflow run artifacts |
| What was the previous production image? | `deployments-production.log` | `tail -2 deployments-production.log \| head -1 \| jq -r '.digest'` |

### Audit System Design

The audit system combines four data sources:

1. **Image labels (OCI):** Every image carries `org.opencontainers.image.version`, `.revision`, and `.created`. These are baked in at build time and cannot be changed after the fact.

2. **Deployment log:** Every deployment (and rollback) is recorded with timestamp, environment, digest, git SHA, deployer, and status. This is the authoritative record of what was deployed when.

3. **CI/CD logs:** Every build, test, scan, and deployment is recorded in GitHub Actions. SARIF scan results are stored in the GitHub Security tab.

4. **Registry metadata:** The registry stores image digests, tags, and (with some registries) vulnerability scan results.

### Audit Query Examples

```bash
# What is running in production right now?
LATEST=$(tail -1 deployments-production.log)
echo "Image: $(echo $LATEST | jq -r '.digest')"
echo "Deployed: $(echo $LATEST | jq -r '.timestamp')"
echo "By: $(echo $LATEST | jq -r '.deployer')"

# What code produced this image?
DIGEST=$(echo $LATEST | jq -r '.digest')
docker inspect --format='{{index .Config.Labels "org.opencontainers.image.revision"}}' \
  "myapp@${DIGEST}"

# What changed between the last two deployments?
CURRENT=$(tail -1 deployments-production.log | jq -r '.git_sha')
PREVIOUS=$(tail -2 deployments-production.log | head -1 | jq -r '.git_sha')
git log --oneline ${PREVIOUS}..${CURRENT}

# What base image was used?
CURRENT_SHA=$(tail -1 deployments-production.log | jq -r '.git_sha')
git show ${CURRENT_SHA}:Dockerfile | grep FROM
```

### Why This Works

No single data source answers all audit questions. The combination of image labels, deployment logs, CI logs, and registry metadata provides a complete audit trail. The deployment log is the authoritative record; image labels provide self-describing metadata; CI logs provide the build context; registry metadata provides the storage context.

### Common Mistakes

- **Relying on a single data source.** Deployment logs without image labels mean you cannot inspect the image independently. Image labels without deployment logs mean you know what was built but not what was deployed.
- **Not recording rollbacks.** Rollbacks must appear in the deployment log as a separate entry with `"status":"rollback"`.
- **Storing audit data only in the CI system.** CI logs can be deleted. Deployment logs should be stored independently (S3, database, or a separate log aggregation system).

## Key Takeaway

A versioning and release strategy is a system that connects code changes to running containers with full traceability. The tagging strategy defines what information is encoded in each tag. The release workflow defines how images move from commit to production. The promotion strategy ensures the same image runs in staging and production. Base image management prevents silent drift. Rollback strategies address different failure modes differently. The audit system combines multiple data sources to answer compliance questions. The entire strategy must be enforceable by automation -- policies that depend on human discipline will eventually be broken.
