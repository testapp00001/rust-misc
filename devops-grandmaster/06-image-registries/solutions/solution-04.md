# Solution 04: Implement a Tag Strategy for a CI/CD Pipeline

## Part 1: The Tagging Strategy (TAGGING-STRATEGY.md)

```markdown
# Acme Corp Image Tagging Strategy

## Tag Formats

### Development Builds (feature branches)
Format: `<user>/web-app:<branch>-<git-sha>`
Example: `acme/web-app:feature-add-login-abc1234`
Purpose: Identify builds from any branch. Never auto-deployed.

### Staging Builds (main branch)
Format: `<user>/web-app:main-<git-sha>` + `<user>/web-app:staging`
Example: `acme/web-app:main-abc1234`, `acme/web-app:staging`
Purpose: The SHA tag provides traceability. The `staging` tag is a
rolling reference that always points to the latest main-branch build.

### Production Releases (git tags)
Format: `<user>/web-app:<git-sha>` + `<user>/web-app:<semver>`
         + `<user>/web-app:<major.minor>` + `<user>/web-app:<major>`
Example: `acme/web-app:abc1234`, `acme/web-app:1.2.3`,
         `acme/web-app:1.2`, `acme/web-app:1`
Purpose: Full semantic versioning for production releases. Users can
pin at any precision level.

### The `latest` Tag
We do NOT use `latest`. It is ambiguous and dangerous in production.
Instead, `staging` serves the "give me the latest" role, but only for
the staging environment.

## Promotion Flow

1. Developer pushes to feature branch -> build with branch SHA tags
2. Feature branch merges to main -> build with main SHA + staging tag
3. Git tag created (e.g., v1.2.3) -> build with semver tags
4. staging promoted to production -> production tag created

## Traceability

Every image embeds `GIT_SHA` as an environment variable. Every tag
includes the git SHA (either as the primary tag or as part of the tag).
Given any deployed image, you can determine the exact commit.
```

## Part 2: The Build Script (build.sh)

```bash
#!/usr/bin/env bash
set -euo pipefail

# ─── Configuration ───────────────────────────────────────────────────
DOCKER_USER="${1:?Usage: $0 <dockerhub-username>}"
IMAGE="${DOCKER_USER}/web-app"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# ─── Detect Git State ────────────────────────────────────────────────
GIT_SHA=$(git rev-parse --short HEAD)
BRANCH=$(git rev-parse --abbrev-ref HEAD)
GIT_TAG=$(git describe --tags --exact-match 2>/dev/null || echo "")

echo "=== Acme Corp Build Pipeline ==="
echo "Image:     ${IMAGE}"
echo "Git SHA:   ${GIT_SHA}"
echo "Branch:    ${BRANCH}"
echo "Git Tag:   ${GIT_TAG:-none}"
echo ""

# ─── Build the Image ─────────────────────────────────────────────────
docker build \
  --build-arg APP_VERSION="${GIT_SHA}" \
  --build-arg GIT_SHA="${GIT_SHA}" \
  -t "${IMAGE}:${GIT_SHA}" \
  "${SCRIPT_DIR}"

echo ""
echo "Built: ${IMAGE}:${GIT_SHA}"

# ─── Tag Based on Context ────────────────────────────────────────────
TAGS_PUSHED=()

if [ -n "$GIT_TAG" ]; then
  # ── Release build (HEAD is a git tag) ──────────────────────────────
  # Strip leading 'v' if present: v1.2.3 -> 1.2.3
  VERSION="${GIT_TAG#v}"

  MAJOR=$(echo "$VERSION" | cut -d. -f1)
  MINOR=$(echo "$VERSION" | cut -d. -f1-2)

  docker tag "${IMAGE}:${GIT_SHA}" "${IMAGE}:${VERSION}"
  docker tag "${IMAGE}:${GIT_SHA}" "${IMAGE}:${MINOR}"
  docker tag "${IMAGE}:${GIT_SHA}" "${IMAGE}:${MAJOR}"

  TAGS_PUSHED+=("${GIT_SHA}" "${VERSION}" "${MINOR}" "${MAJOR}")

  echo "Release tags: ${VERSION}, ${MINOR}, ${MAJOR}"

elif [ "$BRANCH" = "main" ]; then
  # ── Staging build (on main branch) ────────────────────────────────
  docker tag "${IMAGE}:${GIT_SHA}" "${IMAGE}:main-${GIT_SHA}"
  docker tag "${IMAGE}:${GIT_SHA}" "${IMAGE}:staging"

  TAGS_PUSHED+=("main-${GIT_SHA}" "staging")

  echo "Staging tags: main-${GIT_SHA}, staging"

else
  # ── Feature branch build ──────────────────────────────────────────
  # Sanitize branch name: replace / and _ with -, lowercase
  SAFE_BRANCH=$(echo "$BRANCH" | tr '/_' '-' | tr '[:upper:]' '[:lower:]')
  docker tag "${IMAGE}:${GIT_SHA}" "${IMAGE}:${SAFE_BRANCH}-${GIT_SHA}"

  TAGS_PUSHED+=("${SAFE_BRANCH}-${GIT_SHA}")

  echo "Feature tags: ${SAFE_BRANCH}-${GIT_SHA}"
fi

# ─── Push All Tags ───────────────────────────────────────────────────
echo ""
echo "Pushing tags..."
for TAG in "${TAGS_PUSHED[@]}"; do
  echo "  ${IMAGE}:${TAG}"
  docker push "${IMAGE}:${TAG}"
done

echo ""
echo "=== Build Complete ==="
echo "All tags: ${TAGS_PUSHED[*]}"
```

