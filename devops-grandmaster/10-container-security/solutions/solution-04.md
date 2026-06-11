# Solution 04: Implement a Security-Hardened Container Pipeline

## Part A: Fixed Application Files

**Hardened Dockerfile:**

```dockerfile
FROM node:18-slim

# Create non-root user
RUN groupadd --gid 1001 appgroup && \
    useradd --uid 1001 --gid appgroup --shell /bin/sh --create-home appuser

WORKDIR /app

# Copy dependency files first for layer caching
COPY --chown=appuser:appgroup package*.json ./

# Install production dependencies
RUN npm ci --production && \
    npm cache clean --force

# Copy application code
COPY --chown=appuser:appgroup . .

# Remove any .env or sensitive files that might have been copied
RUN rm -f .env .env.* .git

# Switch to non-root user
USER appuser

EXPOSE 3000

HEALTHCHECK --interval=30s --timeout=5s --retries=3 \
  CMD ["node", "-e", "require('http').get('http://localhost:3000/health', (r) => { process.exit(r.statusCode === 200 ? 0 : 1) })"]

CMD ["node", "server.js"]
```

**Hardened docker-compose.yml:**

```yaml
version: "3.8"

services:
  app:
    build: .
    image: my-app:latest
    user: "1001:1001"
    read_only: true
    security_opt:
      - no-new-privileges:true
    cap_drop:
      - ALL
    tmpfs:
      - /tmp:size=100M,noexec,nosuid
    networks:
      - frontend
    ports:
      - "3000:3000"
    secrets:
      - secret_key
    environment:
      SECRET_KEY_FILE: /run/secrets/secret_key
    deploy:
      resources:
        limits:
          memory: 512M
          cpus: '0.5'
    logging:
      driver: json-file
      options:
        max-size: "10m"
        max-file: "3"

networks:
  frontend:
    driver: bridge

secrets:
  secret_key:
    file: ./secrets/secret_key.txt
```

---

## Part B: Security Gate Script

```bash
#!/bin/bash
# security-gate.sh -- Container security gate for CI/CD pipelines
set -euo pipefail

IMAGE_NAME="${1:-my-app:latest}"
FAILURES=0
TOTAL=0

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

pass() {
    echo -e "  ${GREEN}[PASS]${NC} $1"
}

fail() {
    echo -e "  ${RED}[FAIL]${NC} $1"
    FAILURES=$((FAILURES + 1))
}

warn() {
    echo -e "  ${YELLOW}[WARN]${NC} $1"
}

check() {
    TOTAL=$((TOTAL + 1))
}

echo "=== Container Security Gate ==="
echo "Image: $IMAGE_NAME"
echo ""

# -------------------------------------------------------
# Check 1: Build the image
# -------------------------------------------------------
check
echo "[1/6] Building image..."
if docker build -t "$IMAGE_NAME" . > /dev/null 2>&1; then
    pass "Image built successfully"
else
    fail "Image build failed"
    echo ""
    echo "=== Security Gate FAILED (build error) ==="
    exit 1
fi

# -------------------------------------------------------
# Check 2: Scan for CRITICAL vulnerabilities
# -------------------------------------------------------
check
echo "[2/6] Scanning for CRITICAL vulnerabilities..."
if command -v trivy &> /dev/null; then
    SCAN_OUTPUT=$(trivy image --severity CRITICAL --format json "$IMAGE_NAME" 2>/dev/null)
    CRITICAL_COUNT=$(echo "$SCAN_OUTPUT" | jq '[.Results[]?.Vulnerabilities // [] | length] | add // 0')
    if [ "$CRITICAL_COUNT" -eq 0 ]; then
        pass "No CRITICAL vulnerabilities found"
    else
        fail "Found $CRITICAL_COUNT CRITICAL vulnerabilities"
        trivy image --severity CRITICAL "$IMAGE_NAME" 2>/dev/null | grep -E "^\S+" | head -10
    fi
else
    warn "Trivy not installed -- skipping vulnerability scan"
fi

# -------------------------------------------------------
# Check 3: Verify non-root user
# -------------------------------------------------------
check
echo "[3/6] Verifying non-root user..."
CONTAINER_USER=$(docker run --rm "$IMAGE_NAME" whoami 2>/dev/null || echo "unknown")
if [ "$CONTAINER_USER" = "root" ]; then
    fail "Container runs as root"
else
    pass "Container runs as: $CONTAINER_USER"
fi

# -------------------------------------------------------
# Check 4: Check for secrets in image layers
# -------------------------------------------------------
check
echo "[4/6] Checking for secrets in image layers..."
LAYER_SECRETS=$(docker history "$IMAGE_NAME" --no-trunc 2>/dev/null | \
    grep -iE 'PASSWORD=|SECRET=|API_KEY=|TOKEN=|PRIVATE_KEY=' || true)
if [ -n "$LAYER_SECRETS" ]; then
    fail "Potential secrets found in image layers"
    echo "    $LAYER_SECRETS"
else
    pass "No secrets found in image layers"
fi

# -------------------------------------------------------
# Check 5: Verify no sensitive files in image
# -------------------------------------------------------
check
echo "[5/6] Checking for sensitive files..."
SENSITIVE_FILES=""
for file in .env .env.local .env.production .git node_modules/.cache; do
    if docker run --rm "$IMAGE_NAME" sh -c "test -e /app/$file" 2>/dev/null; then
        SENSITIVE_FILES="$SENSITIVE_FILES $file"
    fi
done
if [ -n "$SENSITIVE_FILES" ]; then
    fail "Sensitive files found in image:$SENSITIVE_FILES"
else
    pass "No sensitive files found"
fi

# -------------------------------------------------------
# Check 6: Verify read-only filesystem support
# -------------------------------------------------------
check
echo "[6/6] Verifying read-only filesystem support..."
if docker run --rm --read-only --tmpfs /tmp:size=50M "$IMAGE_NAME" sh -c "echo ok > /tmp/test" 2>/dev/null; then
    pass "Container works with read-only filesystem"
else
    fail "Container does not work with read-only filesystem"
fi

# -------------------------------------------------------
# Summary
# -------------------------------------------------------
echo ""
PASSED=$((TOTAL - FAILURES))
echo "=== Security Gate Summary ==="
echo "Checks: $PASSED/$TOTAL passed"

if [ "$FAILURES" -gt 0 ]; then
    echo -e "${RED}Result: FAILED${NC} ($FAILURES check(s) failed)"
    exit 1
else
    echo -e "${GREEN}Result: PASSED${NC}"
    exit 0
fi
```

