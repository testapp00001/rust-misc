# Module 13: Environment Variables — Config Without Rebuilding Images

> **Previous Module (12):** Health Checks
> **Previous Limitation:** Configuration is hardcoded into your Docker image. Change a database URL? Rebuild. Change a log level? Rebuild. This is unsustainable.
> **This Module Solves That:** Externalize all configuration so the same image runs in dev, staging, and production.

---

## 1. The Problem

You built a web application. It connects to a database, sends emails via SMTP, and talks to a payment API. In development, these point to localhost. In staging, they point to test services. In production, they point to real services.

If you bake these values into your Docker image, you need a different image for every environment. That means:

- Separate Dockerfiles or build args for each environment
- Rebuilding and retesting images just to change a URL
- No way to fix a misconfiguration without a full deploy cycle
- Secrets (passwords, API keys) trapped inside image layers

**The core problem:** A Docker image should be a single, immutable artifact that works everywhere. Configuration is what changes between environments, not the application code.

---

## 2. The Naive Way

### Hardcoding values in the Dockerfile

```dockerfile
FROM node:18-alpine
WORKDIR /app
COPY . .
RUN npm install

# Hardcoded — one image per environment
ENV DATABASE_URL=postgres://admin:password123@db:5432/prod
ENV SMTP_HOST=smtp.gmail.com
ENV LOG_LEVEL=info

CMD ["node", "server.js"]
```

### Why it fails

1. **Rebuild required for every config change.** Changing the database host means a new image build, new test cycle, new push, new deploy.
2. **Secrets are baked into layers.** Even if you delete them in a later `RUN` instruction, they persist in the image history. Anyone with `docker history` can read them.
3. **Environment proliferation.** You need `Dockerfile.dev`, `Dockerfile.staging`, `Dockerfile.prod` — a maintenance nightmare.
4. **No runtime flexibility.** You cannot adjust configuration after the container starts.

### A slightly better but still wrong approach: Build args

```dockerfile
ARG DATABASE_URL
ENV DATABASE_URL=$DATABASE_URL
```

```bash
docker build --build-arg DATABASE_URL=postgres://... -t myapp .
```

This avoids multiple Dockerfiles, but the value is still baked into the image at build time. Build args are not available at runtime — they are discarded after the build completes.

---

## 3. The Right Way

### Runtime configuration via environment variables

The application reads configuration from environment variables at startup. The Docker image contains only the code and its dependencies — zero configuration.

**Application code (Node.js example):**

```javascript
// config.js
const config = {
  databaseUrl: process.env.DATABASE_URL || 'postgres://localhost:5432/dev',
  smtpHost: process.env.SMTP_HOST || 'localhost',
  smtpPort: parseInt(process.env.SMTP_PORT || '587'),
  logLevel: process.env.LOG_LEVEL || 'debug',
  port: parseInt(process.env.PORT || '3000'),
};

// Validate required variables at startup
if (!process.env.DATABASE_URL) {
  console.warn('WARNING: DATABASE_URL not set, using default localhost');
}

module.exports = config;
```

**Application code (Rust example):**

```rust
use std::env;

fn main() {
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost:5432/dev".to_string());
    let log_level = env::var("LOG_LEVEL")
        .unwrap_or_else(|_| "debug".to_string());
    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .expect("PORT must be a valid number");

    println!("Connecting to: {}", database_url);
    println!("Log level: {}", log_level);
    println!("Listening on port: {}", port);
}
```

**Clean Dockerfile — no configuration at all:**

```dockerfile
FROM node:18-alpine
WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production
COPY . .
EXPOSE 3000
CMD ["node", "server.js"]
```

**Runtime configuration via `docker run -e`:**

```bash
# Development
docker run -e DATABASE_URL=postgres://localhost:5432/dev \
           -e LOG_LEVEL=debug \
           -p 3000:3000 myapp:1.0.0

# Staging
docker run -e DATABASE_URL=postgres://staging-db:5432/app \
           -e LOG_LEVEL=info \
           -p 3000:3000 myapp:1.0.0

# Production
docker run -e DATABASE_URL=postgres://prod-db:5432/app \
           -e LOG_LEVEL=warn \
           -p 3000:3000 myapp:1.0.0
```

