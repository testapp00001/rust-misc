# Solution 03: Environment-Specific Configuration

## Part A: File structure

### .env.defaults (committed -- shared across all environments)

```bash
# Shared defaults -- values that are the same in all environments
DB_PORT=5432
REDIS_PORT=6379
```

### .env.development (committed -- dev overrides)

```bash
# Development environment
NODE_ENV=development
LOG_LEVEL=debug
DB_HOST=localhost
DB_NAME=myapp_dev
DB_USER=postgres
DB_PASSWORD=
ENABLE_DEBUG_UI=true
MAX_CONNECTIONS=5
CACHE_TTL=60
```

### .env.staging (committed -- staging overrides)

```bash
# Staging environment
NODE_ENV=staging
LOG_LEVEL=info
DB_HOST=staging-db.internal
DB_NAME=myapp_staging
DB_USER=app_staging
DB_PASSWORD=changeme
ENABLE_DEBUG_UI=false
MAX_CONNECTIONS=20
CACHE_TTL=300
```

### .env.production (committed -- production overrides)

```bash
# Production environment
NODE_ENV=production
LOG_LEVEL=warn
DB_HOST=prod-db.internal
DB_NAME=myapp_prod
DB_USER=app_prod
DB_PASSWORD=changeme
ENABLE_DEBUG_UI=false
MAX_CONNECTIONS=100
CACHE_TTL=3600
```

### .env.secrets (gitignored -- actual passwords)

```bash
# Actual passwords (never commit this file)
# Staging
DB_PASSWORD_STAGING=stg-pass-123
# Production
DB_PASSWORD_PROD=prod-pass-xyz
```

### .gitignore

```
.env.secrets
```

### Why this works

- Shared values (`DB_PORT`, `REDIS_PORT`) are defined once in `.env.defaults`, not duplicated across three environment files.
- Each environment file contains only the values that differ for that environment.
- Passwords are in a single gitignored file. The environment-specific files use placeholder values (`changeme`) that are safe to commit.
- In practice, CI/CD pipelines would inject real passwords at deploy time, not read them from `.env.secrets`.

### Common mistakes

- Duplicating `DB_PORT=5432` in every environment file. If the port changes, you have to update three files. Define it once in defaults.
- Putting real passwords in the environment-specific files. Even if the file is "for production," it will be committed by accident eventually.
- Using the same password for all environments. Development passwords should be empty or trivial; production passwords should be strong and unique.

---

## Part B: Docker Compose files

### docker-compose.yml (base -- shared structure)

```yaml
version: '3.8'

services:
  app:
    build: .
    ports:
      - "${APP_PORT:-3000}:3000"
    env_file:
      - .env.defaults
    restart: unless-stopped

  db:
    image: postgres:15-alpine
    volumes:
      - pgdata:/var/lib/postgresql/data
    environment:
      - POSTGRES_DB=${DB_NAME}
      - POSTGRES_USER=${DB_USER}
      - POSTGRES_PASSWORD=${DB_PASSWORD}
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U ${DB_USER}"]
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

### docker-compose.dev.yml (development overrides)

```yaml
version: '3.8'

services:
  app:
    env_file:
      - .env.defaults
      - .env.development
    environment:
      - DB_PASSWORD=
    volumes:
      - ./src:/app/src
    command: ["python", "app.py", "--reload"]

  db:
    environment:
      - POSTGRES_PASSWORD=devpassword
```

### docker-compose.staging.yml (staging overrides)

```yaml
version: '3.8'

services:
  app:
    env_file:
      - .env.defaults
      - .env.staging
      - .env.secrets
    deploy:
      resources:
        limits:
          cpus: '1.0'
          memory: 512M

  db:
    environment:
      - POSTGRES_PASSWORD=${DB_PASSWORD_STAGING}
```

### docker-compose.prod.yml (production overrides)

```yaml
version: '3.8'

services:
  app:
    env_file:
      - .env.defaults
      - .env.production
      - .env.secrets
    deploy:
      resources:
        limits:
          cpus: '2.0'
          memory: 1G
    restart: always

  db:
    environment:
      - POSTGRES_PASSWORD=${DB_PASSWORD_PROD}