### Why This Script Works

**Git state detection:** `git rev-parse --short HEAD` gives the short SHA
(7 characters). `git rev-parse --abbrev-ref HEAD` gives the branch name.
`git describe --tags --exact-match` returns the tag if HEAD is exactly a
tagged commit, or fails (caught by `|| echo ""`) if it is not.

**Three build paths:** The script has three distinct paths based on context:
- **Release (git tag):** Produces full semver tags (`1.2.3`, `1.2`, `1`).
- **Staging (main branch):** Produces `main-<sha>` and a rolling `staging` tag.
- **Feature branch:** Produces `<branch>-<sha>` tags. These are never
  auto-deployed.

**Branch name sanitization:** Git branch names can contain `/` and `_`
(e.g., `feature/add-login`). Docker tags do not allow `/`. The script replaces
these with `-` and lowercases the result.

**No `latest` tag:** The strategy explicitly avoids `latest`. The `staging`
tag serves the "give me the latest from main" role, but only for staging.

## Part 3: The Promote Script (promote.sh)

```bash
#!/usr/bin/env bash
set -euo pipefail

# ─── Configuration ───────────────────────────────────────────────────
DOCKER_USER="${1:?Usage: $0 <dockerhub-username> <source-tag> <target-tag>}"
SOURCE_TAG="${2:?Usage: $0 <dockerhub-username> <source-tag> <target-tag>}"
TARGET_TAG="${3:?Usage: $0 <dockerhub-username> <source-tag> <target-tag>}"
IMAGE="${DOCKER_USER}/web-app"

echo "=== Promoting Image ==="
echo "Source: ${IMAGE}:${SOURCE_TAG}"
echo "Target: ${IMAGE}:${TARGET_TAG}"
echo ""

# ─── Pull the source image ───────────────────────────────────────────
# This ensures we have the image locally and it matches the registry
docker pull "${IMAGE}:${SOURCE_TAG}"

# ─── Get the image ID ────────────────────────────────────────────────
IMAGE_ID=$(docker images --format "{{.ID}}" "${IMAGE}:${SOURCE_TAG}" | head -1)

if [ -z "$IMAGE_ID" ]; then
  echo "ERROR: Could not find image ${IMAGE}:${SOURCE_TAG}"
  exit 1
fi

echo "Image ID: ${IMAGE_ID}"

# ─── Tag for the target environment ──────────────────────────────────
docker tag "${IMAGE_ID}" "${IMAGE}:${TARGET_TAG}"

# ─── Push the new tag ────────────────────────────────────────────────
echo ""
echo "Pushing ${IMAGE}:${TARGET_TAG}..."
docker push "${IMAGE}:${TARGET_TAG}"

echo ""
echo "=== Promotion Complete ==="
echo "${IMAGE}:${SOURCE_TAG} -> ${IMAGE}:${TARGET_TAG}"
echo "Image ID: ${IMAGE_ID}"
```

