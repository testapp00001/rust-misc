# Exercise 02: Containerize a Python App

**Type:** Guided
**Time:** 30 minutes
**Difficulty:** Easy-Medium

## Objective

Write a complete Dockerfile for a Python Flask application from scratch,
following best practices step by step. By the end, you will have a working
containerized application you can build and run.

## The Application

You are given a simple Python Flask application. Create the following files
in a new directory called `my-flask-app/`.

**app.py:**
```python
from flask import Flask, jsonify

app = Flask(__name__)

@app.route('/')
def hello():
    return jsonify({'message': 'Hello from Docker!', 'status': 'ok'})

@app.route('/health')
def health():
    return jsonify({'status': 'healthy'})

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5000)
```

**requirements.txt:**
```
flask==3.0.0
```

## Tasks

### Step 1: Choose a Base Image

Write the first line of your Dockerfile. You need a base image that:
- Has Python 3.11 installed
- Is as small as possible (look for `-slim` variants)
- Uses a specific version tag, not `latest`

Create a file called `Dockerfile` in your `my-flask-app/` directory.

<details>
<summary>Hint</summary>

Use `python:3.11-slim-bookworm` as your base image. The `-slim` variant
strips out unnecessary packages, and `bookworm` pins the Debian version.

```dockerfile
FROM python:3.11-slim-bookworm
```

</details>

### Step 2: Set the Working Directory

Add a `WORKDIR` instruction. Your application code will live in `/app`
inside the container.

<details>
<summary>Hint</summary>

```dockerfile
WORKDIR /app
```

This creates the directory if it does not exist and sets it as the default
for all subsequent instructions.

</details>

### Step 3: Install Dependencies (with Caching in Mind)

Copy your `requirements.txt` into the container and install the packages.
Think about layer caching -- you want dependencies to be cached separately
from your application code.

Use `pip install --no-cache-dir` to avoid storing the pip download cache
in the image.

<details>
<summary>Hint</summary>

```dockerfile
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt
```

By copying `requirements.txt` before the rest of your code, Docker caches
the dependency installation layer. It only re-runs when `requirements.txt`
changes, not every time you edit `app.py`.

</details>

### Step 4: Copy the Application Code

Now copy the rest of your application files into the container.

<details>
<summary>Hint</summary>

```dockerfile
COPY . .
```

This copies everything from the build context (your `my-flask-app/` directory)
into `/app` inside the container.

</details>

### Step 5: Create a Non-Root User

Running containers as root is a security risk. Add instructions to:
1. Create a user called `appuser` with no password
2. Switch to that user for all subsequent instructions

<details>
<summary>Hint</summary>

```dockerfile
RUN adduser --disabled-password --gecos '' appuser
USER appuser
```

The `--disabled-password` flag prevents login, and `--gecos ''` avoids
interactive prompts about the user's full name.

</details>

### Step 6: Document the Port

Your Flask app listens on port 5000. Add the `EXPOSE` instruction to
document this.

<details>
<summary>Hint</summary>

```dockerfile
EXPOSE 5000
```

Remember: EXPOSE is documentation. It does not publish the port. You still
need `-p 5000:5000` when running the container.

</details>

### Step 7: Define the Default Command

Write the `CMD` instruction that starts your Flask application when the
container runs. Use the exec form (JSON array).

<details>
<summary>Hint</summary>

```dockerfile
CMD ["python", "app.py"]
```

The exec form is preferred over the shell form (`CMD python app.py`) because
it runs the process directly without a shell wrapper, which allows proper
signal handling.

</details>

### Step 8: Build and Run

Build your Docker image and run it:

```bash
cd my-flask-app/
docker build -t my-flask-app:1.0 .
docker run -p 5000:5000 my-flask-app:1.0
```

In another terminal, test it:

```bash
curl http://localhost:5000/
curl http://localhost:5000/health
```

Verify you see the expected JSON responses.

### Step 9: Add a .dockerignore

Create a `.dockerignore` file in `my-flask-app/` to exclude files that
should not be copied into the image.

Think about what files exist in a typical Python project that are not
needed at runtime.

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
```

This prevents version control files, cached bytecode, environment secrets,
and documentation from bloating your image.

</details>

### Step 10: Rebuild and Verify

Rebuild the image with the `.dockerignore` in place:

```bash
docker build -t my-flask-app:1.1 .
docker run -p 5001:5000 my-flask-app:1.1
```

Compare the image sizes:

```bash
docker images my-flask-app
```

## Success Criteria

- [ ] Your Dockerfile has all 7 instructions in the correct order
- [ ] The image builds without errors
- [ ] The container starts and responds to HTTP requests on `/` and `/health`
- [ ] You are running as a non-root user (verify with `docker exec <id> whoami`)
- [ ] You have a `.dockerignore` that excludes at least 5 patterns
- [ ] You understand why `requirements.txt` is copied before the rest of the code

## What You Should Understand After This Exercise

Writing a Dockerfile follows a pattern: base image, working directory,
dependencies, code, security, documentation, command. Each step builds
on the previous one. The order is not arbitrary -- it is driven by
caching efficiency and security best practices.
