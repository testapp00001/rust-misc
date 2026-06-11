# Exercise 05: Docker Compose and Kubernetes ConfigMaps

**Type:** Integration
**Time:** 45 minutes
**Difficulty:** Hard

## Objective

Bridge the gap between Docker Compose environment files and Kubernetes ConfigMaps by building a configuration workflow that works in both local development (Docker Compose) and production (Kubernetes). This exercise integrates environment variable concepts from this module with container orchestration skills.

## Scenario

Your team develops locally with Docker Compose but deploys to Kubernetes. Currently, configuration is managed separately for each platform -- env files for Compose, manually created ConfigMaps for Kubernetes. This leads to drift: a variable added for Compose is forgotten in Kubernetes, or defaults differ between environments.

Your task: create a single source of truth for configuration that generates both Docker Compose env files and Kubernetes ConfigMaps.

## Application

You are deploying a user service with this configuration:

| Variable | Default | Description |
|----------|---------|-------------|
| `APP_PORT` | 3000 | HTTP listen port |
| `APP_HOST` | 0.0.0.0 | Bind address |
| `LOG_LEVEL` | info | Logging verbosity |
| `DB_HOST` | postgres | Database hostname |
| `DB_PORT` | 5432 | Database port |
| `DB_NAME` | userservice | Database name |
| `DB_USER` | app | Database username |
| `DB_POOL_SIZE` | 10 | Connection pool size |
| `REDIS_HOST` | redis | Cache hostname |
| `REDIS_PORT` | 6379 | Cache port |
| `CACHE_TTL` | 300 | Default cache TTL in seconds |
| `CORS_ORIGINS` | * | Allowed CORS origins |

## Tasks

### Part A: Create the Docker Compose environment

Create the following files for local development:

1. `.env` -- All non-secret defaults for local development.
2. `.env.local` -- Secrets (database password).
3. `docker-compose.yml` -- Service definition that loads env files and uses `${VAR:-default}` syntax.
4. `Dockerfile` -- A minimal image with no hardcoded config.

The app service, a postgres service, and a redis service should all start together. The database and redis services should use the same env vars for hostnames and ports.

<details>
<summary>Hint</summary>

The postgres service needs `POSTGRES_DB`, `POSTGRES_USER`, and `POSTGRES_PASSWORD` environment variables. Map your application's `DB_*` variables to these. Use `depends_on` with health checks so the app waits for the database to be ready.

</details>

### Part B: Create a ConfigMap from the same values

Write a Kubernetes ConfigMap manifest (`configmap.yaml`) that contains the same configuration values as your `.env` file. The ConfigMap should be named `user-service-config` and live in the `default` namespace.

<details>
<summary>Hint</summary>

A ConfigMap's `data` section maps string keys to string values, just like an env file. The syntax is:

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: user-service-config
data:
  APP_PORT: "3000"
```

Note that all values in a ConfigMap are strings, even numbers.

</details>

### Part C: Create a Kubernetes Secret for sensitive values

Write a Kubernetes Secret manifest (`secret.yaml`) that contains the database password. Use `stringData` for readability.

<details>
<summary>Hint</summary>

Kubernetes Secrets use base64 encoding under `data:` or plaintext under `stringData:`. For this exercise, use `stringData:`:

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: user-service-secret
stringData:
  DB_PASSWORD: "your-password-here"
```

In production, you would base64-encode the values or use an external secrets manager.

</details>

### Part D: Create the Kubernetes Deployment

Write a Deployment manifest (`deployment.yaml`) that:

1. References the ConfigMap for non-secret configuration.
2. References the Secret for sensitive values.
3. Injects ConfigMap values as environment variables using `envFrom`.
4. Injects Secret values individually using `secretKeyRef`.
5. Sets resource requests and limits.
6. Includes a readiness probe that hits `/health`.

<details>
<summary>Hint</summary>

Use `envFrom` to inject all ConfigMap keys as environment variables:

```yaml
envFrom:
  - configMapRef:
      name: user-service-secret
```

For the Secret, use individual `env` entries with `valueFrom.secretKeyRef`. This lets you control the variable name independently of the Secret key.

</details>

### Part E: Write a generation script

Create a script `generate-k8s-config.sh` that reads the `.env` file and generates a valid Kubernetes ConfigMap YAML. This makes the env file the single source of truth.

The script should:

1. Read each line from `.env`.
2. Skip comments and empty lines.
3. Convert each `KEY=VALUE` pair into a ConfigMap `data` entry.
4. Output valid YAML to stdout.

<details>
<summary>Hint</summary>

Use `grep -v '^#'` to skip comments and `grep -v '^$'` to skip empty lines. Then use `awk -F= '{print "  " $1 ": \"" $2 "\""}'` to format each line as a YAML key-value pair. Pipe the output into a heredoc or template that adds the ConfigMap header.

</details>

### Part F: Verify parity between Compose and Kubernetes

Write a verification script or set of commands that:

1. Prints the resolved environment for Docker Compose (`docker compose config`).
2. Prints the ConfigMap content (`kubectl get configmap user-service-config -o yaml`).
3. Compares the two to confirm they contain the same variables.

<details>
<summary>Hint</summary>

Extract variable names from both sources and use `diff` or `comm` to compare them. You do not need the values to be identical (Kubernetes might have extra metadata), but the variable names and non-secret values should match.

</details>

## Success Criteria

- [ ] Docker Compose starts all three services (app, postgres, redis) using env files.
- [ ] The ConfigMap contains the same non-secret variables as the `.env` file.
- [ ] The Secret contains the database password separately from the ConfigMap.
- [ ] The Deployment injects ConfigMap values via `envFrom` and Secret values via `secretKeyRef`.
- [ ] The generation script produces valid YAML from the `.env` file.
- [ ] You can explain the difference between ConfigMaps (for config) and Secrets (for credentials).
- [ ] The verification script confirms parity between Compose and Kubernetes configs.

## What You Should Understand After This Exercise

Configuration management scales by making one source of truth generate multiple targets. The `.env` file is your canonical config; tools transform it into Docker Compose env files, Kubernetes ConfigMaps, or CI/CD variables. ConfigMaps and env files solve the same problem at different scales -- both externalize configuration from the image. The critical distinction is between config (non-sensitive, in ConfigMaps) and secrets (sensitive, in Secrets or a secrets manager), which is the topic of Module 14.
