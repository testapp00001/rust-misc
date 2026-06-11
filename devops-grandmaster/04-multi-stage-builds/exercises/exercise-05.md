# Exercise 05: Dev, Test, and Prod -- A Three-Stage Pipeline

**Type:** Integration
**Time:** 60 minutes
**Difficulty:** Hard

## Objective

Build a single Dockerfile that produces three different images for three
different environments: development, testing, and production. This exercise
combines multi-stage builds with `docker build --target` to create a
complete image pipeline from one source of truth.

## Scenario

Your team has a Python web application. Currently, they maintain three
separate Dockerfiles:

- `Dockerfile.dev` -- includes debugging tools, hot-reload, test frameworks
- `Dockerfile.test` -- includes test frameworks but not dev tools
- `Dockerfile.prod` -- minimal, production-only

This is a maintenance nightmare. Every dependency change requires updating
three files. The dev image accidentally ships to production. The prod image
is missing a dependency that was added to dev but not prod.

Your task: consolidate into a single multi-stage Dockerfile with three
named final stages.

## Application Code

Create the project:

```bash
mkdir -p /tmp/exercise-05
```

Create `/tmp/exercise-05/requirements.txt`:

```
flask==3.0.3
gunicorn==22.0.0
```

Create `/tmp/exercise-05/requirements-dev.txt`:

```
-r requirements.txt
pytest==8.2.0
pytest-cov==5.0.0
debugpy==1.8.1
flake8==7.0.0
black==24.4.2
```

Create `/tmp/exercise-05/app.py`:

```python
import os
from flask import Flask

app = Flask(__name__)

@app.route("/")
def hello():
    return f"Hello from {os.getenv('ENVIRONMENT', 'unknown')}!\n"

@app.route("/health")
def health():
    return {"status": "ok", "env": os.getenv("ENVIRONMENT", "unknown")}

if __name__ == "__main__":
    app.run(host="0.0.0.0", port=5000, debug=True)
```

Create `/tmp/exercise-05/test_app.py`:

```python
from app import app

def test_hello():
    client = app.test_client()
    response = client.get("/")
    assert response.status_code == 200
    assert b"Hello" in response.data

def test_health():
    client = app.test_client()
    response = client.get("/health")
    assert response.status_code == 200
    data = response.get_json()
    assert data["status"] == "ok"
```

## Tasks

### Part A: Design the Stage Architecture

Before writing any Dockerfile instructions, draw the stage dependency graph.
You need:

1. A **builder** stage that installs all packages (including dev packages)
2. A **dev** target that includes debugging tools and test frameworks
3. A **test** target that includes test frameworks but not debugging tools
4. A **prod** target that is minimal

Which stages depend on which? Can any stages be built in parallel?

<details>
<summary>Hint</summary>

```
builder (installs everything)
  |
  +---> dev (copies everything from builder, adds debug config)
  |
  +---> test (copies everything from builder, adds test config)
  |
  +---> prod (copies only production packages from builder)
```

`dev`, `test`, and `prod` can all reference the builder stage independently.
They do not depend on each other.

</details>

### Part B: Write the Builder Stage

Write the builder stage that:
- Uses `python:3.12` as the base (you need build tools)
- Installs ALL packages from `requirements-dev.txt`
- Uses `--prefix=/install` so packages can be copied cleanly

<details>
<summary>Hint</summary>

```dockerfile
FROM python:3.12 AS builder

WORKDIR /app
COPY requirements-dev.txt .
RUN pip install --no-cache-dir --prefix=/install -r requirements-dev.txt
```

This installs everything: Flask, gunicorn, pytest, debugpy, flake8, black.
The `--prefix=/install` puts them in a separate directory for easy copying.

</details>

### Part C: Write the Production Stage

Write the `prod` stage that:
- Uses `python:3.12-slim` as the base
- Copies ONLY production packages (not test or dev tools)
- Runs as a non-root user
- Uses gunicorn as the server
- Sets `ENVIRONMENT=production`

How do you copy only production packages when the builder installed
everything together?

<details>
<summary>Hint</summary>

You have two options:
1. Install production and dev packages separately in the builder
2. Use a second builder stage that installs only production packages

Option 2 is cleaner:

```dockerfile
FROM python:3.12 AS builder-prod
WORKDIR /app
COPY requirements.txt .
RUN pip install --no-cache-dir --prefix=/install -r requirements.txt
```

Then the prod stage copies from `builder-prod`, not `builder`.

</details>

### Part D: Write the Development Stage

Write the `dev` stage that:
- Uses `python:3.12-slim` as the base
- Copies ALL packages from the builder (including dev tools)
- Copies the application source code
- Includes debugpy for remote debugging (port 5678)
- Sets `ENVIRONMENT=development`
- Runs Flask's built-in development server with hot-reload

<details>
<summary>Hint</summary>

```dockerfile
FROM python:3.12-slim AS dev

ENV ENVIRONMENT=development
WORKDIR /app
COPY --from=builder /install /usr/local
COPY . .

EXPOSE 5000
EXPOSE 5678
CMD ["python", "-m", "debugpy", "--listen", "0.0.0.0:5678", \
     "-m", "flask", "run", "--host=0.0.0.0", "--debug"]
```

</details>

### Part E: Write the Test Stage

Write the `test` stage that:
- Uses `python:3.12-slim` as the base
- Copies ALL packages from the builder
- Copies the application source code and test files
- Sets `ENVIRONMENT=test`
- Runs pytest by default

<details>
<summary>Hint</summary>

```dockerfile
FROM python:3.12-slim AS test

ENV ENVIRONMENT=test
WORKDIR /app
COPY --from=builder /install /usr/local
COPY . .

CMD ["python", "-m", "pytest", "--cov=.", "-v"]
```

</details>

### Part F: Build and Compare All Targets

Build each target and compare sizes:

```bash
cd /tmp/exercise-05

# Build all three targets
docker build --target prod -t exercise5-prod .
docker build --target test -t exercise5-test .
docker build --target dev -t exercise5-dev .

# Compare sizes
docker images | grep exercise5
```

Fill in this table:

| Target | Image Size | Contents |
|---|---|---|
| dev | ??? MB | Flask + dev tools + test tools + debugpy |
| test | ??? MB | Flask + test tools |
| prod | ??? MB | Flask + gunicorn only |

### Part G: Verify Each Target

```bash
# Production
docker run -p 5000:5000 -e ENVIRONMENT=production exercise5-prod &
curl http://localhost:5000
curl http://localhost:5000/health

# Tests
docker run exercise5-test

# Development (with debug port)
docker run -p 5000:5000 -p 5678:5678 exercise5-dev &
curl http://localhost:5000
```

## Success Criteria

- [ ] Single Dockerfile with four stages: builder, prod, test, dev
- [ ] `--target prod` produces the smallest image (under 200MB)
- [ ] `--target test` includes pytest and can run the test suite
- [ ] `--target dev` includes debugpy and development tools
- [ ] All three targets produce working, runnable images
- [ ] Production image runs as non-root user
- [ ] You can explain how `--target` selects which final stage to build
- [ ] You can explain the dependency graph between stages

## What You Should Understand After This Exercise

A single Dockerfile can serve multiple purposes by defining multiple
named final stages. The `docker build --target` flag selects which
stage becomes the final image. This eliminates the need for multiple
Dockerfiles and ensures consistency across environments. The builder
stage is shared, so dependency installation happens once. Each target
stage copies only what it needs from the shared builder.
