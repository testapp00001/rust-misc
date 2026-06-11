# Exercise 04: Implement a Tag Strategy for a CI/CD Pipeline

**Type:** Challenge
**Time:** 45 minutes
**Difficulty:** Medium-Hard

---

## Objective

Design and implement a tagging strategy for a CI/CD pipeline that supports staging and production environments. Write a shell script that automates building, tagging, and pushing images with proper version management.

---

## Prerequisites

- Docker installed and running.
- A Docker Hub account (or access to a private registry).
- Basic shell scripting knowledge.
- `git` installed.

---

## The Scenario

You are building the CI/CD pipeline for a company called Acme Corp. They have:

- A single application called `web-app`.
- Two environments: `staging` and `production`.
- Multiple developers pushing code daily.
- A requirement that every image in production must be traceable to an exact Git commit.
- A requirement that staging always runs the latest code from the `main` branch.
- A requirement that production only runs images that have been explicitly promoted from staging.

---

## Instructions

### Step 1: Set Up a Sample Project

```bash
# Create the project
mkdir -p /tmp/cicd-tagging && cd /tmp/cicd-tagging
git init

# Create a simple application
cat > app.sh << 'EOF'
#!/bin/bash
echo "Acme Web App v${APP_VERSION:-unknown} (commit: ${GIT_SHA:-unknown})"
EOF
chmod +x app.sh

# Create a Dockerfile
cat > Dockerfile << 'EOF'
FROM alpine:3.19
COPY app.sh /app.sh
ARG APP_VERSION=0.0.0
ARG GIT_SHA=unknown
ENV APP_VERSION=${APP_VERSION}
ENV GIT_SHA=${GIT_SHA}
CMD ["/app.sh"]
EOF

# Make an initial commit
git add .
git commit -m "Initial commit"
```

### Step 2: Design the Tag Strategy

Before writing any code, answer these questions:

1. What tag format will you use for images built from the `main` branch that are destined for staging?
2. What tag format will you use for images promoted to production?
3. How will you ensure every production image is traceable to a specific Git commit?
4. Will you use the `latest` tag? If so, where? If not, why not?

Write your answers in a file called `TAGGING-STRATEGY.md`.

### Step 3: Implement the Build Script

Create a script called `build.sh` that:

1. Accepts a Docker Hub username as an argument.
2. Detects the current Git SHA and branch name.
3. Builds the Docker image with the Git SHA embedded.
4. Tags the image according to your strategy.
5. Pushes all tags to the registry.

The script should handle these cases:
- Building from the `main` branch (staging candidate).
- Building from a feature branch (development/testing).
- Building from a Git tag (release candidate).

```bash
# Expected usage:
./build.sh yourusername

# On main branch, it should produce tags like:
#   yourusername/web-app:main-abc1234
#   yourusername/web-app:staging (updated to point to this build)

# On a git tag (v1.2.3), it should produce tags like:
#   yourusername/web-app:abc1234
#   yourusername/web-app:1.2.3
#   yourusername/web-app:1.2
#   yourusername/web-app:1
```

### Step 4: Implement the Promote Script

Create a script called `promote.sh` that:

1. Takes a source tag and a target environment as arguments.
2. Re-tags the staging image for production.
3. Pushes the production tag.

```bash
# Expected usage:
# Promote the current staging image to production
./promote.sh yourusername staging production

# This should:
# 1. Find the image currently tagged as yourusername/web-app:staging
# 2. Tag it as yourusername/web-app:production
# 3. Push the production tag
```

### Step 5: Test the Full Workflow

Simulate the complete CI/CD workflow:

```bash
# 1. Developer pushes to main
git checkout -b feature/add-login
echo "login feature" >> features.txt
git add . && git commit -m "Add login feature"
./build.sh yourusername

# 2. Merge to main
git checkout main
git merge feature/add-login
./build.sh yourusername

# 3. Verify staging has the latest
docker pull yourusername/web-app:staging
docker run --rm yourusername/web-app:staging

# 4. Create a release
git tag v1.0.0
./build.sh yourusername

# 5. Promote to production
./promote.sh yourusername staging production

# 6. Verify production
docker pull yourusername/web-app:production
docker run --rm yourusername/web-app:production
```

---

## Success Criteria

- [ ] You wrote a `TAGGING-STRATEGY.md` that clearly defines tag formats.
- [ ] `build.sh` correctly detects Git SHA and branch.
- [ ] `build.sh` produces different tags based on the branch or Git tag.
- [ ] `build.sh` embeds the Git SHA in the image as an environment variable.
- [ ] `promote.sh` re-tags an existing image without rebuilding.
- [ ] Every image is traceable to a specific Git commit.
- [ ] The `staging` tag always points to the latest `main` build.
- [ ] The `production` tag only changes when explicitly promoted.

---

## Hints

<details>
<summary>Hint 1: Detecting Git Information</summary>

```bash
GIT_SHA=$(git rev-parse --short HEAD)
BRANCH=$(git rev-parse --abbrev-ref HEAD)
GIT_TAG=$(git describe --tags --exact-match 2>/dev/null || echo "")

# Check if HEAD is a tagged commit
if [ -n "$GIT_TAG" ]; then
  echo "Building release: $GIT_TAG"
else
  echo "Building branch: $BRANCH"
fi
```

</details>

<details>
<summary>Hint 2: Semantic Version Parsing</summary>

To create major and minor version aliases from a semver tag like `v1.2.3`:

```bash
VERSION="1.2.3"
MAJOR=$(echo $VERSION | cut -d. -f1)
MINOR=$(echo $VERSION | cut -d. -f1-2)

# Now you have: 1.2.3, 1.2, 1
```

</details>

<details>
<summary>Hint 3: Re-tagging Without Rebuilding</summary>

The `promote.sh` script does not need to rebuild the image. It just needs to:

```bash
# Find the image ID that staging points to
IMAGE_ID=$(docker images --format "{{.ID}}" yourusername/web-app:staging)

# Tag it for production
docker tag $IMAGE_ID yourusername/web-app:production

# Push
docker push yourusername/web-app:production
```

</details>

<details>
<summary>Hint 4: The `latest` Tag Decision</summary>

Many teams avoid `latest` entirely because it is ambiguous. If you use it, consider these rules:
- Only update `latest` when pushing a release (Git tag), never on every commit.
- Never use `latest` in production deployment manifests.
- Document clearly what `latest` points to and when it changes.

</details>
