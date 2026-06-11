# Solution 05: Dev, Test, and Prod -- A Three-Stage Pipeline

## Part A: Stage Architecture

The dependency graph:

```
builder (installs ALL packages including dev tools)
    |
    +---> dev   (copies everything from builder, adds debug config)
    |
    +---> test  (copies everything from builder, adds test config)
    |
    +---> prod  (copies ONLY production packages from builder-prod)

builder-prod (installs ONLY production packages)
    |
    +---> prod
```

**Key design decisions:**

1. `builder` installs everything (Flask, gunicorn, pytest, debugpy, flake8, black) with `--prefix=/install`.
2. `builder-prod` installs only production packages (Flask, gunicorn) with `--prefix=/install`.
3. `dev` and `test` copy from `builder` (they need dev tools).
4. `prod` copies from `builder-prod` (it must not contain test or dev tools).
5. `dev`, `test`, and `prod` do not depend on each other -- they can be built independently.

**Can stages be built in parallel?**
- `dev` and `test` can be built in parallel (both depend only on `builder`).
- `prod` can be built in parallel with `dev` and `test` (it depends on `builder-prod`, not `builder`).
- `builder` and `builder-prod` can be built in parallel (they do not depend on each other).
- Docker BuildKit automatically parallelizes independent stages.

---

## Part B: The Builder Stage

```dockerfile
# ===== Shared Builder (all packages) =====
FROM python:3.12 AS builder

WORKDIR /app
COPY requirements-dev.txt .
RUN pip install --no-cache-dir --prefix=/install -r requirements-dev.txt
```

**Why `requirements-dev.txt` and not `requirements.txt`:**

The `requirements-dev.txt` file includes `-r requirements.txt`, so it installs everything: production packages (Flask, gunicorn) AND dev tools (pytest, debugpy, flake8, black). The `dev` and `test` targets need all of these.

**Why `--prefix=/install`:**

This puts all installed packages in `/install/lib/python3.12/site-packages/` and console scripts in `/install/bin/`. When copied to `/usr/local` in the runtime stage, they land in the correct Python path locations.

---

## Part C: The Production Stage

```dockerfile
# ===== Production Builder (production packages only) =====
FROM python:3.12 AS builder-prod

WORKDIR /app
COPY requirements.txt .
RUN pip install --no-cache-dir --prefix=/install -r requirements.txt

# ===== Production =====
FROM python:3.12-slim AS prod

ENV ENVIRONMENT=production
WORKDIR /app

COPY --from=builder-prod /install /usr/local
COPY . .

RUN useradd --create-home --no-log-init appuser
USER appuser

EXPOSE 5000
CMD ["gunicorn", "--bind", "0.0.0.0:5000", "app:app"]
```

**Why a separate `builder-prod` stage:**

If `prod` copied from `builder`, it would get pytest, debugpy, flake8, and black -- all unnecessary in production. A separate builder that installs only `requirements.txt` ensures the production image contains nothing extra.

**Why `requirements.txt` (not `requirements-dev.txt`):**

`requirements.txt` contains only Flask and gunicorn. `requirements-dev.txt` extends it with testing and development tools. The production image should have neither.

**Why gunicorn:**
- Flask's built-in server is for development only.
- Gunicorn is a production-grade WSGI server.
- It handles concurrent requests, worker management, and graceful restarts.

**Expected size: ~170MB**

---

## Part D: The Development Stage

```dockerfile
# ===== Development =====
FROM python:3.12-slim AS dev

ENV ENVIRONMENT=development
WORKDIR /app

# Copy ALL packages (including dev tools)
COPY --from=builder /install /usr/local
COPY . .

EXPOSE 5000
EXPOSE 5678

CMD ["python", "-m", "debugpy", "--listen", "0.0.0.0:5678", \
     "--wait-for-client", \
     "-m", "flask", "run", "--host=0.0.0.0", "--debug"]
```

**What this includes:**
- Flask and gunicorn (production packages)
- pytest and pytest-cov (test frameworks)
- debugpy (remote debugger)
- flake8 (linter)
- black (formatter)
- Application source code

**Port 5678:** This is the debugpy debug port. IDEs like VS Code connect to this port for remote debugging with breakpoints, variable inspection, and step-through execution.

**`--wait-for-client`:** Optional flag that pauses the application until a debugger connects. Remove this if you want the app to start immediately without waiting for a debugger.

**Flask's `--debug` flag:** Enables hot-reload -- the server automatically restarts when you modify source files. This is essential for development but must not be used in production.

**Why copy from `builder` (not `builder-prod`):**
- Developers need pytest for running tests locally.
- Developers need debugpy for remote debugging.
- Developers need flake8 and black for linting and formatting.
- The dev image is larger than prod, and that is acceptable.

**Expected size: ~250MB**

---

## Part E: The Test Stage

```dockerfile
# ===== Test =====
FROM python:3.12-slim AS test

ENV ENVIRONMENT=test
WORKDIR /app

# Copy ALL packages (test framework needed)
COPY --from=builder /install /usr/local
COPY . .

CMD ["python", "-m", "pytest", "--cov=.", "-v"]
```

**What this includes:**
- Flask and gunicorn (production packages)
- pytest and pytest-cov (test frameworks)
- debugpy, flake8, black (not strictly needed for tests, but present because they come from the same builder)

**What it does NOT include that `dev` has:**
- No special debug configuration
- No hot-reload setup
- No debug port exposure

