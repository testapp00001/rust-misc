# Solution 04: Implement Rollback Strategy with Image Digests

## Part A: Understand Image Digests

### 1. Digest vs Tag

An image digest is a sha256 hash of the image manifest -- the JSON document that describes the image's layers, configuration, and metadata. It is content-addressable: the same content always produces the same digest. A tag is a human-readable string that points to a digest. Tags are mutable (they can be reassigned); digests are immutable (they are determined by the content).

### 2. Pushing the same tag twice

The tag moves to the new image. The old image still exists in the registry (identified by its digest), but the tag no longer points to it. The new image gets a new digest. The old image's digest remains unchanged -- it still identifies the old content.

### 3. Pull by digest

```bash
docker pull myapp@sha256:abc123def456789...
```

The `@sha256:...` syntax tells Docker to pull by digest instead of tag. Docker will refuse to pull if no image with that digest exists in the registry.

### 4. Digest collisions

No. SHA-256 produces a 256-bit hash. The probability of two different images producing the same digest is approximately 1 in 2^128 (due to the birthday problem), which is effectively zero. This is why digests are considered cryptographically unique identifiers.

### 5. Find the digest of a running container

```bash
# Get the image ID of a running container
docker inspect --format='{{.Image}}' <container-name>

# Get the repo digest (registry digest)
docker inspect --format='{{index .RepoDigests 0}}' <image-name>

# For a running container, combine both
docker inspect --format='{{.Image}}' <container-name> | xargs docker inspect --format='{{index .RepoDigests 0}}'
```

### Why This Works

Digests are the foundation of reliable rollback. Because they are content-addressable and collision-resistant, deploying by digest guarantees you get the exact image you expect -- regardless of what tags point to.

### Common Mistakes

- **Confusing image ID with digest.** The image ID is a local hash; the digest is the registry's content hash. They are different values.
- **Not pulling by digest when rollback is critical.** Tags can be reassigned. Digests cannot.

## Part B: Create a Deployment History Tracker

### deploy.sh

```bash
#!/bin/bash
set -e

IMAGE_REF="${1:?Usage: deploy.sh <image-ref> <environment>}"
ENVIRONMENT="${2:?Usage: deploy.sh <image-ref> <environment>}"
LOG_FILE="deployments-${ENVIRONMENT}.log"
HEALTH_URL="${3:-http://localhost:3000/health}"

echo "========================================="
echo "Deploying to ${ENVIRONMENT}"
echo "  Image: ${IMAGE_REF}"
echo "========================================="

# Pull the image
docker pull "${IMAGE_REF}"

# Resolve the digest
if [[ "$IMAGE_REF" == *@sha256:* ]]; then
  DIGEST="${IMAGE_REF#*@}"
  RESOLVED_REF="${IMAGE_REF}"
else
  RESOLVED_REF=$(docker inspect --format='{{index .RepoDigests 0}}' "${IMAGE_REF}" 2>/dev/null || echo "${IMAGE_REF}")
  DIGEST=$(echo "$RESOLVED_REF" | grep -o 'sha256:[a-f0-9]*' || echo "unknown")
fi

# Extract Git SHA from OCI labels
GIT_SHA=$(docker inspect --format='{{index .Config.Labels "org.opencontainers.image.revision"}}' "${IMAGE_REF}" 2>/dev/null || echo "unknown")

# Deploy
export APP_VERSION="${IMAGE_REF}"
docker compose -f docker-compose.yml up -d

# Health check
echo "Waiting for health check..."
HEALTHY=false
for i in $(seq 1 30); do
  if curl -sf "${HEALTH_URL}" > /dev/null 2>&1; then
    HEALTHY=true
    break
  fi
  sleep 2
done

if [ "$HEALTHY" = false ]; then
  echo "ERROR: Health check failed after 60 seconds"
  exit 1
fi

# Record deployment
TIMESTAMP=$(date -u +%Y-%m-%dT%H:%M:%SZ)
DEPLOYER=$(whoami)

ENTRY=$(cat <<EOF
{"timestamp":"${TIMESTAMP}","environment":"${ENVIRONMENT}","image_ref":"${IMAGE_REF}","digest":"${DIGEST}","git_sha":"${GIT_SHA}","deployer":"${DEPLOYER}","status":"success"}
EOF
)

echo "$ENTRY" >> "$LOG_FILE"

# Trim to last 20 entries
if [ -f "$LOG_FILE" ]; then
  LINES=$(wc -l < "$LOG_FILE")
  if [ "$LINES" -gt 20 ]; then
    tail -20 "$LOG_FILE" > "${LOG_FILE}.tmp"
    mv "${LOG_FILE}.tmp" "$LOG_FILE"
  fi
fi

echo ""
echo "Deployment recorded:"
echo "  Timestamp: ${TIMESTAMP}"
echo "  Digest:    ${DIGEST}"
echo "  Git SHA:   ${GIT_SHA}"
echo "  Deployer:  ${DEPLOYER}"
echo "  Log:       ${LOG_FILE}"
```

