# Solution 05: Docker Compose and Kubernetes ConfigMaps

## Part A: Docker Compose environment

### .env (committed -- non-secret defaults)

```bash
# Application
APP_PORT=3000
APP_HOST=0.0.0.0
LOG_LEVEL=info

# Database
DB_HOST=postgres
DB_PORT=5432
DB_NAME=userservice
DB_USER=app
DB_POOL_SIZE=10

# Redis
REDIS_HOST=redis
REDIS_PORT=6379
CACHE_TTL=300

# CORS
CORS_ORIGINS=*
```

### .env.local (gitignored -- secrets)

```bash
DB_PASSWORD=dev-password-123
```

### .gitignore

```
.env.local
```

### Dockerfile

```dockerfile
FROM node:18-alpine
WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production
COPY src/ ./src/
EXPOSE 3000
CMD ["node", "src/server.js"]
```

### docker-compose.yml

```yaml
version: '3.8'

services:
  app:
    build: .
    ports:
      - "${APP_PORT:-3000}:3000"
    environment:
      - NODE_ENV=development
    env_file:
      - .env
      - .env.local
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_healthy
    restart: unless-stopped

  postgres:
    image: postgres:15-alpine
    environment:
      - POSTGRES_DB=${DB_NAME:-userservice}
      - POSTGRES_USER=${DB_USER:-app}
      - POSTGRES_PASSWORD=${DB_PASSWORD:?DB_PASSWORD is required}
    volumes:
      - pgdata:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U ${DB_USER:-app}"]
      interval: 5s
      timeout: 3s
      retries: 5

  redis:
    image: redis:7-alpine
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 5s
      timeout: 3s
      retries: 5

volumes:
  pgdata:
```

### Why this works

- The app reads all configuration from environment variables. The Dockerfile has zero hardcoded config.
- The postgres service reuses the same `DB_NAME`, `DB_USER`, and `DB_PASSWORD` variables via `${VAR}` substitution in docker-compose.yml.
- `depends_on` with `condition: service_healthy` ensures the app does not start until postgres and redis are ready.
- `${DB_PASSWORD:?DB_PASSWORD is required}` causes docker compose to fail immediately if the password is not set, preventing a silent startup with no database access.

### Common mistakes

- Using `DB_PASSWORD` directly in the postgres service's `POSTGRES_PASSWORD` without the `${VAR:?}` syntax. If `.env.local` is missing, the postgres container starts with an empty password and the app cannot authenticate.
- Not using health checks with `depends_on`. Without them, the app starts immediately and tries to connect before postgres is accepting connections.

---

## Part B: Kubernetes ConfigMap

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: user-service-config
  namespace: default
data:
  APP_PORT: "3000"
  APP_HOST: "0.0.0.0"
  LOG_LEVEL: "info"
  DB_HOST: "postgres"
  DB_PORT: "5432"
  DB_NAME: "userservice"
  DB_USER: "app"
  DB_POOL_SIZE: "10"
  REDIS_HOST: "redis"
  REDIS_PORT: "6379"
  CACHE_TTL: "300"
  CORS_ORIGINS: "*"
```

### Why this works

- Every non-secret value from the `.env` file is represented in the ConfigMap.
- All values are strings. Kubernetes ConfigMaps do not have typed values -- the application is responsible for parsing `"3000"` as an integer.
- The ConfigMap name (`user-service-config`) is referenced by the Deployment to inject values as environment variables.

### Common mistakes

- Forgetting that ConfigMap values are always strings. If your application expects an integer for `DB_PORT`, it must parse the string itself.
- Putting secrets (like `DB_PASSWORD`) in the ConfigMap. ConfigMaps are not encrypted and are visible to anyone with read access to the cluster. Use Secrets for sensitive values.
- Using integer values without quotes in YAML. `DB_PORT: 3000` is valid YAML but may cause issues with some Kubernetes tools. Use `DB_PORT: "3000"` for consistency.

---

## Part C: Kubernetes Secret

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: user-service-secret
  namespace: default
type: Opaque
stringData:
  DB_PASSWORD: "prod-secure-password-here"
```

