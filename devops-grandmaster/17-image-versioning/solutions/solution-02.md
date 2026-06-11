# Solution 02: Implement Semantic Versioning for Docker Images

## Part A: Version Management Script

### version.sh

```bash
#!/bin/bash
set -e

ACTION="${1}"
VERSION_FILE="VERSION"

# Read current version
if [ -f "$VERSION_FILE" ]; then
  CURRENT=$(cat "$VERSION_FILE")
else
  CURRENT="0.0.0"
fi

# Parse version components
IFS='.' read -r MAJOR MINOR PATCH <<< "$CURRENT"

# Validate that components are numbers
if ! [[ "$MAJOR" =~ ^[0-9]+$ ]] || ! [[ "$MINOR" =~ ^[0-9]+$ ]] || ! [[ "$PATCH" =~ ^[0-9]+$ ]]; then
  echo "ERROR: Invalid version format in $VERSION_FILE: $CURRENT"
  exit 1
fi

case "$ACTION" in
  major)
    MAJOR=$((MAJOR + 1))
    MINOR=0
    PATCH=0
    ;;
  minor)
    MINOR=$((MINOR + 1))
    PATCH=0
    ;;
  patch)
    PATCH=$((PATCH + 1))
    ;;
  show)
    echo "Current version: v${CURRENT}"
    exit 0
    ;;
  set)
    if [ -z "$2" ]; then
      echo "Usage: $0 set <version>"
      exit 1
    fi
    # Validate version format
    if ! [[ "$2" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
      echo "ERROR: Invalid version format: $2 (expected N.N.N)"
      exit 1
    fi
    echo "$2" > "$VERSION_FILE"
    echo "Version set to v$2"
    exit 0
    ;;
  *)
    echo "Usage: $0 {major|minor|patch|show|set <version>}"
    exit 1
    ;;
esac

NEW_VERSION="${MAJOR}.${MINOR}.${PATCH}"
echo "$NEW_VERSION" > "$VERSION_FILE"
echo "Version bumped: v${CURRENT} -> v${NEW_VERSION}"
```

### Why This Works

The script uses `IFS='.'` to split the version string into its three components, which is the standard way to parse delimited strings in bash. The `case` statement dispatches on the action. For `major`, it increments the major number and resets minor and patch to 0. For `minor`, it increments minor and resets patch. For `patch`, it only increments patch. The `set` command validates the input with a regex before writing. The version is persisted to a `VERSION` file, which is the single source of truth.

### Common Mistakes

- **Not resetting lower components.** If `minor` bump does not reset `patch`, you get `1.1.3` instead of `1.1.0`.
- **Not validating `set` input.** Without validation, `./version.sh set foo` writes garbage to the VERSION file.
- **Using `#!/bin/sh` instead of `#!/bin/bash`.** The `read -r` and `[[ ]]` syntax requires bash.

## Part B: Versioned Dockerfile

### server.js

```javascript
const http = require('http');

const APP_VERSION = process.env.APP_VERSION || 'unknown';
const GIT_SHA = process.env.GIT_SHA || 'unknown';
const BUILD_DATE = process.env.BUILD_DATE || 'unknown';

const server = http.createServer((req, res) => {
  if (req.url === '/health') {
    res.writeHead(200, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({ status: 'healthy', version: APP_VERSION }));
    return;
  }

  res.writeHead(200, { 'Content-Type': 'application/json' });
  res.end(JSON.stringify({
    name: 'Versioning Demo',
    version: APP_VERSION,
    gitSha: GIT_SHA,
    buildDate: BUILD_DATE,
    uptime: process.uptime(),
  }));
});

const port = process.env.PORT || 3000;
server.listen(port, () => {
  console.log(`v${APP_VERSION} (${GIT_SHA}) listening on port ${port}`);
});
```

### Dockerfile