### Why This Works

The script resolves every image reference to its digest, whether the input is a tag or a digest. This ensures the deployment log always records the actual content that was deployed, not just the tag that was requested. The health check runs before recording the deployment, so only successful deployments appear in the log. The log is trimmed to 20 entries to prevent unbounded growth. JSON lines format allows easy parsing with `jq`.

### Common Mistakes

- **Recording the tag instead of the digest.** Tags can change; digests cannot. Always record the digest.
- **Not running the health check before recording.** A failed deployment should not appear in the history as "success."
- **Using a non-structured log format.** JSON lines can be parsed programmatically; plain text cannot.

## Part C: Create a Rollback Script

### rollback.sh

```bash
#!/bin/bash
set -e

ENVIRONMENT="${1:?Usage: rollback.sh <environment> [target]}"
TARGET="${2}"

IMAGE_NAME="myapp"
LOG_FILE="deployments-${ENVIRONMENT}.log"

echo "========================================="
echo "ROLLBACK: ${ENVIRONMENT}"
echo "========================================="

# Determine the rollback target
if [ -n "$TARGET" ]; then
  # Target specified -- could be a digest or a tag
  if [[ "$TARGET" == sha256:* ]]; then
    # It is a digest
    IMAGE_REF="${IMAGE_NAME}@${TARGET}"
  elif [[ "$TARGET" == *@sha256:* ]]; then # It is a full digest reference
    IMAGE_REF="$TARGET"
  else
    # It is a tag -- resolve to digest for safety
    echo "Resolving tag '${TARGET}' to digest..."
    docker pull "${IMAGE_NAME}:${TARGET}"
    RESOLVED=$(docker inspect --format='{{index .RepoDigests 0}}' "${IMAGE_NAME}:${TARGET}" 2>/dev/null)
    if [ -z "$RESOLVED" ]; then
      echo "ERROR: Could not resolve tag '${TARGET}' to a digest"
      exit 1
    fi
    IMAGE_REF="$RESOLVED"
    echo "  Resolved: ${IMAGE_REF}"
  fi
else
  # No target -- roll back to previous deployment
  if [ ! -f "$LOG_FILE" ]; then
    echo "ERROR: No deployment log found for ${ENVIRONMENT}"
    exit 1
  fi

  # Get the second-to-last successful deployment
  PREVIOUS=$(tail -2 "$LOG_FILE" | head -1 | jq -r '.digest')
  if [ -z "$PREVIOUS" ] || [ "$PREVIOUS" = "null" ]; then
    echo "ERROR: No previous deployment found to roll back to"
    exit 1
  fi

  IMAGE_REF="${IMAGE_NAME}@${PREVIOUS}"
  echo "Rolling back to previous deployment: ${PREVIOUS}"
fi

# Verify image exists
if ! docker pull "${IMAGE_REF}" 2>/dev/null; then
  echo "ERROR: Image ${IMAGE_REF} not found in registry"
  exit 1
fi

# Get the currently running image for the summary
CURRENT_DIGEST=$(docker inspect --format='{{.Image}}' "${IMAGE_NAME}" 2>/dev/null || echo "unknown")

# Deploy the rollback
export APP_VERSION="${IMAGE_REF}"
docker compose -f docker-compose.yml up -d

# Health check
echo "Verifying rollback..."
HEALTHY=false
for i in $(seq 1 30); do
  if curl -sf http://localhost:3000/health > /dev/null 2>&1; then
    HEALTHY=true
    break
  fi
  sleep 2
done

if [ "$HEALTHY" = false ]; then
  echo "WARNING: Health check failed after rollback"
  echo "The rollback was deployed but may not be healthy"
fi

# Record the rollback
TIMESTAMP=$(date -u +%Y-%m-%dT%H:%M:%SZ)
DIGEST=$(echo "$IMAGE_REF" | grep -o 'sha256:[a-f0-9]*' || echo "unknown")
GIT_SHA=$(docker inspect --format='{{index .Config.Labels "org.opencontainers.image.revision"}}' "${IMAGE_REF}" 2>/dev/null || echo "unknown")

ENTRY=$(cat <<EOF
{"timestamp":"${TIMESTAMP}","environment":"${ENVIRONMENT}","image_ref":"${IMAGE_REF}","digest":"${DIGEST}","git_sha":"${GIT_SHA}","deployer":"$(whoami)","status":"rollback","previous_digest":"${CURRENT_DIGEST}"}
EOF
)

echo "$ENTRY" >> "$LOG_FILE"

echo ""
echo "========================================="
echo "ROLLBACK SUMMARY"
echo "========================================="
echo "  Environment:  ${ENVIRONMENT}"
echo "  Previous:     ${CURRENT_DIGEST}"
echo "  Rolled back:  ${DIGEST}"
echo "  Health check: $([ "$HEALTHY" = true ] && echo "PASSED" || echo "FAILED")"
echo "========================================="
```

