# Solution 02: Containerize a Python App

## Complete Dockerfile

```dockerfile
FROM python:3.11-slim-bookworm

WORKDIR /app

COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

COPY . .

RUN adduser --disabled-password --gecos '' appuser
USER appuser

EXPOSE 5000

CMD ["python", "app.py"]
```

## Complete .dockerignore

```
.git
.gitignore
__pycache__
*.pyc
*.pyo
.env
.DS_Store
*.md
```

## Step-by-Step Explanation

### Step 1: Base Image

```dockerfile
FROM python:3.11-slim-bookworm
```

**Why this image:**
- `python:3.11` -- specific version, not `latest`
- `-slim` -- stripped-down variant (~150MB vs ~900MB for full)
- `-bookworm` -- pins the Debian version for reproducibility

**Alternative considered:** `python:3.11-alpine` is even smaller (~50MB), but Alpine uses `musl` libc instead of `glibc`, which can cause issues with some Python packages that have C extensions (like numpy, pandas). The `-slim` variant is the safest small image for Python.

### Step 2: Working Directory

```dockerfile
WORKDIR /app
```

**Why /app:**
- Convention for application code inside containers
- Keeps your code separate from system files
- Creates the directory automatically if it does not exist

### Step 3: Dependencies (with caching)

```dockerfile
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt
```

**Why this order:**
- Copying `requirements.txt` alone (not the whole project) means this layer only rebuilds when dependencies change
- `--no-cache-dir` prevents pip from storing downloads in the image (saves ~10-50MB)
- If you copied everything first, any code change would trigger a full dependency reinstall

### Step 4: Application Code

```dockerfile
COPY . .
```

**Why after dependencies:**
- This layer changes frequently (every code edit)
- Because dependencies are in a previous cached layer, only this layer rebuilds on code changes
- Build time drops from minutes to seconds for code-only changes

### Step 5: Non-Root User

```dockerfile
RUN adduser --disabled-password --gecos '' appuser
USER appuser
```

**Why:**
- Containers run as root by default, which is a security risk
- If an attacker exploits a vulnerability in your app, they get root access to the container
- Running as non-root limits the blast radius of a compromise
- `--disabled-password` prevents password-based login
- `--gecos ''` skips the interactive "full name" prompt

### Step 6: Port Documentation

```dockerfile
EXPOSE 5000
```

**Why:**
- Documents that the app listens on port 5000
- Helps tools and humans understand the container's network requirements
- Does NOT publish the port -- you still need `-p 5000:5000` at runtime

### Step 7: Default Command

```dockerfile
CMD ["python", "app.py"]
```

**Why exec form:**
- Runs `python` directly as PID 1 (no shell wrapper)
- Signals (SIGTERM, SIGINT) reach the process directly, enabling graceful shutdown
- Shell form (`CMD python app.py`) would run through `/bin/sh -c`, which can cause signal handling issues

---

## Build and Run Commands

```bash
# Build the image
docker build -t my-flask-app:1.0 .

# Run the container
docker run -p 5000:5000 my-flask-app:1.0

# Test it
curl http://localhost:5000/
# Expected: {"message":"Hello from Docker!","status":"ok"}

curl http://localhost:5000/health
# Expected: {"status":"healthy"}
```

---

## Common Mistakes to Avoid

1. **Using `FROM python:latest`.** The `latest` tag can change at any time, breaking your builds unpredictably. Always pin to a specific version.

2. **Running `COPY . .` before `pip install`.** This is the most common Dockerfile performance mistake. Every code change forces a full dependency reinstall.

3. **Forgetting `--no-cache-dir`.** Without it, pip's download cache stays in the image, adding unnecessary size.

4. **Creating a user but not switching to it.** `RUN adduser ...` creates the user, but `USER appuser` is what actually makes the container run as that user. Both lines are needed.

5. **Using `CMD python app.py` (shell form).** This wraps the process in `/bin/sh -c`, which means SIGTERM from `docker stop` goes to the shell, not Python. The container takes 10 seconds to stop (the default timeout before SIGKILL).

6. **Not having a `.dockerignore`.** Without it, `COPY . .` copies `.git/`, `__pycache__/`, `.env`, and other files that bloat the image and can leak secrets.

---

## Verification Checklist

After building, verify your image:

```bash
# Check the image exists and its size
docker images my-flask-app

# Run and test endpoints
docker run -d --name test-flask -p 5000:5000 my-flask-app:1.0
curl http://localhost:5000/
curl http://localhost:5000/health

# Verify non-root user
docker exec test-flask whoami
# Expected: appuser

# Check the image layers
docker history my-flask-app:1.0

# Clean up
docker stop test-flask && docker rm test-flask
```