---

## Part C: Full Pipeline Script

```bash
#!/bin/bash
# deploy-pipeline.sh -- Full container build, scan, and deploy pipeline
set -euo pipefail

# Default values
IMAGE_NAME="my-app"
VERSION="latest"
REGISTRY=""
SIGN_IMAGE=false

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --image) IMAGE_NAME="$2"; shift 2 ;;
        --version) VERSION="$2"; shift 2 ;;
        --registry) REGISTRY="$2"; shift 2 ;;
        --sign) SIGN_IMAGE=true; shift ;;
        *) echo "Unknown option: $1"; exit 1 ;;
    esac
done

# Generate image tags
SHA=$(git rev-parse --short HEAD 2>/dev/null || echo "no-git")
FULL_IMAGE="${REGISTRY:+$REGISTRY/}${IMAGE_NAME}"
TAG_VERSION="${FULL_IMAGE}:${VERSION}"
TAG_SHA="${FULL_IMAGE}:sha-${SHA}"

echo -e "${BLUE}=== Container Deployment Pipeline ===${NC}"
echo "Image:     $FULL_IMAGE"
echo "Version:   $VERSION"
echo "SHA:       $SHA"
echo "Registry:  ${REGISTRY:-local}"
echo ""

# -------------------------------------------------------
# Stage 1: Security Gate
# -------------------------------------------------------
echo -e "${BLUE}[Stage 1/4] Running security gate...${NC}"

# Run the security gate script
if [ -f ./security-gate.sh ]; then
    bash ./security-gate.sh "$TAG_VERSION"
else
    # Inline security checks if the script is not available
    echo "Building image..."
    docker build -t "$TAG_VERSION" . || { echo -e "${RED}Build failed${NC}"; exit 1; }

    echo "Scanning for vulnerabilities..."
    if command -v trivy &> /dev/null; then
        trivy image --exit-code 1 --severity CRITICAL "$TAG_VERSION" || {
            echo -e "${RED}Critical vulnerabilities found${NC}"
            exit 1
        }
    else
        echo -e "${YELLOW}Trivy not installed -- skipping scan${NC}"
    fi

    echo "Verifying non-root user..."
    USER_CHECK=$(docker run --rm "$TAG_VERSION" whoami 2>/dev/null)
    if [ "$USER_CHECK" = "root" ]; then
        echo -e "${RED}Container runs as root${NC}"
        exit 1
    fi
fi

echo -e "${GREEN}Security gate passed${NC}"
echo ""

# -------------------------------------------------------
# Stage 2: Tag image
# -------------------------------------------------------
echo -e "${BLUE}[Stage 2/4] Tagging image...${NC}"

docker tag "$TAG_VERSION" "$TAG_SHA"
echo "  Tagged: $TAG_VERSION"
echo "  Tagged: $TAG_SHA"

if [ -n "$REGISTRY" ]; then
    TAG_LATEST="${FULL_IMAGE}:latest"
    docker tag "$TAG_VERSION" "$TAG_LATEST"
    echo "  Tagged: $TAG_LATEST"
fi

echo -e "${GREEN}Tagging complete${NC}"
echo ""

# -------------------------------------------------------
# Stage 3: Sign image (optional)
# -------------------------------------------------------
echo -e "${BLUE}[Stage 3/4] Signing image...${NC}"

if [ "$SIGN_IMAGE" = true ]; then
    if command -v cosign &> /dev/null; then
        # Generate key pair if not present
        if [ ! -f cosign.key ]; then
            echo "Generating cosign key pair..."
            cosign generate-key-pair
        fi

        echo "Signing image..."
        cosign sign --key cosign.key "$TAG_VERSION"
        cosign sign --key cosign.key "$TAG_SHA"
        echo -e "${GREEN}Image signed${NC}"
    else
        echo -e "${YELLOW}Cosign not installed -- skipping signing${NC}"
    fi
else
    echo -e "${YELLOW}Signing skipped (use --sign to enable)${NC}"
fi
echo ""

# -------------------------------------------------------
# Stage 4: Push image (if registry specified)
# -------------------------------------------------------
echo -e "${BLUE}[Stage 4/4] Pushing image...${NC}"

if [ -n "$REGISTRY" ]; then
    docker push "$TAG_VERSION"
    docker push "$TAG_SHA"
    docker push "$TAG_LATEST"
    echo -e "${GREEN}Pushed to $REGISTRY${NC}"
else
    echo -e "${YELLOW}No registry specified -- skipping push${NC}"
fi

# -------------------------------------------------------
# Summary
# -------------------------------------------------------
echo ""
echo -e "${GREEN}=== Pipeline Complete ===${NC}"
echo "Image:     $TAG_VERSION"
echo "SHA:       $TAG_SHA"
echo "Status:    Ready for deployment"
```

