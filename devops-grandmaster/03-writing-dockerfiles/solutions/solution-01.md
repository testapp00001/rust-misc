# Solution 01: Anatomy of a Dockerfile

## Part A: Explain Each Instruction

### `FROM python:3.11-slim-bookworm`
Sets the base image to Python 3.11 on a minimal Debian Bookworm distribution. The `-slim` variant excludes unnecessary packages (compilers, man pages, etc.) to keep the image small. The `bookworm` tag pins the Debian version so builds are reproducible.

### `WORKDIR /app`
Sets the working directory inside the container to `/app`. All subsequent instructions (`COPY`, `RUN`, `CMD`) execute relative to this directory. Creates the directory if it does not exist. This is preferred over using `RUN mkdir /app` because it is declarative and automatically handles the directory creation.

### `COPY requirements.txt .`
Copies only `requirements.txt` from the build context into `/app/` inside the container. By copying this file alone (before the rest of the code), Docker can cache the dependency installation layer. If you copied everything first, any code change would invalidate the pip install cache.

### `RUN pip install --no-cache-dir -r requirements.txt`
Installs Python dependencies from `requirements.txt`. The `--no-cache-dir` flag tells pip not to store downloaded packages in a cache directory, saving space in the image layer. This runs during build time, not at runtime.

### `COPY . .`
Copies the entire build context (except files in `.dockerignore`) into `/app/` inside the container. This is placed after the dependency install so that code changes do not trigger a re-download of all packages.

### `RUN adduser --disabled-password --gecos '' appuser`
Creates a Linux user named `appuser` with no password and no personal information fields. This is a security best practice -- containers should not run as root. The `--disabled-password` flag prevents login, and `--gecos ''` avoids interactive prompts.

### `USER appuser`
Switches the active user for all subsequent instructions and for the container at runtime. After this line, the process started by `CMD` runs as `appuser` instead of `root`. This limits the damage if the application is compromised.

### `EXPOSE 5000`
Documents that the application listens on port 5000. This is purely informational -- it does not actually publish the port. You still need `-p 5000:5000` at runtime. Think of it as a label for humans and tools.

### `HEALTHCHECK --interval=30s --timeout=3s --retries=3 CMD curl -f http://localhost:5000/health || exit 1`
Tells Docker how to check if the container is healthy. Every 30 seconds, it tries to fetch the `/health` endpoint. If the request fails or takes more than 3 seconds, it marks the check as failed. After 3 consecutive failures, the container is marked unhealthy. Container orchestrators (Docker Compose, Kubernetes) use this to restart unhealthy containers.

### `CMD ["python", "app.py"]`
Defines the default command to run when the container starts. The exec form (JSON array) runs `python` directly as PID 1 without a shell wrapper. This is important because it allows the process to receive signals (like SIGTERM) properly, which enables graceful shutdown.

---

## Part B: Why This Order?

### 1. Why does `COPY requirements.txt .` come before `COPY . .`?

Docker builds images in layers and caches each layer. When a layer changes, all subsequent layers are rebuilt. By copying `requirements.txt` first and installing dependencies, that layer is cached. If you only change `app.py`, Docker reuses the cached dependency layer and only rebuilds the `COPY . .` layer. If you copied everything first, *any* file change would invalidate the pip install cache, forcing a full dependency reinstall on every build.

### 2. Why does `RUN adduser` come after `COPY` and `pip install`?

The `adduser` command does not depend on the application code or packages. Placing it after the code copy keeps it out of the frequently-changing layers. More importantly, since `USER appuser` comes after `adduser`, placing them together at the end means all the heavy work (copying, installing) happens as root where it has the necessary permissions.

### 3. Why does `CMD` come at the very end?

`CMD` defines the runtime behavior. It is the final instruction because everything the application needs (code, dependencies, user, port) must be set up first. Also, if there were multiple `CMD` instructions, only the last one takes effect -- placing it at the end makes the intent clear.

### 4. Why does `USER appuser` come before `EXPOSE` and `CMD`?

`USER` must come after all instructions that need root privileges (like `RUN pip install` and `RUN adduser`). Once the user switches, subsequent `RUN` commands would execute as the non-root user. `EXPOSE` and `CMD` do not require root, so they can safely come after `USER`.

---

## Part C: What If We Change the Order?

### Scenario 1: `COPY . .` before `COPY requirements.txt .`

```dockerfile
COPY . .
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt
```

**What breaks:** The build cache for dependencies is destroyed. Every time you change *any* file in your project (even a comment in `app.py`), Docker invalidates the `COPY . .` layer, which forces all subsequent layers (including `pip install`) to rebuild. Your build time doubles or triples because you re-download and reinstall all packages on every code change.

### Scenario 2: `USER appuser` before `RUN pip install`

```dockerfile
COPY requirements.txt .
RUN adduser --disabled-password --gecos '' appuser
USER appuser
RUN pip install --no-cache-dir -r requirements.txt
```

**What breaks:** The `pip install` command runs as `appuser`, which likely does not have write permissions to the system Python package directory (`/usr/local/lib/python3.11/site-packages/`). The build fails with a `PermissionError`. Even if you worked around this with `--user`, it would install packages in a non-standard location and complicate the Python path.

### Scenario 3: `EXPOSE 5000` right after `FROM`

```dockerfile
FROM python:3.11-slim-bookworm
EXPOSE 5000
WORKDIR /app
```

**What breaks:** Functionally, nothing breaks. `EXPOSE` is purely documentary and does not depend on order. However, it violates the convention of grouping related instructions. Placing it near `CMD` makes the Dockerfile more readable -- you can see at a glance what port the application uses and how it starts. This is a readability issue, not a correctness issue.

---

## Part D: Identifying Instructions

| Instruction | What it does | Required? |
|-------------|-------------|-----------|
| FROM | Sets the base image for the build. Every Dockerfile must start with FROM (or ARG before the first FROM). | **Yes** |
| WORKDIR | Sets the working directory for subsequent instructions. Creates the directory if it does not exist. | No |
| COPY | Copies files from the build context into the image filesystem. | No (but you need some way to get code in) |
| RUN | Executes a command during the build and commits the result as a new layer. | No |
| CMD | Sets the default command to run when the container starts. Only the last CMD takes effect. | No (inherited from base image) |
| ENTRYPOINT | Configures the container to run as an executable. CMD becomes arguments to ENTRYPOINT. | No (inherited from base image) |
| ENV | Sets environment variables that persist at runtime. | No |
| EXPOSE | Documents which ports the container listens on. Does not publish ports. | No |
| ARG | Defines build-time variables. Only available during the build, not at runtime. | No |
| USER | Sets the user for subsequent RUN, CMD, and ENTRYPOINT instructions. | No |
| HEALTHCHECK | Tells Docker how to test if the container is still working. | No |

---

## Common Mistakes to Avoid

1. **Thinking EXPOSE publishes ports.** It does not. You must use `-p` at runtime or define port mappings in docker-compose.yml.

2. **Placing COPY . . before dependency install.** This is the single most common Dockerfile performance mistake. It forces a full dependency reinstall on every code change.

3. **Forgetting to switch USER.** Creating a user with `adduser` but never calling `USER` means the container still runs as root. The `USER` instruction is what actually switches.

4. **Using shell form for CMD.** `CMD python app.py` runs through `/bin/sh -c`, which means signals go to the shell, not your process. Use exec form: `CMD ["python", "app.py"]`.

5. **Not using --no-cache-dir with pip.** The pip cache can add 50-100MB to your image for no runtime benefit.
