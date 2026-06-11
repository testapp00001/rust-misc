# Solution 04: The Bloated Dockerfile Diet

## Part A: Identified Problems

Here are all the problems in the bloated Dockerfile, categorized:

### 1. `FROM python:latest` -- Performance, Best Practice
**What:** Uses the `latest` tag instead of a specific version.
**Why it matters:** The `latest` tag can change at any time when Python releases a new version. A build that worked yesterday might break today. Builds are not reproducible.
**Fix:** `FROM python:3.11-slim-bookworm`

### 2. `FROM python:latest` (full image) -- Performance
**What:** Uses the full Python image (~900MB) instead of the slim variant (~150MB).
**Why it matters:** The full image includes compilers, development headers, man pages, and other packages that a runtime application does not need. This wastes storage, slows pulls, and increases attack surface.
**Fix:** Use `python:3.11-slim-bookworm` or `python:3.11-alpine`.

### 3. Three separate `apt-get install` RUN layers -- Performance
**What:** Each `apt-get install` is a separate RUN instruction, creating three layers.
**Why it matters:** Each layer adds overhead. More importantly, each `apt-get update` re-downloads the package index. Combining them into one RUN reduces layers and build time.
**Fix:** Combine into a single `RUN` with `&&`.

### 4. No `apt-get clean` or `rm -rf /var/lib/apt/lists/*` -- Performance
**What:** The apt cache is left in the image after installing packages.
**Why it matters:** The apt cache can add 30-100MB of unnecessary data to the image. This data is only needed during installation, not at runtime.
**Fix:** Add `&& rm -rf /var/lib/apt/lists/*` at the end of the apt-get RUN command.

### 5. Installing `curl`, `vim`, `git` -- Security, Performance
**What:** Installs packages that are not needed by the application at runtime.
**Why it matters:** Every package increases the image size and the attack surface. `vim` and `git` are development tools, not runtime dependencies. `curl` is only needed if the app or health check uses it.
**Fix:** Remove all three. If a health check needs HTTP, use Python's `urllib` instead.

### 6. Separate `pip install` commands -- Performance
**What:** Each `pip install` is a separate RUN instruction, creating four layers.
**Why it matters:** Each layer is cached independently, but more layers means more overhead. Combining them into one command (or better, using `requirements.txt`) reduces layers.
**Fix:** Use `COPY requirements.txt . && RUN pip install --no-cache-dir -r requirements.txt`.

### 7. No `--no-cache-dir` for pip -- Performance
**What:** pip stores downloaded packages in a cache directory inside the image.
**Why it matters:** The pip cache can add 50-100MB to the image. It is never used at runtime.
**Fix:** Add `--no-cache-dir` to the pip install command.

### 8. `COPY . /app` before `WORKDIR /app` -- Correctness, Performance
**What:** `COPY` uses an absolute path instead of relative, and it comes before `WORKDIR`.
**Why it matters:** This works functionally, but it means `WORKDIR` is set after the code is copied. More importantly, because `COPY . /app` comes after all the `RUN` commands, any change to the application code invalidates the cache for the COPY layer but does not affect the earlier layers. However, if the dependencies were in a single `requirements.txt` and installed earlier, this would be fine. The real problem is that `COPY .` is the only code copy, so there is no dependency-file-first caching.
**Fix:** Use `WORKDIR /app` first, then `COPY requirements.txt .`, then `RUN pip install`, then `COPY . .`.

### 9. `useradd -m appuser` without `USER appuser` -- Security
**What:** Creates a user but never switches to it.
**Why it matters:** The container still runs as root. The `useradd` command does nothing useful without the `USER` instruction. This is a false sense of security -- someone reading the Dockerfile might think it runs as non-root.
**Fix:** Add `USER appuser` after the user creation.

### 10. `CMD python app.py` (shell form) -- Correctness
**What:** Uses shell form for CMD instead of exec form.
**Why it matters:** Shell form wraps the command in `/bin/sh -c`. This means the Python process is not PID 1 and does not receive signals directly. `docker stop` sends SIGTERM to the shell, which may not forward it to Python, causing a 10-second delay before SIGKILL.
**Fix:** Use `CMD ["python", "app.py"]` or better, use gunicorn.

