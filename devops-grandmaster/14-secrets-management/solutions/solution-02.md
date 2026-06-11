# Solution 02: Docker Secrets for a Swarm Service

## Part A: Initialize Docker Swarm

```bash
docker swarm init
```

### Why this works

`docker swarm init` creates a single-node Swarm cluster on the local machine. The current node becomes a manager. Docker secrets are only available in Swarm mode -- in standalone Docker, the `docker secret` commands do not exist. A single-node Swarm is sufficient for development and testing. In production, you would have multiple manager and worker nodes.

### Common mistakes

- Running `docker swarm init` on a machine that is already part of a Swarm. Check with `docker info | grep Swarm` first. If it says "active," you are already in a Swarm.
- Thinking you need multiple machines for this exercise. A single-node Swarm supports all secret features.
- Forgetting to initialize Swarm and then wondering why `docker secret create` fails with "this node is not a swarm manager."

---

## Part B: Create Docker Secrets

```bash
# Create the database password secret
echo "super-secret-password-123!" | docker secret create db_password -

# Create the database user secret
echo "appuser" | docker secret create db_user -

# List secrets to verify
docker secret ls

# Inspect a secret (shows metadata, not the value)
docker secret inspect db_password
```

### Why this works

- `echo "value" | docker secret create name -` reads the secret value from stdin (the `-` argument).
- `docker secret ls` shows all secrets with their ID, name, creation time, and update time.
- `docker secret inspect` shows metadata (name, ID, creation date, labels) but never shows the secret value. There is no `docker secret get` command -- the value is write-only from the CLI's perspective.

### Common mistakes

- Using `echo` without `-n` and including a trailing newline in the secret. Use `echo -n "value"` or `printf "value"` to avoid this. However, for this exercise the trailing newline is acceptable since the application should `.trim()` the value.
- Trying to read the secret value back from Docker. This is by design -- Docker intentionally prevents reading secret values after creation. If you need to verify the value, you must test it through the application.
- Creating secrets outside of Swarm mode. If you get "this node is not a swarm manager," run `docker swarm init` first.

---

## Part C: Updated Stack File

```yaml
version: '3.8'

services:
  db:
    image: postgres:15-alpine
    environment:
      POSTGRES_USER_FILE: /run/secrets/db_user
      POSTGRES_PASSWORD_FILE: /run/secrets/db_password
      POSTGRES_DB: myapp
    secrets:
      - db_password
      - db_user
    volumes:
      - db_data:/var/lib/postgresql/data

  app:
    image: nginx:alpine
    ports:
      - "8080:80"
    environment:
      DB_HOST: db
      DB_NAME: myapp
      DB_USER_FILE: /run/secrets/db_user
      DB_PASSWORD_FILE: /run/secrets/db_password
    secrets:
      - db_password
      - db_user

secrets:
  db_password:
    external: true
  db_user:
    external: true

volumes:
  db_data:
```

### Why this works

- The top-level `secrets:` block references secrets created with `docker secret create` using `external: true`. This tells Docker Compose that the secrets already exist in the Swarm and should not be created from files.
- Each service's `secrets:` list mounts the named secrets as files at `/run/secrets/<secret_name>`.
- Non-sensitive configuration (`DB_HOST`, `DB_NAME`, `POSTGRES_DB`) remains as environment variables.
- The `_FILE` suffix on environment variables (e.g., `POSTGRES_PASSWORD_FILE`) is a convention used by the official PostgreSQL image to read secrets from files. The image reads the file at the specified path and uses its contents as the variable's value.
- The actual secret values do not appear anywhere in the compose file.

### Common mistakes

- Omitting `external: true` and letting Docker Compose try to create the secrets from files. If you created secrets with `docker secret create`, they are external.
- Putting the secret value in the compose file under `secrets: db_password: file: ./password.txt`. This works for Docker Compose (non-Swarm), but the exercise requires Docker Swarm secrets.
- Forgetting that secrets are per-service. If both `db` and `app` need the same secret, both must list it in their `secrets:` section.
- Using `POSTGRES_PASSWORD` (without `_FILE`) and expecting Docker to read from `/run/secrets/`. The `_FILE` suffix is a feature of the official PostgreSQL image, not a Docker feature.