### Why This Script Works

**No rebuild:** The promote script does not rebuild the image. It finds the
image ID that the source tag points to, then creates a new tag pointing to the
same image ID. Both tags now reference the same manifest. When pushed, the
registry already has all the layers, so only the tag metadata is transferred.

**Image ID lookup:** `docker images --format "{{.ID}}"` returns the image ID
for a given repository and tag. This is a short hash (e.g., `abc123def456`)
that uniquely identifies the image.

**`docker pull` before tagging:** This ensures the local Docker daemon has the
image. Without this, `docker images` might return nothing if the image was
previously removed locally.

## Part 4: Testing the Full Workflow

```bash
# Set up
mkdir -p /tmp/cicd-tagging && cd /tmp/cicd-tagging
git init

cat > app.sh << 'EOF'
#!/bin/bash
echo "Acme Web App v${APP_VERSION:-unknown} (commit: ${GIT_SHA:-unknown})"
EOF
chmod +x app.sh

cat > Dockerfile << 'EOF'
FROM alpine:3.19
COPY app.sh /app.sh
ARG APP_VERSION=0.0.0
ARG GIT_SHA=unknown
ENV APP_VERSION=${APP_VERSION}
ENV GIT_SHA=${GIT_SHA}
CMD ["/app.sh"]
EOF

git add .
git commit -m "Initial commit"
```

### Simulation

```bash
# 1. Developer pushes to feature branch
git checkout -b feature/add-login
echo "login feature" >> features.txt
git add . && git commit -m "Add login feature"
./build.sh yourusername
# Produces: yourusername/web-app:feature-add-login-abc1234

# 2. Merge to main
git checkout main
git merge feature/add-login
./build.sh yourusername
# Produces: yourusername/web-app:main-def5678
#           yourusername/web-app:staging

# 3. Verify staging
docker pull yourusername/web-app:staging
docker run --rm yourusername/web-app:staging
# Output: Acme Web App vdef5678 (commit: def5678)

# 4. Create a release
git tag v1.0.0
./build.sh yourusername
# Produces: yourusername/web-app:ghi9012
#           yourusername/web-app:1.0.0
#           yourusername/web-app:1.0
#           yourusername/web-app:1

# 5. Promote to production
./promote.sh yourusername staging production
# Re-tags staging as production without rebuilding

# 6. Verify production
docker pull yourusername/web-app:production
docker run --rm yourusername/web-app:production
# Output: Acme Web App vghi9012 (commit: ghi9012)
```

### Verify Traceability

```bash
# Given a production image, find the source commit
docker inspect yourusername/web-app:production \
  --format '{{range .Config.Env}}{{println .}}{{end}}' | grep GIT_SHA
# GIT_SHA=ghi9012

# Now look up the commit
git log --oneline ghi9012
# ghi9012 Merge branch 'feature/add-login'
```

Every production image is traceable to an exact Git commit.

---

## Common Mistakes

1. **Using `latest` for staging.** The script uses `staging` instead, which
   is explicit about its purpose. `latest` has no semantic meaning and
   causes confusion when multiple environments exist.

2. **Not sanitizing branch names.** Docker tags do not allow `/`. A branch
   named `feature/add-login` must be converted to `feature-add-login`.
   Forgetting this produces an "invalid reference format" error.

3. **Rebuilding during promotion.** The promote script must NOT rebuild. The
   whole point of promotion is to deploy the exact same image that was tested
   in staging. Rebuilding could introduce differences (updated packages, new
   timestamps) that were not tested.

4. **Embedding the version only in the tag.** Tags can be changed. The script
   embeds `GIT_SHA` as an environment variable inside the image itself. Even
   if the tag is deleted or re-pointed, the image still carries its origin
   information.

5. **Pushing semver tags on every commit.** The script only creates semver
   tags (`1.2.3`, `1.2`, `1`) when HEAD is a git tag. On regular commits,
   it creates branch-based or SHA-based tags. This prevents tag pollution
   and ensures semver tags only exist for intentional releases.

6. **Forgetting to update the `staging` tag.** The script always re-tags and
   pushes `staging` on main-branch builds. This ensures `staging` always
   points to the latest main build, which is the expected behavior for a
   staging environment.