### Why This Works

The script handles three types of target input: a raw digest (`sha256:abc123`), a full digest reference (`myapp@sha256:abc123`), or a tag (`v1.2.3`). When a tag is provided, it is resolved to a digest before deploying -- this catches cases where a tag has been tampered with or reassigned. When no target is specified, the script reads the deployment log and uses the second-to-last entry as the rollback target. The rollback is recorded as a new entry with `"status":"rollback"` and a reference to the previous digest, creating a clear audit trail.

### Common Mistakes

- **Deploying by tag without resolving to digest.** If the tag was reassigned, you deploy the wrong image.
- **Not recording the rollback.** Rollbacks must appear in the deployment history for audit purposes.
- **Rolling back again if the rollback fails.** This can create an infinite rollback loop. Fail loudly and let a human decide.

## Part D: Add Digest Pinning to Docker Compose

### docker-compose.yml

```yaml
services:
  app:
    image: ${IMAGE_REF:?IMAGE_REF is required (tag or digest, e.g., myapp:v1.2.3 or myapp@sha256:abc...)}
    ports:
      - "${APP_PORT:-3000}:3000"
    environment:
      - NODE_ENV=${NODE_ENV:-production}
    healthcheck:
      test: ["CMD", "wget", "-qO-", "http://localhost:3000/health"]
      interval: 10s
      timeout: 5s
      retries: 3
    restart: unless-stopped
```

### Usage

```bash
# Deploy by tag
IMAGE_REF=myapp:v1.2.3 docker compose up -d

# Deploy by digest
IMAGE_REF=myapp@sha256:abc123def456789 docker compose up -d

# Deploy from the rollback script (which always resolves to digest)
./rollback.sh production
```

### Why This Works

Docker natively supports both tag-based and digest-based image references. The `${IMAGE_REF:?...}` syntax enforces that the variable is set, preventing accidental deployment of an undefined image. The compose file is intentionally simple -- the complexity lives in the deploy and rollback scripts, not in the compose file.

### Common Mistakes

- **Using `image: myapp:latest` in the compose file.** This defeats the purpose of versioning.
- **Not using the `:?` syntax.** Without it, docker-compose uses an empty string and produces a confusing error.
- **Hardcoding the image reference.** Always use a variable so the same compose file works for any version.

## Part E: Write a CI/CD Rollback Workflow

### .github/workflows/rollback.yml