---

## Part D: Test the Pipeline

**Testing with the fixed Dockerfile:**

```bash
chmod +x security-gate.sh deploy-pipeline.sh

# Run the full pipeline
./deploy-pipeline.sh --image my-app --version v1.0.0

# Expected output:
# [Stage 1/4] Running security gate...
#   [PASS] Image built successfully
#   [PASS] No CRITICAL vulnerabilities found
#   [PASS] Container runs as: appuser
#   [PASS] No secrets found in image layers
#   [PASS] No sensitive files found
#   [PASS] Container works with read-only filesystem
# Security gate passed
# [Stage 2/4] Tagging image...
# [Stage 3/4] Signing image...
# [Stage 4/4] Pushing image...
# === Pipeline Complete ===
```

**Testing failure detection:**

```bash
# Revert to root user in Dockerfile
sed -i 's/USER appuser//' Dockerfile

# Run pipeline -- should fail at security gate
./deploy-pipeline.sh --image my-app --version v1.0.0

# Expected output:
#   [FAIL] Container runs as root
# Security gate FAILED
```

---

## Common Mistakes

- **Not using `set -euo pipefail`:** Without this, the script continues after a failed command. In a security pipeline, you want to fail fast -- any check failure should stop the entire pipeline.
- **Skipping the vulnerability scan in CI:** Some teams skip scanning because it takes time. This defeats the purpose of the pipeline. The scan is the most important check.
- **Not checking `docker history` for secrets:** Even if the current Dockerfile does not contain secrets, previous image layers might. `docker history --no-trunc` shows the full command used in each layer.
- **Hardcoding the image name in the script:** The script should accept the image name as a parameter so it can be reused across projects.
- **Not testing the pipeline itself:** The pipeline script should be tested as thoroughly as the application. Introduce deliberate failures and verify that the pipeline catches them.