### Why this works

- `stringData` accepts plaintext values. Kubernetes base64-encodes them automatically when storing the Secret.
- `type: Opaque` is the generic Secret type for arbitrary key-value pairs.
- The Secret is separate from the ConfigMap, allowing different access controls (RBAC) and rotation policies.

### Common mistakes

- Using `data:` with manually base64-encoded values. While this works, `stringData:` is more readable and less error-prone. Kubernetes handles the encoding.
- Committing the Secret manifest with real passwords to git. In practice, use `kubectl create secret` imperatively, or use a secrets manager (Module 14).
- Naming the Secret something generic like `my-secret`. Use a descriptive name like `user-service-secret` so it is clear which service it belongs to.

---

## Part D: Kubernetes Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: user-service
  namespace: default
  labels:
    app: user-service
spec:
  replicas: 2
  selector:
    matchLabels:
      app: user-service
  template:
    metadata:
      labels:
        app: user-service
    spec:
      containers:
        - name: user-service
          image: user-service:latest
          ports:
            - containerPort: 3000
          envFrom:
            - configMapRef:
                name: user-service-config
          env:
            - name: DB_PASSWORD
              valueFrom:
                secretKeyRef:
                  name: user-service-secret
                  key: DB_PASSWORD
            - name: NODE_ENV
              value: "production"
          resources:
            requests:
              cpu: 100m
              memory: 128Mi
            limits:
              cpu: 500m
              memory: 256Mi
          readinessProbe:
            httpGet:
              path: /health
              port: 3000
            initialDelaySeconds: 5
            periodSeconds: 10
          livenessProbe:
            httpGet:
              path: /health
              port: 3000
            initialDelaySeconds: 15
            periodSeconds: 20
```

### Why this works

- `envFrom` with `configMapRef` injects all keys from the ConfigMap as environment variables. This is the equivalent of `env_file` in Docker Compose.
- Individual `env` entries with `secretKeyRef` inject specific Secret values. This gives fine-grained control over which secrets are exposed and under what name.
- `NODE_ENV` is set directly in the Deployment because it should always be `"production"` in this context, regardless of the ConfigMap.
- `readinessProbe` ensures the pod does not receive traffic until `/health` responds with 200.
- Resource requests and limits prevent the container from consuming unbounded resources.

### Common mistakes

- Using `envFrom` for Secrets. While technically possible (`secretRef`), this injects every key from the Secret. If the Secret contains values for multiple services, unrelated secrets leak into the container.
- Forgetting `readinessProbe`. Without it, Kubernetes sends traffic to the pod immediately, even if the application has not finished starting or connecting to the database.
- Setting `limits` without `requests`. Kubernetes uses `requests` for scheduling and `limits` for enforcement. Without `requests`, the pod might be scheduled on a node with insufficient resources.

---

## Part E: Generation script

```bash
#!/usr/bin/env bash
set -euo pipefail

ENV_FILE="${1:-.env}"

if [ ! -f "$ENV_FILE" ]; then
    echo "Error: File '$ENV_FILE' not found" >&2
    exit 1
fi

cat <<EOF
apiVersion: v1
kind: ConfigMap
metadata:
  name: user-service-config
  namespace: default
data:
EOF

# Process each non-comment, non-empty line
grep -v '^#' "$ENV_FILE" | grep -v '^$' | while IFS= read -r line; do
    # Split on first = only (values may contain =)
    key="${line%%=*}"
    value="${line#*=}"

    # Skip lines that look like secrets (password, key, token, secret)
    case "${key,,}" in
        *password*|*secret*|*key*|*token*)
            echo "  # SKIPPED: $key (secret -- use a Secret manifest)" >&2
            continue
            ;;
    esac

    # Output as YAML (quote all values to handle numbers and special chars)
    printf '  %s: "%s"\n' "$key" "$value"