```yaml
name: Rollback

on:
  workflow_dispatch:
    inputs:
      environment:
        required: true
        type: choice
        description: Target environment
        options:
          - staging
          - production
      target:
        required: true
        type: choice
        description: Rollback target
        options:
          - previous
          - specific-digest
          - specific-tag
      digest:
        required: false
        type: string
        description: Image digest (required if target is specific-digest)
      tag:
        required: false
        type: string
        description: Image tag (required if target is specific-tag)

concurrency:
  group: rollback-${{ inputs.environment }}
  cancel-in-progress: false

jobs:
  rollback:
    runs-on: ubuntu-latest
    environment:
      name: ${{ inputs.environment }}
    permissions:
      contents: read
      packages: read
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Validate inputs
        run: |
          if [ "${{ inputs.target }}" = "specific-digest" ] && [ -z "${{ inputs.digest }}" ]; then
            echo "ERROR: digest is required when target is specific-digest"
            exit 1
          fi
          if [ "${{ inputs.target }}" = "specific-tag" ] && [ -z "${{ inputs.tag }}" ]; then
            echo "ERROR: tag is required when target is specific-tag"
            exit 1
          fi

      - name: Determine rollback target
        id: target
        run: |
          IMAGE_NAME="ghcr.io/${{ github.repository }}"

          case "${{ inputs.target }}" in
            previous)
              # Read from deployment history (stored as artifact or in a known location)
              if [ -f "deployments-${{ inputs.environment }}.log" ]; then
                DIGEST=$(tail -2 "deployments-${{ inputs.environment }}.log" | head -1 | jq -r '.digest')
                echo "ref=${IMAGE_NAME}@${DIGEST}" >> $GITHUB_OUTPUT
                echo "digest=${DIGEST}" >> $GITHUB_OUTPUT
              else
                echo "ERROR: No deployment history found"
                exit 1
              fi
              ;;
            specific-digest)
              echo "ref=${IMAGE_NAME}@${{ inputs.digest }}" >> $GITHUB_OUTPUT
              echo "digest=${{ inputs.digest }}" >> $GITHUB_OUTPUT
              ;;
            specific-tag)
              # Resolve tag to digest
              docker pull "${IMAGE_NAME}:${{ inputs.tag }}"
              DIGEST=$(docker inspect --format='{{index .RepoDigests 0}}' "${IMAGE_NAME}:${{ inputs.tag }}" | grep -o 'sha256:[a-f0-9]*')
              echo "ref=${IMAGE_NAME}@${DIGEST}" >> $GITHUB_OUTPUT
              echo "digest=${DIGEST}" >> $GITHUB_OUTPUT
              ;;
          esac

      - name: Validate image exists
        run: |
          echo "Validating image: ${{ steps.target.outputs.ref }}"
          docker manifest inspect "${{ steps.target.outputs.ref }}" || {
            echo "ERROR: Image not found in registry"
            exit 1
          }

      - name: Deploy rollback
        run: |
          export IMAGE_REF="${{ steps.target.outputs.ref }}"
          export APP_PORT=$([ "${{ inputs.environment }}" = "staging" ] && echo "3001" || echo "3000")
          docker compose up -d

      - name: Health check
        id: health
        run: |
          APP_PORT=$([ "${{ inputs.environment }}" = "staging" ] && echo "3001" || echo "3000")
          for i in $(seq 1 30); do
            if curl -sf "http://localhost:${APP_PORT}/health" > /dev/null 2>&1; then
              echo "healthy=true" >> $GITHUB_OUTPUT
              exit 0
            fi
            sleep 2
          done
          echo "healthy=false" >> $GITHUB_OUTPUT

      - name: Generate rollback summary
        if: always()
        run: |
          cat >> $GITHUB_STEP_SUMMARY << EOF
          ## Rollback Summary

          | Property | Value |
          |----------|-------|
          | **Environment** | \`${{ inputs.environment }}\` |
          | **Target type** | \`${{ inputs.target }}\` |
          | **Image digest** | \`${{ steps.target.outputs.digest }}\` |
          | **Health check** | \`${{ steps.health.outputs.healthy }}\` |
          | **Triggered by** | @${{ github.actor }} |
          | **Workflow run** | [View logs](${{ github.server_url }}/${{ github.repository }}/actions/runs/${{ github.run_id }}) |
          EOF
```

### Why This Works

The workflow uses `workflow_dispatch` with structured inputs: environment, target type, and optional digest/tag. The `concurrency:` block prevents simultaneous rollbacks to the same environment -- if a rollback is already in progress, the new one is queued (not cancelled). The image is validated with `docker manifest inspect` before deploying. Health checks run for 60 seconds. The summary records all details for audit purposes. The workflow does NOT attempt a second rollback if the health check fails -- this prevents rollback loops.

### Common Mistakes

- **Allowing concurrent rollbacks.** Two simultaneous rollbacks to the same environment create race conditions. Use `concurrency:`.
- **Auto-rolling back on failed health check.** This creates a rollback loop. Fail loudly and let a human decide.
- **Not validating the image before deploying.** A rollback to a non-existent image wastes time and creates confusion.

## Key Takeaway

Digests are the only reliable rollback target because they are content-addressable and immutable. Tags can be reassigned; digests cannot. A deployment history that records digests creates an audit trail and makes rollback targets deterministic. The rollback workflow must validate images before deploying, check health after deploying, and avoid rollback loops. Docker-compose supports digest-based references natively -- use `${IMAGE_REF:?...}` to enforce version pinning at deploy time.