### 11. Using Flask dev server in production -- Correctness
**What:** Runs `python app.py` which starts Flask's built-in development server.
**Why it matters:** Flask's dev server is single-threaded, not optimized for production workloads, and not designed to handle concurrent requests. It explicitly warns "Do not use the development server in a production setting."
**Fix:** Use `CMD ["gunicorn", "--bind", "0.0.0.0:5000", "app:app"]`.

### 12. No `.dockerignore` -- Performance, Security
**What:** The `.dockerignore` file is not mentioned.
**Why it matters:** Without it, `COPY . .` copies everything, including `.git/`, `__pycache__/`, `.env` (with secrets), `node_modules/`, test files, and documentation. This bloats the image and can leak sensitive data.
**Fix:** Create a `.dockerignore` file.

---

## Part B: Optimized Dockerfile

```dockerfile
FROM python:3.11-slim-bookworm

WORKDIR /app

COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

COPY . .

RUN adduser --disabled-password --gecos '' appuser
USER appuser

EXPOSE 5000

CMD ["gunicorn", "--bind", "0.0.0.0:5000", "app:app"]
```

### What Changed and Why

| Bloated | Optimized | Why |
|---------|-----------|-----|
| `FROM python:latest` | `FROM python:3.11-slim-bookworm` | Pinned version, minimal image |
| 3 separate `apt-get install` | Removed entirely | Packages not needed at runtime |
| 4 separate `pip install` | `COPY requirements.txt` + `pip install -r` | Single layer, dependency caching |
| No `--no-cache-dir` | `--no-cache-dir` | Saves 50-100MB |
| `COPY . /app` after RUN | `COPY requirements.txt` then `COPY . .` before RUN | Proper cache ordering |
| `useradd` without `USER` | `adduser` + `USER appuser` | Actually runs as non-root |
| `CMD python app.py` | `CMD ["gunicorn", ...]` | Production server, exec form |

---

## Part C: .dockerignore

```
.git
.gitignore
__pycache__
*.pyc
*.pyo
.env
.DS_Store
*.md
node_modules
tests
Dockerfile
```

### What Each Entry Excludes

| Pattern | Why Exclude It |
|---------|---------------|
| `.git` | Version control history -- large, not needed at runtime |
| `.gitignore` | Git configuration -- not needed at runtime |
| `__pycache__` | Python bytecode cache -- regenerated at runtime |
| `*.pyc`, `*.pyo` | Compiled Python files -- regenerated at runtime |
| `.env` | Environment variables with secrets -- must not be in the image |
| `.DS_Store` | macOS metadata -- not needed |
| `*.md` | Documentation -- not needed at runtime |
| `node_modules` | Frontend dependencies -- not needed in the Python image |
| `tests` | Test files -- not needed in production |
| `Dockerfile` | The Dockerfile itself -- not needed inside the image |

---

## Part D: Size Comparison

Expected results (actual numbers vary by system):

| Metric | Bloated | Optimized | Improvement |
|--------|---------|-----------|-------------|
| Image size | ~950MB | ~160MB | ~83% smaller |
| Layer count | ~12 | ~7 | ~40% fewer |
| Build time (cached) | ~30s | ~3s | ~90% faster |
| Packages installed | python + curl + vim + git | python only | Minimal attack surface |

The size difference comes from:
- Slim base image: ~750MB saved
- Removing apt packages: ~50MB saved
- Removing pip cache: ~50MB saved
- `.dockerignore`: varies, but prevents `.git` and `node_modules` from entering the image

---

## Common Mistakes to Avoid

1. **Installing "nice to have" packages.** Every package in the image is a potential vulnerability. If you do not need it at runtime, do not install it.

2. **Separate RUN commands for related operations.** Each RUN creates a layer. Combine related commands (apt-get update + install + clean) into one RUN.

3. **Forgetting that `adduser` without `USER` is useless.** The user is created but the container still runs as root.

4. **Using the dev server in production.** Flask's dev server is explicitly not for production. Use gunicorn, uvicorn, or another production WSGI/ASGI server.

5. **Not measuring.** Always check `docker images` to see the actual size. Small changes in the Dockerfile can have dramatic effects on image size.
