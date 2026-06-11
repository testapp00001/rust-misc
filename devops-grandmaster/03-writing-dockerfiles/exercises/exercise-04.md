# Exercise 04: The Bloated Dockerfile Diet

**Type:** Challenge
**Time:** 45 minutes
**Difficulty:** Medium-Hard

## Objective

You are given a Dockerfile that *works* but violates nearly every best practice.
Your job is to identify every problem, explain why it is a problem, and rewrite
the Dockerfile to follow best practices. Then measure the difference.

## The Bloated Dockerfile

A junior developer wrote this Dockerfile for a Python Flask application.
It builds and runs correctly, but it is slow, insecure, and bloated.

```dockerfile
FROM python:latest

RUN apt-get update
RUN apt-get install -y curl
RUN apt-get install -y vim
RUN apt-get install -y git

RUN pip install flask
RUN pip install gunicorn
RUN pip install requests
RUN pip install pandas
RUN pip install numpy

COPY . /app

WORKDIR /app

RUN useradd -m appuser

EXPOSE 5000

CMD python app.py
```

## The Application Context

The application is a simple REST API. Here is what it actually needs:

**app.py:**
```python
from flask import Flask, jsonify

app = Flask(__name__)

@app.route('/')
def hello():
    return jsonify({'message': 'Hello!', 'status': 'ok'})

@app.route('/health')
def health():
    return jsonify({'status': 'healthy'})

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5000)
```

**requirements.txt:**
```
flask==3.0.0
gunicorn==21.2.0
```

The application does NOT use `requests`, `pandas`, or `numpy`.
It does NOT need `curl`, `vim`, or `git` at runtime.

## Tasks

### Part A: Identify the Problems

List every problem with the bloated Dockerfile. For each problem:
1. State what is wrong
2. Explain *why* it is a problem (not just "it is bad practice")
3. Categorize it as: **Security**, **Performance**, **Correctness**, or **Best Practice**

You should find at least **10 distinct problems**.

<details>
<summary>Hint 1: Base Image</summary>

Look at the `FROM` line. Two issues:
- What tag is being used?
- What variant is being used (slim vs full)?

</details>

<details>
<summary>Hint 2: RUN Instructions</summary>

Count the `RUN` instructions. Each one creates a layer. Consider:
- Could any be combined?
- Are the `apt-get` commands missing cleanup?
- Are the `pip install` commands missing flags?

</details>

<details>
<summary>Hint 3: COPY and WORKDIR Order</summary>

Look at where `COPY` and `WORKDIR` appear. What happens to the build cache?
Also look at what is being copied -- is everything needed?

</details>

<details>
<summary>Hint 4: USER Instruction</summary>

The user is created but never actually used. What does that mean for
container security?

</details>

### Part B: Rewrite the Dockerfile

Write a new Dockerfile that fixes every problem you identified. Your new
Dockerfile must:

1. Use a specific, slim base image tag
2. Combine RUN commands to minimize layers
3. Clean up apt caches after installing packages
4. Copy only what the application needs (use `requirements.txt` for deps)
5. Use `--no-cache-dir` for pip
6. Actually switch to the non-root user
7. Use the exec form for CMD
8. Use `gunicorn` instead of the Flask dev server for CMD

<details>
<summary>Hint: Structure</summary>

Your optimized Dockerfile should follow this structure:

```dockerfile
FROM python:3.11-slim-bookworm

WORKDIR /app

# System deps (if any, combined and cleaned)
# Dependency install (requirements.txt first)
# Copy code
# Create and switch to non-root user
# Expose port
# CMD with gunicorn
```

</details>

### Part C: Add a .dockerignore

Write a `.dockerignore` file that prevents unnecessary files from entering
the build context. The project has:
- `.git/` directory
- `__pycache__/` directories
- `.env` file with secrets
- `*.pyc` files
- `node_modules/` (from a frontend build)
- `*.md` documentation files
- `tests/` directory (not needed in production image)
- `Dockerfile` itself (no need to copy it into the image)
- `.gitignore`

<details>
<summary>Hint</summary>

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

</details>

### Part D: Measure the Difference

If you have Docker available, build both the bloated and optimized versions
and compare:

```bash
# Build the bloated version
docker build -f Dockerfile.bloated -t app-bloated .

# Build the optimized version
docker build -t app-optimized .

# Compare sizes
docker images | grep app-

# Compare layer count
docker history app-bloated
docker history app-optimized
```

Record:
- The size of each image
- The number of layers in each
- How much smaller the optimized image is (percentage)

## Success Criteria

- [ ] You identified at least 10 distinct problems in the bloated Dockerfile
- [ ] Each problem is categorized (Security, Performance, Correctness, Best Practice)
- [ ] Your rewritten Dockerfile addresses every identified problem
- [ ] Your rewritten Dockerfile builds and runs correctly
- [ ] You have a `.dockerignore` that excludes unnecessary files
- [ ] You can explain the size difference between the two images

## What You Should Understand After This Exercise

A "working" Dockerfile is not the same as a good Dockerfile. Image size,
build speed, security, and maintainability all depend on following best
 practices. Every unnecessary layer, package, and file costs you in build
time, pull time, storage, and attack surface. The bloated Dockerfile is
not hypothetical -- it is representative of what most beginners write.