---

## Part D: Secret Reading Script

```bash
#!/bin/bash
# read-secrets.sh -- Demonstrates reading Docker secrets from files

set -euo pipefail

SECRET_DIR="/run/secrets"

# Check that the secret directory exists
if [ ! -d "$SECRET_DIR" ]; then
    echo "ERROR: Secret directory $SECRET_DIR does not exist."
    echo "Are you running inside a Docker Swarm service with secrets mounted?"
    exit 1
fi

# Read database password
DB_PASSWORD_FILE="$SECRET_DIR/db_password"
if [ ! -f "$DB_PASSWORD_FILE" ]; then
    echo "ERROR: Secret file $DB_PASSWORD_FILE not found."
    exit 1
fi
DB_PASSWORD=$(cat "$DB_PASSWORD_FILE" | tr -d '\n')

# Read database user
DB_USER_FILE="$SECRET_DIR/db_user"
if [ ! -f "$DB_USER_FILE" ]; then
    echo "ERROR: Secret file $DB_USER_FILE not found."
    exit 1
fi
DB_USER=$(cat "$DB_USER_FILE" | tr -d '\n')

# Print confirmation (never print the actual secret)
echo "Secrets loaded successfully:"
echo "  db_user:     $DB_USER (${#DB_USER} chars)"
echo "  db_password: [REDACTED] (${#DB_PASSWORD} chars)"
```

### Why this works

- `set -euo pipefail` ensures the script exits on any error, undefined variable, or pipe failure.
- `cat "$FILE" | tr -d '\n'` reads the file and strips trailing newlines. Docker secrets may or may not have a trailing newline depending on how they were created.
- `${#VARIABLE}` returns the length of the string in bash, allowing verification without exposing the value.
- The script checks for the existence of the secret directory and each secret file before attempting to read, providing clear error messages if secrets are not mounted.

### Common mistakes

- Not stripping newlines from the secret value. If the secret was created with `echo "value"` (which adds a newline), the password will be `"value\n"` instead of `"value"`, and database authentication will fail.
- Printing the actual secret value for debugging. This defeats the purpose of using secrets. Always print only the length or a masked version.
- Not using `set -euo pipefail`. Without it, a missing file might cause the script to continue with an empty variable, leading to silent authentication failures.

---

## Part E: Deploy and Verify

```bash
# Deploy the stack
docker stack deploy -c docker-compose.yml myapp

# Wait for services to start
sleep 5

# List the services
docker service ls

# Verify secrets are mounted in the db service
docker exec $(docker ps -q -f name=myapp_db) ls -la /run/secrets/
# Expected output:
# -r--r--r--  1 root root  ... db_password
# -r--r--r--  1 root root  ... db_user

# Verify secrets are mounted in the app service
docker exec $(docker ps -q -f name=myapp_app) ls -la /run/secrets/

# Verify docker inspect does NOT reveal secret values
docker inspect $(docker ps -q -f name=myapp_db) | grep -i "password"
# Expected: No matches for the actual password string

# Verify docker inspect shows the secret mount (metadata only)
docker inspect $(docker ps -q -f name=myapp_db) | grep -A 5 "Secrets"

# Test the application
curl http://localhost:8080
```

### Why this works

- `docker stack deploy` creates services defined in the compose file. The `secrets:` directive in each service mounts the pre-created Docker secrets.
- `ls -la /run/secrets/` inside the container shows the secret files mounted as tmpfs (read-only, root-owned).
- `docker inspect` shows that secrets are mounted but does not reveal their values. The secret data is encrypted in the Swarm's Raft log and only decrypted inside the container's tmpfs.
- The `grep` for the password string confirms that the plaintext value does not appear in any Docker metadata.

### Common mistakes

- Trying to run `docker stack deploy` without initializing Swarm first. You will get an error about Swarm mode.
- Confusing `docker compose up` (standalone) with `docker stack deploy` (Swarm). Standalone Docker Compose does not support Docker Swarm secrets -- it supports file-based secrets only.
- Not waiting for services to be ready before running verification commands. Use `docker service ls` to check that replicas are running.
- Running `docker inspect` on the wrong container. Use `docker ps -f name=myapp_db` to find the correct container ID.