**Why `--cov=.` and `-v`:**
- `--cov=.` enables code coverage reporting for the current directory.
- `-v` enables verbose output, showing each test name and its result.
- Together they give a clear picture of test results and coverage.

**`ENVIRONMENT=test`:** The application can use this to adjust behavior in test mode (e.g., use a test database, disable external API calls).

**Expected size: ~250MB**

---

## Part F: Complete Dockerfile

```dockerfile
# ===== Builder (all packages) =====
FROM python:3.12 AS builder

WORKDIR /app
COPY requirements-dev.txt .
RUN pip install --no-cache-dir --prefix=/install -r requirements-dev.txt

# ===== Builder (production packages only) =====
FROM python:3.12 AS builder-prod

WORKDIR /app
COPY requirements.txt .
RUN pip install --no-cache-dir --prefix=/install -r requirements.txt

# ===== Production =====
FROM python:3.12-slim AS prod

ENV ENVIRONMENT=production
WORKDIR /app
COPY --from=builder-prod /install /usr/local
COPY . .

RUN useradd --create-home --no-log-init appuser
USER appuser

EXPOSE 5000
CMD ["gunicorn", "--bind", "0.0.0.0:5000", "app:app"]

# ===== Development =====
FROM python:3.12-slim AS dev

ENV ENVIRONMENT=development
WORKDIR /app
COPY --from=builder /install /usr/local
COPY . .

EXPOSE 5000
EXPOSE 5678
CMD ["python", "-m", "debugpy", "--listen", "0.0.0.0:5678", \
     "--wait-for-client", \
     "-m", "flask", "run", "--host=0.0.0.0", "--debug"]

# ===== Test =====
FROM python:3.12-slim AS test

ENV ENVIRONMENT=test
WORKDIR /app
COPY --from=builder /install /usr/local
COPY . .

CMD ["python", "-m", "pytest", "--cov=.", "-v"]
```

**Build and compare:**

```bash
cd /tmp/exercise-05

# Build each target
docker build --target prod -t exercise5-prod .
docker build --target test -t exercise5-test .
docker build --target dev -t exercise5-dev .

# Compare sizes
docker images | grep exercise5
```

**Expected sizes:**

| Target | Image Size | Contents |
|---|---|---|
| prod | ~170MB | Flask + gunicorn only |
| test | ~250MB | Flask + pytest + coverage tools |
| dev | ~250MB | Flask + pytest + debugpy + linters + formatters |

---

## Part G: Verify Each Target

```bash
# Production
docker run -p 5000:5000 -e ENVIRONMENT=production exercise5-prod &
curl http://localhost:5000
# Expected: "Hello from production!"
curl http://localhost:5000/health
# Expected: {"status": "ok", "env": "production"}

# Tests
docker run exercise5-test
# Expected: pytest output showing 2 tests passed with coverage report

# Development
docker run -p 5000:5000 -p 5678:5678 exercise5-dev &
curl http://localhost:5000
# Expected: "Hello from development!"
# Connect VS Code debugger to localhost:5678 for remote debugging
```

**How `--target` works:**

```bash
docker build --target prod -t exercise5-prod .
```

The `--target` flag tells Docker to build up to and including the named stage, and use that stage as the final image. Docker still builds all stages that the target depends on (like `builder-prod` for `prod`), but stages after the target (like `dev` and `test`) are not built.

Without `--target`, Docker builds the LAST stage in the Dockerfile (in this case, `test`). This is why the order of stages matters when you do not use `--target`.

---

## Common Mistakes

1. **Using one builder for both dev and prod packages.** If `prod` copies from the same `builder` as `dev`, it gets pytest, debugpy, and all other dev tools. Use a separate `builder-prod` stage that installs only `requirements.txt`.

2. **Forgetting `--target` when building specific stages.** Without `--target`, Docker builds the last stage. If you want the production image, you must specify `--target prod`. Otherwise you get the test image (since it is last in the Dockerfile).

3. **Not setting `ENVIRONMENT` for each target.** The application uses `ENVIRONMENT` to adjust behavior. Without it, all targets behave the same way, defeating the purpose of having separate images.

4. **Running tests in the production image.** The production image should not contain pytest. Tests should run in the `test` target, which is built specifically for that purpose.

5. **Using Flask's dev server in production.** The `dev` target uses `flask run --debug` for hot-reload. The `prod` target must use gunicorn. Never use Flask's built-in server for production traffic.

6. **Exposing debug port in production.** Port 5678 (debugpy) should only be exposed in the `dev` target. Exposing it in production is a security risk -- it allows remote code execution.

7. **Copying source code before packages.** In all stages, `COPY --from=builder /install /usr/local` should come before `COPY . .`. This way, if only source code changes, the package copy layer is cached.

8. **Not using `.dockerignore`.** All three targets use `COPY . .`, which copies the entire build context. Without a `.dockerignore`, this includes `.git`, `__pycache__`, `.env`, and other files that should not be in the image.

9. **Confusing the stage order.** The last stage in the Dockerfile is the default target. If you want `prod` to be the default, put it last. If you always use `--target`, the order does not matter, but it is good practice to put the most common target last.

10. **Not using BuildKit.** BuildKit (enabled by default in modern Docker) can build independent stages in parallel. Without BuildKit, stages are built sequentially, which is slower. Ensure `DOCKER_BUILDKIT=1` is set or use `docker buildx build`.