Same image. Three different configurations. Zero rebuilds.

---

## 4. The Production Way

### Docker Compose with environment files

In production, you never type `docker run` commands by hand. You use Docker Compose (or Kubernetes, or an orchestrator) to manage configuration declaratively.

**Project structure:**

```
myapp/
├── docker-compose.yml
├── .env                    # Shared defaults (committed to git)
├── .env.local              # Local overrides (gitignored)
├── .env.example            # Template for new developers (committed)
├── Dockerfile
└── src/
```

**docker-compose.yml:**

```yaml
version: '3.8'

services:
  app:
    build: .
    ports:
      - "${APP_PORT:-3000}:3000"
    environment:
      - NODE_ENV=production
      - LOG_LEVEL=${LOG_LEVEL:-info}
    env_file:
      - .env
      - .env.local  # Overrides .env if present
    depends_on:
      db:
        condition: service_healthy

  db:
    image: postgres:15-alpine
    volumes:
      - postgres_data:/var/lib/postgresql/data
    env_file:
      - .env
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U ${DB_USER}"]
      interval: 10s
      timeout: 5s
      retries: 5

volumes:
  postgres_data:
```

**.env (committed to git — no secrets):**

```bash
# Application defaults
APP_PORT=3000
NODE_ENV=production
LOG_LEVEL=info

# Database
DB_USER=app
DB_NAME=myapp
DB_PORT=5432

# SMTP
SMTP_PORT=587
```

**.env.local (gitignored — contains secrets):**

```bash
# Database password (never commit this)
DB_PASSWORD=s3cur3-p@ssw0rd!

# API keys (never commit this)
STRIPE_API_KEY=sk_live_abc123...

# SMTP credentials
SMTP_PASSWORD=email-app-password
```

**.env.example (committed — template for developers):**

```bash
# Copy this file to .env.local and fill in the values
APP_PORT=3000
NODE_ENV=development
LOG_LEVEL=debug
DB_USER=app
DB_PASSWORD=changeme
DB_NAME=myapp
DB_PORT=5432
STRIPE_API_KEY=sk_test_your_key_here
SMTP_PASSWORD=your_smtp_password
```

### Configuration hierarchy (highest priority wins)

```
Highest priority
     |
     |  1. docker run -e    (runtime overrides)
     |  2. environment:     (in docker-compose.yml)
     |  3. .env.local       (gitignored env file)
     |  4. .env             (default env file)
     |  5. ENV in Dockerfile (build-time defaults)
     |  6. Code defaults     (process.env.X || "default")
     |
Lowest priority
```

### Variable substitution in docker-compose.yml

```yaml
services:
  app:
    image: myapp:${APP_VERSION:-latest}
    ports:
      - "${HOST_PORT:-3000}:${CONTAINER_PORT:-3000}"
    environment:
      - DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@${DB_HOST:-db}:${DB_PORT:-5432}/${DB_NAME}
```

The `${VAR:-default}` syntax means: use `VAR` if set, otherwise use `default`.

The `${VAR:?error message}` syntax means: fail immediately if `VAR` is not set. Use this for required values:

```yaml
environment:
  - DATABASE_URL=${DATABASE_URL:?DATABASE_URL is required}
```

### The 12-Factor App Methodology

