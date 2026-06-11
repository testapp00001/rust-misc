# Exercise 03: Environment-Specific Configuration

**Type:** Independent
**Time:** 30 minutes
**Difficulty:** Medium

## Objective

Design and implement a configuration strategy that supports three environments (development, staging, production) using a single Docker image and environment-specific files. This exercise trains you to think about configuration as a deployment concern, not a code concern.

## Scenario

Your team runs the same application in three environments. Each environment has different database hosts, log levels, feature flags, and resource limits. Currently, the team maintains three branches (`dev`, `staging`, `main`) with different hardcoded values in each. This causes merge conflicts, accidental deployments of debug code, and constant confusion about which branch is "correct."

Your task: design a configuration structure so that one image, one branch, and one codebase serve all three environments.

## Requirements

The application needs the following configuration:

| Variable | Development | Staging | Production |
|----------|-------------|---------|------------|
| `NODE_ENV` | development | staging | production |
| `LOG_LEVEL` | debug | info | warn |
| `DB_HOST` | localhost | staging-db.internal | prod-db.internal |
| `DB_PORT` | 5432 | 5432 | 5432 |
| `DB_NAME` | myapp_dev | myapp_staging | myapp_prod |
| `DB_USER` | postgres | app_staging | app_prod |
| `DB_PASSWORD` | (empty) | stg-pass-123 | prod-pass-xyz |
| `ENABLE_DEBUG_UI` | true | false | false |
| `MAX_CONNECTIONS` | 5 | 20 | 100 |
| `CACHE_TTL` | 60 | 300 | 3600 |

## Tasks

### Part A: Design the file structure

Decide which values go in which files. Create the following files with appropriate content:

- `.env.defaults` -- Shared values across all environments (committed to git).
- `.env.development` -- Development-only overrides (committed to git).
- `.env.staging` -- Staging overrides (committed, with placeholder for password).
- `.env.production` -- Production overrides (committed, with placeholder for password).
- `.env.secrets` -- Actual passwords (gitignored).

<details>
<summary>Hint</summary>

Values that are the same across all environments (like `DB_PORT=5432`) go in `.env.defaults`. Values that differ per environment go in the environment-specific file. Passwords always go in `.env.secrets`. No secret should ever appear in a file that is committed to git.

</details>

### Part B: Create a docker-compose file per environment

Create three docker-compose override files:

- `docker-compose.yml` -- Base configuration shared across all environments.
- `docker-compose.dev.yml` -- Development overrides (mount source code, enable debug).
- `docker-compose.staging.yml` -- Staging overrides.
- `docker-compose.prod.yml` -- Production overrides (resource limits, restart policy).

Each file should load the correct env files for its environment.

<details>
<summary>Hint</summary>

The base `docker-compose.yml` defines the service structure. Override files use `docker compose -f docker-compose.yml -f docker-compose.dev.yml up` to layer configurations. Each override file should reference the appropriate env files via the `env_file` directive.

</details>

### Part C: Write startup scripts

Create a shell script `start.sh` that accepts an environment argument and starts the correct compose configuration:

```bash
./start.sh dev
./start.sh staging
./start.sh prod
```

The script should validate that the required env files exist before starting.

<details>
<summary>Hint</summary>

Use a `case` statement to map the argument to the correct compose file. Check for file existence with `[ -f filename ]`. Print a clear error message if a required file is missing.

</details>

### Part D: Verify environment isolation

Write the commands you would run to verify that:

1. Development starts with `LOG_LEVEL=debug` and `ENABLE_DEBUG_UI=true`.
2. Production starts with `LOG_LEVEL=warn` and `MAX_CONNECTIONS=100`.
3. The same Docker image is used in both cases.

<details>
<summary>Hint</summary>

Use `docker compose config` to print the resolved configuration without actually starting containers. This shows the final merged values from all env files and compose overrides. Compare the output for each environment.

</details>

## Success Criteria

- [ ] No secret appears in any file that would be committed to git.
- [ ] Shared values (like `DB_PORT`) are defined once in `.env.defaults`, not repeated per environment.
- [ ] Each environment loads exactly the right set of env files.
- [ ] `docker compose config` shows different resolved values for dev vs. prod.
- [ ] The startup script validates file existence and gives clear error messages.
- [ ] A single Docker image (no rebuild) works for all three environments.

## What You Should Understand After This Exercise

Environment-specific configuration is a file organization problem, not a code problem. By splitting configuration into layers (defaults, environment overrides, secrets), you get a clean separation where changing an environment's behavior means editing a text file, not rebuilding an image. The `docker compose -f` flag lets you layer configurations without duplication.
