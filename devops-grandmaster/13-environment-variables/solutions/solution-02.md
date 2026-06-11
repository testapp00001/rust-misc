# Solution 02: Externalize Hardcoded Configuration

## Part A: Refactored app.py

```python
import os
from flask import Flask, jsonify

app = Flask(__name__)

# Read all configuration from environment variables
DATABASE_URL = os.environ.get("DATABASE_URL", "postgres://postgres@localhost:5432/myapp")
LOG_LEVEL = os.environ.get("LOG_LEVEL", "debug")
PORT = int(os.environ.get("PORT", "5000"))
REDIS_URL = os.environ.get("REDIS_URL", "redis://localhost:6379")
MAX_WORKERS = int(os.environ.get("MAX_WORKERS", "2"))


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

### Why this works

- `os.environ.get("KEY", "default")` returns the environment variable if set, otherwise returns the default.
- Defaults point to `localhost` for local development, not to production hosts.
- `PORT` and `MAX_WORKERS` are cast to `int` because environment variables are always strings.
- The `/config` endpoint returns the resolved values, making it easy to verify which config is active.

### Common mistakes

- Forgetting to cast to `int` -- `os.environ.get("PORT", "5000")` returns a string, and `app.run(port="5000")` may work in Flask but will fail in other frameworks.
- Using production hosts as defaults -- if the env var is missing, the app should connect to localhost (safe), not to prod-db (dangerous).
- Hardcoding the database URL with a password in the default -- even the default should be password-free for local dev.

---

## Part B: Clean Dockerfile

```dockerfile
FROM python:3.11-slim
WORKDIR /app
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt
COPY app.py .
EXPOSE 5000
CMD ["python", "app.py"]
```

### Why this works

- Zero `ENV` directives for application configuration. The image is a clean, environment-agnostic artifact.
- `EXPOSE 5000` documents the default port but does not set it. The actual port comes from the `PORT` environment variable at runtime.
- The image can be used in dev, staging, and production by passing different environment variables at `docker run` time.

### Common mistakes

- Keeping `ENV` directives "for documentation." Use `EXPOSE` or comments instead. `ENV` in the Dockerfile sets a default that is hard to override cleanly.
- Removing `EXPOSE` thinking it sets the port. `EXPOSE` is documentation only -- it does not publish or restrict the port.

---

## Part C: .env files

### .env (committed to git -- non-secret defaults)

```bash
# Application defaults for local development
PORT=5000
LOG_LEVEL=debug
REDIS_URL=redis://localhost:6379
MAX_WORKERS=2
```

### .env.local (gitignored -- secrets)

```bash
# Database connection with password (never commit this file)
DATABASE_URL=postgres://admin:password123@localhost:5432/myapp
```

### .env.example (committed -- template for new developers)

```bash
# Copy this file to .env.local and fill in your values
# cp .env.example .env.local

DATABASE_URL=postgres://user:changeme@localhost:5432/myapp
```

### .gitignore

```
.env.local
```

### Why this works

- `.env` contains only non-secret defaults. Safe to commit.
- `.env.local` contains the database password. Gitignored so it never enters version control.
- `.env.example` shows new developers what variables they need to set without revealing actual secrets.
- The three-file pattern (defaults, secrets, template) is a standard convention.

### Common mistakes

- Committing `.env.local` to git. Once a secret is in git history, it is there forever. Use `.gitignore` and check with `git status` before committing.
- Putting secrets in `.env` because "it is just for local development." Secrets in committed files get copied to every clone, every CI system, every backup.
- Forgetting to create `.env.example`. New developers will not know what variables to set.

---

## Part D: docker-compose.yml

```yaml
version: '3.8'

services:
  app:
    build: .
    ports:
      - "${PORT:-5000}:5000"
    environment:
      - PYTHONUNBUFFERED=1
    env_file:
      - .env
      - .env.local
    restart: unless-stopped
```

### Why this works

- `env_file` loads `.env` first, then `.env.local`. Variables in `.env.local` override those in `.env` because they are loaded second.
- `${PORT:-5000}` uses the `PORT` variable if set, otherwise defaults to 5000. This allows the host port to be configured via the env file.
- `PYTHONUNBUFFERED=1` is set in `environment:` because it is always needed regardless of env file content.
- The container port (5000) is hardcoded in the port mapping because the app always listens on 5000 internally (controlled by the `PORT` env var inside the container).

### Common mistakes

- Using `${PORT:-5000}:${PORT:-5000}` -- the second `PORT` refers to the *container's* port, which is controlled by the app's `PORT` env var. If you change `PORT=8080` in `.env`, the container listens on 8080 but the compose file still maps to 5000 inside. Map to the fixed container port instead.
- Listing `env_file` entries in the wrong order. Later files override earlier ones, so secrets (`.env.local`) should come after defaults (`.env`).
- Not using `:?` for required values. If `DATABASE_URL` must always be set, use `${DATABASE_URL:?DATABASE_URL is required}` to fail immediately rather than starting with a broken config.

---

## Part E: Test the configuration

```bash
# 1. Start with default configuration
docker compose up -d
curl http://localhost:5000/config
# Expected: LOG_LEVEL=debug, MAX_WORKERS=2, etc.

# 2. Override LOG_LEVEL at the command line
LOG_LEVEL=debug docker compose up -d
curl http://localhost:5000/config
# Expected: LOG_LEVEL=debug (from command line, highest priority)

# 3. Override MAX_WORKERS and verify
MAX_WORKERS=8 docker compose up -d
curl http://localhost:5000/config
# Expected: MAX_WORKERS=8

# 4. Stop and clean up
docker compose down
```

### Why this works

- Setting a variable before the `docker compose` command exports it to the shell environment. Docker Compose picks up shell environment variables with higher priority than env files.
- The `/config` endpoint returns the resolved values, confirming which source won the priority contest.
- This demonstrates the configuration hierarchy in action: shell env > env_file > code default.

### Common mistakes

- Forgetting that `docker compose up -d` in detached mode does not show startup errors. Use `docker compose logs` to check for problems.
- Not recreating the container after changing env files. `docker compose up -d` only recreates containers if the configuration changed. Use `docker compose up -d --force-recreate` if needed.