done
```

### Why this works

- `grep -v '^#'` removes comment lines. `grep -v '^$'` removes empty lines.
- `${line%%=*}` extracts the key (everything before the first `=`). `${line#*=}` extracts the value (everything after the first `=`).
- The `case` statement skips keys that look like secrets (containing "password", "secret", "key", or "token"). This prevents accidental secret exposure in ConfigMaps.
- All values are quoted in YAML to prevent type interpretation issues.
- The script outputs to stdout, so you can redirect to a file (`./generate-k8s-config.sh .env > configmap.yaml`) or pipe to `kubectl apply`.

### Common mistakes

- Using `awk -F=` to split. This breaks if the value contains `=` (common in connection strings like `postgres://user=pass@host`). Use shell parameter expansion (`${line%%=*}`) which splits on the first `=` only.
- Not quoting YAML values. `DB_PORT: 5432` works, but `DB_PASSWORD: p@ss!w0rd` breaks YAML parsing due to special characters. Always quote: `DB_PASSWORD: "p@ss!w0rd"`.
- Forgetting to handle the secret-skipping logic. Without it, `DB_PASSWORD` from the `.env` file ends up in the ConfigMap, visible to anyone with cluster access.

---

## Part F: Verify parity

```bash
#!/usr/bin/env bash
set -euo pipefail

echo "=== Docker Compose resolved variables ==="
COMPOSE_VARS=$(docker compose config 2>/dev/null \
    | grep -oP '^\s+\K[A-Z_]+(?==)' \
    | sort -u)
echo "$COMPOSE_VARS"

echo ""
echo "=== Kubernetes ConfigMap variables ==="
# Requires: kubectl access to the cluster
K8S_VARS=$(kubectl get configmap user-service-config -o jsonpath='{.data}' \
    | python3 -c "import sys,json; print('\n'.join(sorted(json.loads(sys.stdin.read()).keys())))")
echo "$K8S_VARS"

echo ""
echo "=== Comparison ==="
# Variables in Compose but not in Kubernetes
COMPOSE_ONLY=$(comm -23 <(echo "$COMPOSE_VARS") <(echo "$K8S_VARS"))
# Variables in Kubernetes but not in Compose
K8S_ONLY=$(comm -13 <(echo "$COMPOSE_VARS") <(echo "$K8S_VARS"))

if [ -z "$COMPOSE_ONLY" ] && [ -z "$K8S_ONLY" ]; then
    echo "PASS: ConfigMap contains the same variables as Docker Compose."
else
    if [ -n "$COMPOSE_ONLY" ]; then
        echo "In Compose only (missing from ConfigMap):"
        echo "$COMPOSE_ONLY"
    fi
    if [ -n "$K8S_ONLY" ]; then
        echo "In ConfigMap only (not in Compose):"
        echo "$K8S_ONLY"
    fi
fi
```

### Why this works

- `docker compose config` prints the fully resolved configuration. Extracting variable names from it gives the definitive list of what Compose sees.
- `kubectl get configmap -o jsonpath` extracts the ConfigMap data as JSON. A Python one-liner parses it and prints sorted keys.
- `comm -23` shows lines only in the first file (Compose-only). `comm -13` shows lines only in the second file (K8s-only). An empty diff means parity.
- The script focuses on variable *names*, not values, because Compose values may include substituted references (`${DB_PASSWORD}`) while ConfigMap values are literal strings.

### Common mistakes

- Comparing raw env file content with ConfigMap content. The env file may contain comments and empty lines; the ConfigMap does not. Extract and compare keys only.
- Forgetting that Docker Compose adds internal variables (like `PATH`) that are not in the ConfigMap. Filter to only application-specific variables.
- Not accounting for the fact that secrets (like `DB_PASSWORD`) should intentionally differ between Compose (env file) and Kubernetes (Secret, not ConfigMap). The comparison should exclude secret variables.