```dockerfile
# Pin the base image to a specific version
FROM node:18.19.0-alpine3.19

# Build arguments for version metadata
ARG APP_VERSION=unknown
ARG GIT_SHA=unknown
ARG BUILD_DATE=unknown

# Inject as environment variables (available at runtime)
ENV APP_VERSION=${APP_VERSION}
ENV GIT_SHA=${GIT_SHA}
ENV BUILD_DATE=${BUILD_DATE}

WORKDIR /app

# Copy dependency files first (cache layer)
COPY package*.json ./
RUN npm ci --only=production

# Copy application code
COPY server.js .

# OCI-compliant labels
LABEL org.opencontainers.image.version="${APP_VERSION}"
LABEL org.opencontainers.image.revision="${GIT_SHA}"
LABEL org.opencontainers.image.created="${BUILD_DATE}"

# Run as non-root user
USER node

EXPOSE 3000
CMD ["node", "server.js"]
```

### Why This Works

The Dockerfile uses `ARG` for build-time variables and `ENV` to make them available at runtime. This distinction is critical: `ARG` values are only available during `docker build`, but `ENV` values persist into the running container. The OCI labels embed metadata directly in the image, making it self-describing. The `USER node` directive ensures the container does not run as root. The base image is pinned to `node:18.19.0-alpine3.19` instead of `node:18-alpine`, preventing silent base image drift.

### Common Mistakes

- **Using `ARG` without `ENV`.** `ARG` values are not available at runtime. You need both.
- **Using a floating base image tag.** `FROM node:18-alpine` changes when new Alpine versions are released.
- **Running as root.** Always use `USER node` or `USER 1000` for Node.js applications.

## Part C: Build Script

### build.sh

```bash
#!/bin/bash
set -e

IMAGE_NAME="myapp"

# Get version: argument overrides VERSION file
if [ -n "$1" ]; then
  APP_VERSION="$1"
elif [ -f VERSION ]; then
  APP_VERSION=$(cat VERSION)
else
  echo "ERROR: No version argument and no VERSION file found"
  exit 1
fi

# Ensure version has 'v' prefix
if [[ "$APP_VERSION" != v* ]]; then
  APP_VERSION="v${APP_VERSION}"
fi

# Get git SHA and build date
GIT_SHA=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")
BUILD_DATE=$(date -u +%Y-%m-%dT%H:%M:%SZ)

echo "========================================="
echo "Building ${IMAGE_NAME}"
echo "  Version:    ${APP_VERSION}"
echo "  Git SHA:    ${GIT_SHA}"
echo "  Build Date: ${BUILD_DATE}"
echo "========================================="

# Build with all tags in one command
docker build \
  --build-arg APP_VERSION="${APP_VERSION}" \
  --build-arg GIT_SHA="${GIT_SHA}" \
  --build-arg BUILD_DATE="${BUILD_DATE}" \
  -t "${IMAGE_NAME}:${APP_VERSION}" \
  -t "${IMAGE_NAME}:${GIT_SHA}" \
  -t "${IMAGE_NAME}:${APP_VERSION}-${GIT_SHA}" \
  -t "${IMAGE_NAME}:latest" \
  .

echo ""
echo "Tags created:"
docker images "${IMAGE_NAME}" --format "  {{.Tag}}\t{{.Size}}\t{{.CreatedAt}}"
```

### Why This Works

The script reads the version from the `VERSION` file or accepts it as a command-line argument. The `v` prefix is added if missing for consistency. Multiple `-t` flags in a single `docker build` command create multiple tags for the same image -- this is efficient because Docker only builds once and applies all tags. The `GIT_SHA` uses `git rev-parse --short HEAD` for a 7-character short SHA that is human-readable but still unique for most repositories.

### Common Mistakes

- **Building the image multiple times for different tags.** Use multiple `-t` flags in one build command instead.
- **Not handling the case where `git` is unavailable.** The `2>/dev/null || echo "unknown"` fallback handles non-git environments.
- **Forgetting the `v` prefix.** Inconsistent versioning (`1.2.3` vs `v1.2.3`) causes confusion.

