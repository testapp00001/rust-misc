# Exercise 02: Fix the Dockerfile That Rebuilds Everything

**Type:** Guided
**Time:** 30 minutes
**Difficulty:** Easy-Medium

## Objective

Take a poorly-written Dockerfile that rebuilds all dependencies on every code
change and fix it so that dependency installation is cached independently from
source code changes. This is the single most impactful caching optimization
you will learn.

## Starting Point

You are given a Python Flask application with the following structure:

```
analytics-service/
├── Dockerfile
├── requirements.txt
├── setup.py
├── .env
├── README.md
└── src/
    ├── __init__.py
    ├── app.py
    ├── models.py
    ├── routes.py
    └── utils.py
```

The current Dockerfile:

```dockerfile
FROM python:3.11

WORKDIR /app

COPY . .

RUN pip install -r requirements.txt
RUN pip install -e .

CMD ["python", "src/app.py"]
```

The `requirements.txt` contains:

```
flask==3.0.0
sqlalchemy==2.0.23
psycopg2-binary==2.9.9
gunicorn==21.2.0
celery==5.3.6
redis==5.0.1
numpy==1.26.2
pandas==2.1.4
scikit-learn==1.3.2
```

## The Problem

Every time you change a single line in `src/routes.py`, the build takes
**3 minutes** because it reinstalls all Python packages from scratch.

```bash
$ time docker build -t analytics-service .
real    3m12.456s

# Fix a typo in src/routes.py
$ time docker build -t analytics-service .
real    3m08.789s   # Still 3 minutes!
```

## Tasks

### Part A: Identify the Root Cause

Explain exactly why the current Dockerfile reinstalls all packages when only
`src/routes.py` changes. Reference the specific line in the Dockerfile that
causes the problem.

<details>
<summary>Hint</summary>

Look at the order of `COPY` and `RUN` instructions. Which instruction
invalidates first when a source file changes?

</details>

### Part B: Reorder the Instructions

Rewrite the Dockerfile so that dependency installation is cached when only
source code changes. Your Dockerfile must:

1. Copy dependency files (`requirements.txt`) before copying source code
2. Install dependencies before copying source code
3. Still produce a working application

<details>
<summary>Hint</summary>

The key insight: `requirements.txt` changes rarely. Source code changes
constantly. Copy the rarely-changing file first, install dependencies,
then copy the constantly-changing code.

</details>

### Part C: Fix the Base Image

The current Dockerfile uses `python:3.11` (900MB+). Change it to a smaller
variant. Which variant should you choose and why?

<details>
<summary>Hint</summary>

Compare `python:3.11`, `python:3.11-slim`, and `python:3.11-alpine`.
Consider size, compatibility, and build dependencies.

</details>

### Part D: Add a .dockerignore

The current build has no `.dockerignore`. Create one that excludes files that
should never be in the build context. List at least 8 entries.

<details>
<summary>Hint</summary>

Think about what files exist in a typical Python project that are not needed
inside the container: version control, environment files, caches, IDE config,
test artifacts, documentation.

</details>

### Part E: Add BuildKit Cache Mounts

Add a pip cache mount to the dependency installation step. This ensures that
even when `requirements.txt` changes, pip can reuse already-downloaded packages
from previous builds.

<details>
<summary>Hint</summary>

Use the `# syntax=docker/dockerfile:1` directive and the
`RUN --mount=type=cache,target=...` syntax. The pip cache directory
on Linux is `/root/.cache/pip`.

</details>

### Part F: Verify Your Fix

After making all changes, your rebuild time for a source code change should
drop from 3 minutes to under 15 seconds. Describe what you would run to
verify this.

<details>
<summary>Hint</summary>

Build once (cold cache), then change a source file and build again (warm
cache). Measure both with `time`.

</details>

## Success Criteria

- [ ] Your Dockerfile copies `requirements.txt` before copying source code
- [ ] Your Dockerfile uses `python:3.11-slim` (not `python:3.11`)
- [ ] You have a `.dockerignore` with at least 8 entries
- [ ] Your Dockerfile uses a BuildKit cache mount for pip
- [ ] You can explain why the new ordering makes rebuilds faster
- [ ] You can predict which layers are cached when only source code changes

## What You Should Understand After This Exercise

The single most impactful Dockerfile caching optimization is separating
dependency installation from code copying. By ordering instructions from
"changes rarely" to "changes often," you ensure that the expensive
dependency installation step is cached on every code change.