Environment variables are a core principle of the [12-Factor App](https://12factor.net/) methodology:

| Factor | Principle | How env vars help |
|--------|-----------|-------------------|
| III. Config | Store config in the environment | Externalizes all configuration |
| V. Build/Release/Run | Strictly separate build and run | Same image, different envs |
| VI. Processes | Execute as stateless processes | Config comes from env, not filesystem |
| X. Dev/Prod Parity | Keep dev and prod as similar as possible | Same image, just different values |

### Security: Never bake secrets into images

```bash
# BAD — secrets in image layers
docker build --build-arg DB_PASSWORD=secret123 -t myapp .

# BETTER — but still visible in docker inspect
docker run -e DB_PASSWORD=secret123 myapp

# BEST — use Docker secrets or external secret managers (Module 14)
```

Even with env vars, secrets are visible via:

```bash
docker inspect <container>   # Shows all environment variables
cat /proc/<pid>/environ      # Can read from host
```

This is the limitation we will solve in Module 14: Secrets Management.

### Config validation at startup

The best production applications validate all configuration at startup and fail fast:

```rust
use std::env;
use std::process;

fn validate_config() -> Config {
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        eprintln!("ERROR: DATABASE_URL environment variable is required");
        process::exit(1);
    });

    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .unwrap_or_else(|_| {
            eprintln!("ERROR: PORT must be a valid port number (1-65535)");
            process::exit(1);
        });

    let log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());
    if !["trace", "debug", "info", "warn", "error"].contains(&log_level.as_str()) {
        eprintln!("ERROR: LOG_LEVEL must be one of: trace, debug, info, warn, error");
        process::exit(1);
    }

    Config { database_url, port, log_level }
}
```

This pattern catches configuration errors immediately at container startup, not when the first request hits a broken code path.

---

## 5. Hands-On Lab

### Lab: Configure a full-stack application with environment variables

**Objective:** Deploy a multi-service application (API + database) where all configuration is externalized via environment variables.

#### Step 1: Create the project structure

```bash
mkdir -p env-vars-lab && cd env-vars-lab
```

#### Step 2: Create the application

**server.js:**

```javascript
const express = require('express');
const { Pool } = require('pg');

const app = express();

// Read ALL configuration from environment variables
const config = {
  port: parseInt(process.env.PORT || '3000'),
  logLevel: process.env.LOG_LEVEL || 'info',
  db: {
    host: process.env.DB_HOST || 'localhost',
    port: parseInt(process.env.DB_PORT || '5432'),
    database: process.env.DB_NAME || 'myapp',
    user: process.env.DB_USER || 'postgres',
    password: process.env.DB_PASSWORD || '',
  },
  features: {
    enableMetrics: process.env.ENABLE_METRICS === 'true',
    maxUploadSize: process.env.MAX_UPLOAD_SIZE || '10mb',
  },
};

// Validate at startup
const required = ['DB_PASSWORD'];
for (const key of required) {
  if (!process.env[key]) {
    console.error(`FATAL: Required environment variable ${key} is not set`);
    process.exit(1);
  }
}

console.log(`[CONFIG] Port: ${config.port}`);
console.log(`[CONFIG] Log level: ${config.logLevel}`);
console.log(`[CONFIG] Database: ${config.db.host}:${config.db.port}/${config.db.database}`);
console.log(`[CONFIG] Metrics enabled: ${config.features.enableMetrics}`);

const pool = new Pool(config.db);

app.get('/health', async (req, res) => {
  try {
    await pool.query('SELECT 1');
    res.json({ status: 'healthy', db: 'connected' });
  } catch (err) {
    res.status(503).json({ status: 'unhealthy', db: err.message });
  }
});

app.get('/config', (req, res) => {
  // Never expose secrets in config endpoint!
  res.json({
    port: config.port,
    logLevel: config.logLevel,
    dbHost: config.db.host,
    dbName: config.db.database,
    features: config.features,
  });
});

app.get('/', (req, res) => {
  res.json({
    message: 'Environment Variables Lab',
    env: process.env.NODE_ENV || 'development',
    timestamp: new Date().toISOString(),
  });
});

app.listen(config.port, () => {
  console.log(`Server running on port ${config.port}`);
});
```

**package.json:**

```json
{
  "name": "env-vars-lab",
  "version": "1.0.0",
  "dependencies": {
    "express": "^4.18.2",
    "pg": "^8.11.3"
  }
}
```

#### Step 3: Create the Dockerfile

```dockerfile
FROM node:18-alpine
WORKDIR /app
COPY package*.json ./
RUN npm install --production
COPY server.js .
EXPOSE 3000
CMD ["node", "server.js"]
```

#### Step 4: Create environment files

**.env (defaults, committed to git):**

```bash
# Application
PORT=3000
NODE_ENV=production
LOG_LEVEL=info
MAX_UPLOAD_SIZE=10mb

# Database
DB_HOST=db
DB_PORT=5432
DB_NAME=myapp
DB_USER=postgres

# Features
ENABLE_METRICS=false
```

**.env.local (gitignored, contains secrets):**

```bash
DB_PASSWORD=super-secret-password
```

**.env.example (template):**

```bash
# Copy to .env.local and fill in values
DB_PASSWORD=changeme
```

#### Step 5: Create docker-compose.yml

```yaml
version: '3.8'

services:
  app:
    build: .
    ports:
      - "${PORT:-3000}:3000"
    environment:
      - NODE_ENV=${NODE_ENV:-development}
    env_file:
      - .env
      - .env.local
    depends_on:
      db:
        condition: service_healthy
    restart: unless-stopped

  db:
    image: postgres:15-alpine
    volumes:
      - pgdata:/var/lib/postgresql/data
    environment:
      - POSTGRES_DB=${DB_NAME:-myapp}
      - POSTGRES_USER=${DB_USER:-postgres}
      - POSTGRES_PASSWORD=${DB_PASSWORD:?DB_PASSWORD is required}
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U ${DB_USER:-postgres}"]
      interval: 5s
      timeout: 3s
      retries: 5

volumes:
  pgdata:
```

#### Step 6: Create .gitignore

```
.env.local
node_modules
```

#### Step 7: Run and test

```bash
# Start with default configuration
docker compose up -d

# Test with defaults
curl http://localhost:3000/
curl http://localhost:3000/config
curl http://localhost:3000/health

# Override a single variable at runtime
LOG_LEVEL=debug ENABLE_METRICS=true docker compose up -d

# Test the override took effect
curl http://localhost:3000/config
```

#### Step 8: Observe the hierarchy

```bash
# Set a value in .env
echo "LOG_LEVEL=info" >> .env

# Override it in .env.local
echo "LOG_LEVEL=debug" >> .env.local

# Override that in docker-compose.yml environment section
# environment:
#   - LOG_LEVEL=warn

# Override that at the command line
LOG_LEVEL=trace docker compose up -d

# Check which one won
curl http://localhost:3000/config
# Should show "trace" — highest priority wins
```

#### Step 9: Test validation

```bash
# Remove the password and watch the app fail fast
docker compose down
sed -i '/DB_PASSWORD/d' .env.local
docker compose up
# App should exit with: FATAL: Required environment variable DB_PASSWORD is not set
```

#### Cleanup

```bash
docker compose down -v
```

---

## 6. Limitation

Environment variables are the right way to externalize configuration, but they have a critical security flaw: **they are not secret.**

Anyone who can run `docker inspect` on your container can read every environment variable, including database passwords and API keys:

```bash
docker inspect myapp | grep -A 20 "Env"
# Reveals: DB_PASSWORD=super-secret-password
```

On Linux, you can also read them from `/proc/<pid>/environ` if you have access to the host.

In Docker Swarm, Kubernetes, or CI/CD systems, environment variables may be logged, stored in audit trails, or visible to anyone with cluster access.

**You need a way to inject secrets that is:**
- Encrypted at rest
- Not visible via `docker inspect`
- Rotatable without redeployment
- Auditable (who accessed what, when)

---

## 7. Next Topic

**Module 14: Secrets Management** — We will learn how to properly handle passwords, API keys, and certificates using Docker secrets, file-based secret injection, and external secret managers like HashiCorp Vault and AWS Secrets Manager. Environment variables are for configuration; secrets need a different approach.