## Part D: Versioned Docker Compose Setup

### docker-compose.yml (base)

```yaml
services:
  app:
    image: ${REGISTRY:-localhost:5000}/myapp:${APP_VERSION:?APP_VERSION is required}
    ports:
      - "${APP_PORT:-3000}:3000"
    environment:
      - NODE_ENV=production
    healthcheck:
      test: ["CMD", "wget", "-qO-", "http://localhost:3000/health"]
      interval: 10s
      timeout: 5s
      retries: 3
```

### docker-compose.staging.yml

```yaml
services:
  app:
    ports:
      - "3001:3000"
    environment:
      - NODE_ENV=staging
```

### docker-compose.production.yml

```yaml
services:
  app:
    ports:
      - "3000:3000"
    environment:
      - NODE_ENV=production
    deploy:
      replicas: 2
      resources:
        limits:
          memory: 256M
          cpus: '0.5'
```

### Why This Works

`${APP_VERSION:?APP_VERSION is required}` forces docker-compose to fail with a clear error if `APP_VERSION` is not set. This prevents accidental deployment of unversioned images. The `REGISTRY` variable has a default (`localhost:5000`) for local development. The staging and production files override only what differs from the base, keeping configuration DRY.

### Common Mistakes

- **Not requiring `APP_VERSION`.** Without the `:?` syntax, docker-compose silently uses an empty string, which causes confusing errors.
- **Duplicating the full service definition in each override file.** Use separate compose files with `-f` to keep overrides minimal.
- **Hardcoding ports in the base file.** Use variables so different environments can use different ports.

## Part E: Test the Versioning System

```bash
# 1. Initialize the version at 1.0.0
./version.sh set 1.0.0
# Output: Version set to v1.0.0

# 2. Build version 1.0.0
./build.sh
# Builds myapp:v1.0.0, myapp:<sha>, myapp:v1.0.0-<sha>, myapp:latest

# 3. Bump to 1.0.1 and build
./version.sh patch
# Output: Version bumped: v1.0.0 -> v1.0.1
./build.sh

# 4. Bump to 1.1.0 and build
./version.sh minor
# Output: Version bumped: v1.0.1 -> v1.1.0
./build.sh

# 5. Deploy 1.0.0 to staging and verify
APP_VERSION=v1.0.0 APP_PORT=3001 \
  docker compose -f docker-compose.yml -f docker-compose.staging.yml up -d
curl http://localhost:3001/
# Response: { "version": "v1.0.0", "gitSha": "abc1234", ... }

# 6. Deploy 1.1.0 to production and verify
APP_VERSION=v1.1.0 APP_PORT=3000 \
  docker compose -f docker-compose.yml -f docker-compose.production.yml up -d
curl http://localhost:3000/
# Response: { "version": "v1.1.0", "gitSha": "abc1234", ... }

# 7. List all images
docker images myapp
# Shows v1.0.0, v1.0.1, v1.1.0, plus SHA and latest tags for each
```

### Why This Works

This sequence demonstrates the complete workflow: version bump, build, deploy, verify. Each version is independently deployable. The staging and production environments run different versions simultaneously. The health endpoint confirms which version is running in each environment.

### Common Mistakes

- **Forgetting to set `APP_VERSION` when deploying.** The compose file will fail with a clear error, which is the intended behavior.
- **Not verifying the deployed version.** Always `curl` the health endpoint to confirm the correct version is running.
- **Bumping versions without building.** The VERSION file and the built image must stay in sync.

## Key Takeaway

Semantic versioning for Docker images requires three components: a version management script (to bump versions), a versioned Dockerfile (to bake metadata into the image), and a docker-compose setup (to deploy specific versions). The `VERSION` file is the single source of truth. Build arguments inject version metadata at build time. OCI labels make the image self-describing. Required environment variables in docker-compose prevent accidental deployment of unversioned images. The combination of SemVer tags (for readability) and Git SHA tags (for traceability) gives you the best of both worlds.
