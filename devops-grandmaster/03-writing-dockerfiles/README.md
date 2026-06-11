# Module 03: Writing Dockerfiles

## The Problem: How Do You Package YOUR Code?

You can run other people's containers. Now you need to containerize **your own application**.

A Dockerfile is a recipe that tells Docker how to build your image.

## The Simplest Dockerfile

Create a file called `Dockerfile` (no extension):

```dockerfile
# Start from an existing image
FROM python:3.11-slim

# Set the working directory inside the container
WORKDIR /app

# Copy your code into the container
COPY app.py .

# Install dependencies
RUN pip install flask

# What command to run when the container starts
CMD ["python", "app.py"]
```

## Dockerfile Instructions Explained

### FROM — The Base Image
```dockerfile
FROM python:3.11-slim
FROM node:18-alpine
FROM golang:1.21
FROM nginx:latest
FROM ubuntu:22.04
FROM scratch  # Empty image (for compiled languages)
```

**Always use specific tags**, not `latest`:
```dockerfile
# BAD — could change anytime
FROM python:latest

# GOOD — predictable
FROM python:3.11.4-slim-bookworm
```

### WORKDIR — Set Working Directory
```dockerfile
WORKDIR /app
# All subsequent commands run from /app
# Creates the directory if it doesn't exist
```

### COPY — Copy Files From Host
```dockerfile
# Copy single file
COPY app.py .

# Copy entire directory
COPY src/ ./src/

# Copy with rename
COPY config.yml ./config/app.yml

# Copy everything in current directory
COPY . .
```

### RUN — Execute Commands During Build
```dockerfile
# Install packages
RUN apt-get update && apt-get install -y curl

# Install Python packages
RUN pip install flask gunicorn

# Install Node packages
RUN npm install

# Chain commands to reduce layers
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    curl \
    ca-certificates && \
    rm -rf /var/lib/apt/lists/*
```

### CMD — Default Command
```dockerfile
# Exec form (preferred) — no shell wrapping
CMD ["python", "app.py"]

# Shell form — runs through /bin/sh -c
CMD python app.py

# Only ONE CMD per Dockerfile (last one wins)
```

### ENTRYPOINT — The Main Executable
```dockerfile
ENTRYPOINT ["python"]
CMD ["app.py"]

# Result: python app.py
# Can override CMD: docker run myimage test.py → python test.py
```

### ENV — Environment Variables
```dockerfile
ENV PYTHONUNBUFFERED=1
ENV APP_ENV=production
ENV PORT=8080
```

### EXPOSE — Document Ports
```dockerfile
EXPOSE 8080
# Just documentation! Doesn't actually publish the port.
# Use -p flag at runtime: docker run -p 8080:8080 myimage
```

### ARG — Build-Time Variables
```dockerfile
ARG PYTHON_VERSION=3.11
FROM python:${PYTHON_VERSION}-slim

# Use at build time:
# docker build --build-arg PYTHON_VERSION=3.12 .
```

### USER — Run as Non-Root
```dockerfile
# Create a non-root user
RUN adduser --disabled-password --gecos '' appuser
USER appuser
```

### HEALTHCHECK — Container Health
```dockerfile
HEALTHCHECK --interval=30s --timeout=3s --retries=3 \
  CMD curl -f http://localhost:8080/health || exit 1
```

## Complete Examples

### Python Flask App
```dockerfile
FROM python:3.11-slim-bookworm

WORKDIR /app

# Install dependencies first (better caching)
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Copy application code
COPY . .

# Create non-root user
RUN adduser --disabled-password --gecos '' appuser
USER appuser

EXPOSE 5000

CMD ["gunicorn", "--bind", "0.0.0.0:5000", "app:app"]
```

### Node.js Express App
```dockerfile
FROM node:18-alpine

WORKDIR /app

# Install dependencies first
COPY package.json package-lock.json ./
RUN npm ci --only=production

# Copy application code
COPY . .

# Create non-root user
RUN addgroup -g 1001 appgroup && \
    adduser -u 1001 -G appgroup -s /bin/sh -D appuser
USER appuser

EXPOSE 3000

CMD ["node", "server.js"]
```

