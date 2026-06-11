# Solution 02: Fix the Dockerfile That Rebuilds Everything

## Part A: Root Cause

The problem is line 5: `COPY . .`

```dockerfile
FROM python:3.11      # Line 1
WORKDIR /app           # Line 2
COPY . .               # Line 3 ← THIS copies everything (including src/)
RUN pip install ...    # Line 4 ← Rebuilds when Line 3 changes
RUN pip install -e .   # Line 5 ← Rebuilds when Line 4 changes
CMD ["python", ...]    # Line 6
```

When `src/routes.py` changes, `COPY . .` detects the change (because it
copies all files, including `src/`). This invalidates the `COPY . .` layer,
which cascades to every layer after it -- including `pip install -r requirements.txt`.

The dependencies have not changed (`requirements.txt` is the same), but Docker
does not know that because it already invalidated the layer that runs `pip install`.

## Part B: Reordered Dockerfile

```dockerfile
FROM python:3.11-slim

WORKDIR /app

# 1. Copy dependency files first (changes rarely)
COPY requirements.txt .
COPY setup.py .

# 2. Install dependencies (cached until requirements.txt changes)
RUN pip install --no-cache-dir -r requirements.txt
RUN pip install -e .

# 3. Copy source code last (changes constantly)
COPY src/ ./src/

CMD ["python", "src/app.py"]
```

Why this works:

```
When src/routes.py changes:
  FROM python:3.11-slim              CACHED
  WORKDIR /app                        CACHED
  COPY requirements.txt .             CACHED (requirements.txt unchanged)
  COPY setup.py .                     CACHED (setup.py unchanged)
  RUN pip install -r requirements.txt CACHED (previous layer cached)
  RUN pip install -e .                CACHED (previous layer cached)
  COPY src/ ./src/                    REBUILT (src/routes.py changed)
  CMD ["python", "src/app.py"]        CACHED
```

Only the `COPY src/` layer rebuilds. The expensive `pip install` steps are cached.

## Part C: Base Image Fix

```dockerfile
FROM python:3.11-slim
```

Use `python:3.11-slim` instead of `python:3.11`:

| Variant | Size | Notes |
|---------|------|-------|
| `python:3.11` | ~900MB | Full Debian, includes build tools |
| `python:3.11-slim` | ~150MB | Minimal Debian, no build tools |
| `python:3.11-alpine` | ~50MB | Alpine Linux, musl libc |

Why not Alpine: Alpine uses musl libc instead of glibc. Some Python packages
(numpy, pandas, psycopg2) require compilation on Alpine, which makes builds
slower and can cause compatibility issues. `slim` is the best balance of
size and compatibility.

## Part D: .dockerignore

```
.git
.gitignore
.env
.env.local
.DS_Store
__pycache__
*.pyc
*.pyo
.pytest_cache
.coverage
htmlcov
.mypy_cache
.ruff_cache
*.egg-info
dist
build
.venv
venv
node_modules
*.md
Dockerfile*
.dockerignore
```

Why each entry matters:

- `.git/` -- can be hundreds of MB and changes on every commit
- `__pycache__/`, `*.pyc` -- compiled Python, regenerated on build
- `.venv/`, `venv/` -- local virtual environments, not needed in container
- `*.md`, `Dockerfile*` -- documentation, not needed at runtime
- `.env` -- secrets should not be in the image

## Part E: BuildKit Cache Mount

```dockerfile
# syntax=docker/dockerfile:1

FROM python:3.11-slim

WORKDIR /app

# System dependencies (rarely changes)
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    gcc \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Python dependencies (changes rarely)
COPY requirements.txt .
RUN --mount=type=cache,target=/root/.cache/pip \
    pip install --no-cache-dir -r requirements.txt

# Application package (changes sometimes)
COPY setup.py .
RUN pip install -e .

# Source code (changes constantly)
COPY src/ ./src/

CMD ["python", "src/app.py"]
```

How the cache mount works:

```
Build 1 (cold cache):
  pip downloads all packages from PyPI → stores in /root/.cache/pip
  Time: 180 seconds

Build 2 (requirements.txt unchanged):
  Layer cached normally → pip install step skipped entirely
  Time: 5 seconds

Build 3 (requirements.txt changed, added one package):
  Layer invalidated → pip runs again
  But: cache mount still has all previously downloaded packages
  pip only downloads the new package
  Time: 20 seconds (instead of 180 seconds)
```

Without cache mount: Build 3 takes 180 seconds (re-downloads everything).
With cache mount: Build 3 takes 20 seconds (reuses cached downloads).

## Part F: Verification

```bash
# Step 1: Clean build (cold cache)
time docker build -t analytics-service .

# Step 2: Change a source file
echo "# comment" >> src/routes.py

# Step 3: Rebuild (warm cache)
time docker build -t analytics-service .
```

Expected results:

```
Cold build:  real  3m12s  (full dependency install)
Warm build:  real  0m08s  (only COPY src/ layer rebuilds)
```

To see exactly which layers are cached:

```bash
docker build --progress=plain -t analytics-service . 2>&1 | grep -E "CACHED|DONE"
```

## Common Mistakes

1. **Forgetting `# syntax=docker/dockerfile:1`.** Without this directive,
   Docker uses the legacy builder which does not support `--mount=type=cache`.
   The build will fail with a syntax error.

2. **Caching the wrong directory.** Mounting `/app` instead of `/root/.cache/pip`
   would cache the application code, not the pip downloads. This breaks the build.

3. **Using `pip install` without `--no-cache-dir`.** When using cache mounts,
   always use `--no-cache-dir` to avoid storing packages both in the layer and
   in the cache mount (doubling storage).

4. **Not pinning base image versions.** Using `python:3.11` without the `-slim`
   suffix means you get a 900MB image. Using `python:latest` means your cache
   invalidates whenever a new Python version is released.
