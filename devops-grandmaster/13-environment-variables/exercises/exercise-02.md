# Exercise 02: Externalize Hardcoded Configuration

**Type:** Guided
**Time:** 30 minutes
**Difficulty:** Easy-Medium

## Objective

Take a Dockerfile and application with hardcoded configuration values and refactor them to use environment variables. This exercise trains the mechanical skill of separating configuration from code.

## Scenario

A developer left the team and handed you this project. Every configuration value is hardcoded directly in the Dockerfile and the application code. Your job is to fix it so the same image works for development, staging, and production without rebuilding.

## Starting Code

### Dockerfile (broken)

```dockerfile
FROM python:3.11-slim
WORKDIR /app
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt
COPY app.py .

# Hardcoded configuration -- this is the problem
ENV DATABASE_URL=postgres://admin:password123@prod-db:5432/myapp
ENV LOG_LEVEL=info
ENV PORT=8080
ENV REDIS_URL=redis://prod-redis:6379
ENV MAX_WORKERS=4

CMD ["python", "app.py"]
```

### app.py (broken)

```python
from flask import Flask, jsonify

app = Flask(__name__)

# Hardcoded -- cannot change without editing the file
DATABASE_URL = "postgres://admin:password123@prod-db:5432/myapp"
LOG_LEVEL = "info"
PORT = 8080
REDIS_URL = "redis://prod-redis:6379"
MAX_WORKERS = 4

@app.route("/config")
def config():
    return jsonify({
        "database_url": DATABASE_URL,
        "log_level": LOG_LEVEL,
        "port": PORT,
        "redis_url": REDIS_URL,
        "max_workers": MAX_WORKERS,
    })

@app.route("/health")
def health():
    return jsonify({"status": "ok"})

if __name__ == "__main__":
    app.run(host="0.0.0.0", port=PORT)
```

## Tasks

### Part A: Refactor app.py

Rewrite `app.py` so that every configuration value is read from an environment variable at startup. Each variable should have a sensible default that works for local development (localhost, debug-level logging, etc.).

<details>
<summary>Hint</summary>

Use `os.environ.get("VAR_NAME", "default_value")` for each variable. The default for `DATABASE_URL` should point to `localhost`, not to a production host.

</details>

### Part B: Clean the Dockerfile

Rewrite the Dockerfile so it contains zero configuration values. It should define only the image base, working directory, dependencies, exposed port, and startup command. All configuration comes from environment variables at runtime.

<details>
<summary>Hint</summary>

Remove every `ENV` line that sets a configuration value. Keep only `EXPOSE` and `CMD`. The `EXPOSE` directive documents which port the app uses but does not set it -- that comes from the `PORT` environment variable.

</details>

### Part C: Create the .env files

Create three files:

1. `.env` -- Default values suitable for development (committed to git).
2. `.env.local` -- Secrets like database passwords (gitignored).
3. `.env.example` -- Template showing what values are needed (committed to git).

<details>
<summary>Hint</summary>

The `.env` file should contain non-secret defaults like `PORT=5000` and `LOG_LEVEL=debug`. The `.env.local` file should contain `DATABASE_URL` with the real password. The `.env.example` file should show the structure with placeholder values like `changeme`.

</details>

### Part D: Create docker-compose.yml

Write a `docker-compose.yml` that:

- Builds the app from the local Dockerfile.
- Loads environment variables from `.env` and `.env.local`.
- Uses `${VAR:-default}` syntax for overridable values.
- Uses `${VAR:?error}` syntax to require the database URL.
- Exposes the app port using the `PORT` variable.

<details>
<summary>Hint</summary>

The `env_file` directive loads files in order -- later files override earlier ones. Use `environment:` for values that should always be set regardless of env files (like `PYTHONUNBUFFERED=1`).

</details>

### Part E: Test the configuration

Describe the commands you would run to verify that:

1. The app starts with default configuration.
2. You can override `LOG_LEVEL` at the command line.
3. The `/config` endpoint reflects the overridden value.

<details>
<summary>Hint</summary>

Use `docker compose up -d`, then `curl http://localhost:5000/config`. To override a variable, set it before the `docker compose` command: `LOG_LEVEL=debug docker compose up -d`.

</details>

## Success Criteria

- [ ] `app.py` reads all five configuration values from environment variables.
- [ ] Each variable has a sensible default for local development.
- [ ] The Dockerfile contains zero hardcoded configuration values.
- [ ] Three `.env` files exist with correct content and purpose.
- [ ] `docker-compose.yml` uses `env_file`, `${VAR:-default}`, and `${VAR:?error}` correctly.
- [ ] You can describe how to test that overrides work.

## What You Should Understand After This Exercise

Externalizing configuration is a mechanical refactoring: replace hardcoded values with `os.environ.get()` calls (or your language's equivalent), remove `ENV` directives from the Dockerfile that set business configuration, and wire everything through env files and docker-compose. The image stays the same; only the runtime values change.