### Go Application
```dockerfile
# Build stage
FROM golang:1.21-alpine AS builder

WORKDIR /app
COPY go.mod go.sum ./
RUN go mod download

COPY . .
RUN CGO_ENABLED=0 GOOS=linux go build -o server .

# Run stage (minimal image)
FROM alpine:3.18

RUN apk --no-cache add ca-certificates
WORKDIR /app
COPY --from=builder /app/server .

EXPOSE 8080
CMD ["./server"]
```

### Static Website (Nginx)
```dockerfile
FROM nginx:alpine

# Remove default config
RUN rm /etc/nginx/conf.d/default.conf

# Add custom config
COPY nginx.conf /etc/nginx/conf.d/

# Copy website files
COPY dist/ /usr/share/nginx/html/

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
```

## Building the Image

```bash
# Build with default Dockerfile
docker build -t myapp:1.0 .

# Build with specific file
docker build -f Dockerfile.prod -t myapp:1.0 .

# Build with build args
docker build --build-arg VERSION=1.0 -t myapp:1.0 .

# Build with no cache
docker build --no-cache -t myapp:1.0 .
```

## Best Practices

### 1. Use .dockerignore
Create `.dockerignore` to exclude unnecessary files:
```
.git
.gitignore
node_modules
*.md
.env
.DS_Store
__pycache__
*.pyc
```

### 2. Order Matters for Caching
```dockerfile
# BAD — changing code re-installs dependencies
COPY . .
RUN pip install -r requirements.txt

# GOOD — dependencies cached separately
COPY requirements.txt .
RUN pip install -r requirements.txt
COPY . .
```

### 3. Minimize Layers
```dockerfile
# BAD — 3 layers
RUN apt-get update
RUN apt-get install -y curl
RUN rm -rf /var/lib/apt/lists/*

# GOOD — 1 layer
RUN apt-get update && \
    apt-get install -y curl && \
    rm -rf /var/lib/apt/lists/*
```

### 4. Use Specific Tags
```dockerfile
# BAD
FROM python:latest

# GOOD
FROM python:3.11.4-slim-bookworm
```

### 5. Run as Non-Root
```dockerfile
# Always create and switch to non-root user
RUN adduser --disabled-password --gecos '' appuser
USER appuser
```

## Hands-On Exercise

### Exercise 1: Containerize a Python App
Create these files:

**app.py:**
```python
from flask import Flask
app = Flask(__name__)

@app.route('/')
def hello():
    return 'Hello from Docker!'

@app.route('/health')
def health():
    return {'status': 'ok'}

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5000)
```

**requirements.txt:**
```
flask==3.0.0
```

**Dockerfile:**
```dockerfile
FROM python:3.11-slim
WORKDIR /app
COPY requirements.txt .
RUN pip install -r requirements.txt
COPY . .
EXPOSE 5000
CMD ["python", "app.py"]
```

**Build and run:**
```bash
docker build -t my-flask-app:1.0 .
docker run -p 5000:5000 my-flask-app:1.0
# Visit http://localhost:5000
```

### Exercise 2: Add Health Check
Modify the Dockerfile to add a health check:
```dockerfile
HEALTHCHECK --interval=30s --timeout=3s --retries=3 \
  CMD curl -f http://localhost:5000/health || exit 1
```

Build and check:
```bash
docker build -t my-flask-app:1.0 .
docker run -d --name myapp -p 5000:5000 my-flask-app:1.0
docker inspect --format='{{.State.Health.Status}}' myapp
```

## Limitation: Your Image is HUGE

A basic Python app image can be 900MB+. That's slow to build, slow to push, slow to pull.

**Next problem:** How do you make images smaller?

→ **Next module:** [04-multi-stage-builds](../04-multi-stage-builds/) — Shrink images from 1GB to 10MB

## Checklist

- [ ] I can write a Dockerfile from scratch
- [ ] I understand FROM, WORKDIR, COPY, RUN, CMD, ENTRYPOINT
- [ ] I know the difference between ARG and ENV
- [ ] I use .dockerignore to exclude unnecessary files
- [ ] I order instructions for optimal caching
- [ ] I run containers as non-root users