```

### Why this works

- The base file defines the service topology (app, db, redis) once.
- Override files add environment-specific `env_file` entries, resource limits, and behavior changes.
- Development mounts source code for live reloading; production sets resource limits and strict restart policies.
- The `docker compose -f` flag layers these files, with later files overriding earlier ones.

### Common mistakes

- Defining the full service in every override file. The override file should only contain the *differences* from the base.
- Forgetting that `env_file` in an override replaces the base file's `env_file`, it does not merge them. If the base has `env_file: [.env.defaults]` and the override has `env_file: [.env.staging]`, only `.env.staging` is loaded. You must list all needed files in the override.
- Not using `deploy.resources` for production. Without resource limits, a single container can consume all host resources.

---

## Part C: Startup script

```bash
#!/usr/bin/env bash
set -euo pipefail

ENV="${1:-}"

if [ -z "$ENV" ]; then
    echo "Usage: ./start.sh <dev|staging|prod>"
    exit 1
fi

# Map environment name to compose file
case "$ENV" in
    dev|development)
        COMPOSE_FILE="docker-compose.dev.yml"
        ENV_NAME="Development"
        ;;
    staging|stg)
        COMPOSE_FILE="docker-compose.staging.yml"
        ENV_NAME="Staging"
        ;;
    prod|production)
        COMPOSE_FILE="docker-compose.prod.yml"
        ENV_NAME="Production"
        ;;
    *)
        echo "Error: Unknown environment '$ENV'"
        echo "Usage: ./start.sh <dev|staging|prod>"
        exit 1
        ;;
esac

# Validate required files exist
REQUIRED_FILES=("docker-compose.yml" "$COMPOSE_FILE" ".env.defaults")

for file in "${REQUIRED_FILES[@]}"; do
    if [ ! -f "$file" ]; then
        echo "Error: Required file '$file' not found"
        exit 1
    fi
done

echo "Starting $ENV_NAME environment..."
echo "Using: docker-compose.yml + $COMPOSE_FILE"

docker compose \
    -f docker-compose.yml \
    -f "$COMPOSE_FILE" \
    up -d

echo "$ENV_NAME environment started."
echo "Run 'docker compose -f docker-compose.yml -f $COMPOSE_FILE logs -f' to view logs."
```

### Why this works

- `set -euo pipefail` makes the script fail on any error, undefined variable, or pipe failure.
- The `case` statement maps short names (`dev`, `prod`) to the correct compose file.
- File existence checks prevent cryptic Docker Compose errors from missing files.
- The script prints the resolved compose files so the operator can verify the configuration.

### Common mistakes

- Not quoting `$ENV` in the case statement. Without quotes, `./start.sh` (no argument) would match nothing but might not produce a clear error.
- Forgetting `set -e`. Without it, a failed `docker compose` command would be silently ignored and the script would print "started" anyway.
- Hardcoding `docker compose` without the `-f` flags in the final command. The operator needs to see exactly which files are being used.

---

## Part D: Verify environment isolation

```bash
# 1. Print resolved development configuration
docker compose \
    -f docker-compose.yml \
    -f docker-compose.dev.yml \
    config

# 2. Print resolved production configuration
docker compose \
    -f docker-compose.yml \
    -f docker-compose.prod.yml \
    config

# 3. Compare variable names between environments
docker compose -f docker-compose.yml -f docker-compose.dev.yml config \
    | grep -oP 'LOG_LEVEL=\K.*' > /tmp/dev_vars.txt
docker compose -f docker-compose.yml -f docker-compose.prod.yml config \
    | grep -oP 'LOG_LEVEL=\K.*' > /tmp/prod_vars.txt

echo "=== Development LOG_LEVEL ==="
cat /tmp/dev_vars.txt

echo "=== Production LOG_LEVEL ==="
cat /tmp/prod_vars.txt
```

### Why this works

- `docker compose config` resolves all env files, variable substitutions, and overrides, then prints the final merged configuration. This is the single best command for debugging configuration issues.
- Comparing the output for different environments reveals exactly what changes between them.
- The same Docker image is used in both cases -- only the environment variables differ.

### Common mistakes

- Running `docker compose config` without the `-f` flags. It only reads `docker-compose.yml` by default, missing all environment-specific overrides.
- Confusing `docker compose config` (prints resolved config) with `docker compose ps` (shows running containers). The `config` command works even when no containers are running.
