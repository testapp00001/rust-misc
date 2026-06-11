# Module 04: Multi-Stage Builds

## The Problem

In Module 03, you learned to write Dockerfiles. They work. Your app runs inside a container. Life is good.

Then you check the image size.

```
$ docker images
REPOSITORY   TAG       SIZE
my-python    latest    924MB
my-node      latest    1.1GB
my-java      latest    1.4GB
```

A gigabyte. For a "Hello World" application that prints a single line of text.

You push this image to a registry. It takes two minutes. Your CI pipeline spends five minutes building and pushing. Your Kubernetes cluster pulls it on every deploy. Your cloud bill arrives and you owe money for storing and transferring gigabytes of data that nobody needs.

What is actually inside that 924MB Python image?

```
Python runtime:          ~150MB
pip and setuptools:       ~30MB
C compiler (gcc):         ~80MB
Make:                     ~10MB
Linux headers:            ~50MB
Your source code:          ~1MB  <-- the only thing you actually need
Compiled .pyc files:       ~1MB
---------------------------------
Build tools left behind: ~300MB
OS packages you never use: ~400MB
Total waste:             ~900MB out of 924MB
```

98% of your image is garbage. Build tools, compilers, package managers, header files, and an operating system full of utilities you will never run inside a production container.

This is the single-stage build problem. One `FROM` instruction. Everything gets built and shipped in the same image. Build tools and runtime artifacts live together forever.

---

## The Naive Way

The obvious fix: delete the build tools after using them.

```dockerfile
# Naive cleanup approach
FROM python:3.12

WORKDIR /app
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

COPY . .

# "Clean up" build dependencies
RUN apt-get remove -y gcc make && \
    apt-get autoremove -y && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

CMD ["python", "app.py"]
```

Build it. Check the size.

```
$ docker build -t my-python-naive .
$ docker images my-python-naive
REPOSITORY       TAG       SIZE
my-python-naive  latest    924MB
```

Still 924MB.

This does not work. Docker images are made of layers. Each `RUN` instruction creates a new layer. The `apt-get remove` command runs in a new layer, but the previous layer that installed gcc still exists in the image history. Docker image layers are additive. You cannot delete data from a previous layer by writing to a new one.

```
Layer 1: python:3.12 base          (450MB)
Layer 2: pip install               (200MB)
Layer 3: COPY source code            (1MB)
Layer 4: apt-get remove gcc        (+5MB)  <-- adds a "deletion record"
--------------------------------------------
Total pulled:                      (924MB)  <-- nothing was actually removed
```

The deletion layer just records which files to hide. The original files still exist in earlier layers and still get transferred, stored, and pulled.

Some people try harder:

```dockerfile
# Even worse: chaining everything into one RUN
FROM python:3.12
RUN apt-get update && \
    apt-get install -y gcc python3-dev && \
    pip install --no-cache-dir numpy pandas && \
    apt-get remove -y gcc python3-dev && \
    apt-get autoremove -y && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*
COPY . /app
WORKDIR /app
CMD ["python", "app.py"]
```

This helps slightly because the install and cleanup happen in the same layer. But it makes your Dockerfile unreadable, destroys Docker cache effectiveness (more on this in Module 05), and still leaves a bloated base image underneath.

The naive way does not work. You cannot shrink an image by cleaning up inside it. You need a fundamentally different approach.

---

## The Right Way: Multi-Stage Builds

The key insight: **you do not need to ship the kitchen. You only need to ship the meal.**

A multi-stage Dockerfile has multiple `FROM` instructions. Each `FROM` starts a new, independent image. You use the first image (the "builder") to compile, install, and prepare everything. Then you copy only the finished artifacts into a clean, minimal final image.

```dockerfile
# ===== Stage 1: Builder =====
FROM python:3.12 AS builder

WORKDIR /app
COPY requirements.txt .
RUN pip install --no-cache-dir --prefix=/install -r requirements.txt

COPY . .

# ===== Stage 2: Runtime =====
FROM python:3.12-slim

WORKDIR /app

# Copy installed packages from builder
COPY --from=builder /install /usr/local

# Copy application code from builder
COPY --from=builder /app .

CMD ["python", "app.py"]
```

What happens here:

```
Stage 1 (builder):
  python:3.12          -- full image with gcc, make, headers
  pip install          -- compiles C extensions if needed
  Result: compiled packages in /install

Stage 2 (runtime):
  python:3.12-slim     -- minimal image, no build tools
  COPY --from=builder  -- copies ONLY the compiled packages
  Result: runs the app, no build tools present

Final image contains:
  python:3.12-slim base    (~150MB)
  Installed packages        (~30MB)
  Your application code      (~1MB)
  ---------------------------
  Total                    (~181MB)   vs 924MB before
```

The builder image is thrown away. It never gets pushed to a registry. It never gets pulled by a production server. It exists only during the build process.

Build and compare:

```bash
# Single-stage (old way)
docker build -t myapp-single -f Dockerfile.single .
docker images myapp-single
# REPOSITORY      TAG       SIZE
# myapp-single    latest    924MB

# Multi-stage (new way)
docker build -t myapp-multi -f Dockerfile.multi .
docker images myapp-multi
# REPOSITORY      TAG       SIZE
# myapp-multi     latest    181MB
```

That is a 5x reduction. And we can go much further.

### How COPY --from Works

The `--from` flag is the magic of multi-stage builds. It copies files from one stage to another:

```dockerfile
# Reference by stage name
COPY --from=builder /app/dist ./dist

# Reference by stage number (0-indexed)
COPY --from=0 /app/dist ./dist

# Reference an external image (not built in this Dockerfile)
COPY --from=nginx:alpine /etc/nginx/nginx.conf /etc/nginx/
```

You can copy from any named stage, any numbered stage, or even from a completely separate image pulled from a registry.

---

## Language-by-Language Examples

Every language has its own build toolchain. Here is the multi-stage pattern for each major ecosystem.

### Python

Python is tricky because many packages need C compilers to install. Multi-stage keeps those compilers out of your final image.

```dockerfile
# ===== Builder =====
FROM python:3.12 AS builder

WORKDIR /app

# Install build dependencies (only needed in builder)
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    gcc \
    g++ \
    libffi-dev \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY requirements.txt .
RUN pip install --no-cache-dir --prefix=/install -r requirements.txt

# ===== Runtime =====
FROM python:3.12-slim

# Install only runtime libraries (not dev headers)
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    libffi8 \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /install /usr/local
COPY . .

# Run as non-root user
RUN useradd --create-home appuser
USER appuser

CMD ["python", "app.py"]
```

Size comparison:

| Approach | Size |
|---|---|
| `FROM python:3.12` (full) | 924MB |
| `FROM python:3.12-slim` (no multi-stage) | 185MB |
| Multi-stage with `python:3.12-slim` | 150MB |
| Multi-stage with distroless | 85MB |

### Node.js

Node.js applications often have hundreds of megabytes of `node_modules`. Multi-stage builds let you separate dev dependencies from production dependencies.

```dockerfile
# ===== Builder =====
FROM node:20 AS builder

WORKDIR /app

# Install ALL dependencies (including devDependencies)
COPY package.json package-lock.json ./
RUN npm ci

# Build the application (TypeScript, Webpack, etc.)
COPY . .
RUN npm run build

# Prune dev dependencies
RUN npm prune --production

# ===== Runtime =====
FROM node:20-slim

WORKDIR /app

# Copy only production node_modules and built output
COPY --from=builder /app/node_modules ./node_modules
COPY --from=builder /app/dist ./dist
COPY --from=builder /app/package.json ./

# Run as non-root
USER node

CMD ["node", "dist/index.js"]
```

For an even smaller image, use Alpine:

```dockerfile
# ===== Builder =====
FROM node:20-alpine AS builder

WORKDIR /app
COPY package.json package-lock.json ./
RUN npm ci
COPY . .
RUN npm run build
RUN npm prune --production

# ===== Runtime =====
FROM node:20-alpine

WORKDIR /app
COPY --from=builder /app/node_modules ./node_modules
COPY --from=builder /app/dist ./dist
COPY --from=builder /app/package.json ./

USER node

CMD ["node", "dist/index.js"]
```

Size comparison:

| Approach | Size |
|---|---|
| `FROM node:20` (full) | 1.1GB |
| Multi-stage with `node:20-slim` | 210MB |
| Multi-stage with `node:20-alpine` | 130MB |

### Go

Go is the multi-stage poster child. Go compiles to a single static binary. No runtime. No dependencies. No interpreter. You can copy that one file into an empty image.

```dockerfile
# ===== Builder =====
FROM golang:1.22 AS builder

WORKDIR /app

# Cache dependencies
COPY go.mod go.sum ./
RUN go mod download

# Build a static binary
COPY . .
RUN CGO_ENABLED=0 GOOS=linux go build \
    -ldflags="-s -w" \
    -o /server \
    ./cmd/server

# ===== Runtime =====
FROM scratch

# Copy the CA certificates for HTTPS calls
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

# Copy the binary
COPY --from=builder /server /server

EXPOSE 8080
ENTRYPOINT ["/server"]
```

`FROM scratch` means an image with nothing in it. No shell. No libc. No OS. Just your binary. This is the smallest possible container.

```bash
$ docker images my-go-app
REPOSITORY   TAG       SIZE
my-go-app    latest    12MB
```

12 megabytes. Down from over 1GB with `FROM golang:1.22`.

If you need a shell for debugging, use Alpine instead of scratch:

```dockerfile
# ===== Runtime (debuggable) =====
FROM alpine:3.19

RUN apk --no-cache add ca-certificates tzdata

COPY --from=builder /server /server

EXPOSE 8080
ENTRYPOINT ["/server"]
```

This adds ~7MB for Alpine and its utilities, giving you a shell and package manager inside the container.

Size comparison:

| Approach | Size |
|---|---|
| `FROM golang:1.22` | 1.2GB |
| Multi-stage with `alpine:3.19` | 25MB |
| Multi-stage with `scratch` | 12MB |

### Rust

Like Go, Rust compiles to a native binary. The build is slow, but the result is tiny.

```dockerfile
# ===== Builder =====
FROM rust:1.78 AS builder

WORKDIR /app

# Cache dependencies (same trick as Go)
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Now copy real source and rebuild (only app code recompiles)
COPY src ./src
RUN touch src/main.rs && cargo build --release

# ===== Runtime =====
FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/myapp /usr/local/bin/myapp

RUN useradd --create-home appuser
USER appuser

CMD ["myapp"]
```

For a fully static Rust binary (like Go's `FROM scratch` approach):

```dockerfile
# ===== Builder =====
FROM rust:1.78 AS builder

WORKDIR /app
COPY . .

# Build with musl for static linking
RUN apt-get update && apt-get install -y musl-tools && \
    rustup target add x86_64-unknown-linux-musl && \
    cargo build --release --target x86_64-unknown-linux-musl

# ===== Runtime =====
FROM scratch

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/myapp /myapp

ENTRYPOINT ["/myapp"]
```

Size comparison:

| Approach | Size |
|---|---|
| `FROM rust:1.78` | 1.6GB |
| Multi-stage with `debian:bookworm-slim` | 120MB |
| Multi-stage with `scratch` (static) | 8MB |

### Java

Java applications ship as JAR files or use newer tools like `jlink` to create custom runtimes. Multi-stage builds keep the JDK out of production.

```dockerfile
# ===== Builder =====
FROM maven:3.9-eclipse-temurin-21 AS builder

WORKDIR /app
COPY pom.xml .
RUN mvn dependency:go-offline

COPY src ./src
RUN mvn package -DskipTests

# ===== Runtime =====
FROM eclipse-temurin:21-jre

WORKDIR /app

COPY --from=builder /app/target/*.jar app.jar

RUN useradd --create-home appuser
USER appuser

EXPOSE 8080
ENTRYPOINT ["java", "-jar", "app.jar"]
```

For the smallest possible Java image, use `jlink` to create a custom runtime that includes only the modules your application needs:

```dockerfile
# ===== Builder =====
FROM maven:3.9-eclipse-temurin-21 AS builder

WORKDIR /app
COPY pom.xml .
RUN mvn dependency:go-offline
COPY src ./src
RUN mvn package -DskipTests

# ===== Jlink =====
FROM eclipse-temurin:21 AS jlink

# Create minimal JVM with only required modules
RUN jlink \
    --add-modules java.base,java.net.http,java.logging \
    --strip-debug \
    --no-man-pages \
    --no-header-files \
    --compress=zip-6 \
    --output /jvm

# ===== Runtime =====
FROM debian:bookworm-slim

COPY --from=jlink /jvm /opt/jvm
ENV PATH="/opt/jvm/bin:${PATH}"

WORKDIR /app
COPY --from=builder /app/target/*.jar app.jar

RUN useradd --create-home appuser
USER appuser

EXPOSE 8080
ENTRYPOINT ["java", "-jar", "app.jar"]
```

Size comparison:

| Approach | Size |
|---|---|
| `FROM maven:3.9-eclipse-temurin-21` (full JDK + Maven) | 1.4GB |
| Multi-stage with `eclipse-temurin:21-jre` | 320MB |
| Multi-stage with custom `jlink` runtime | 95MB |

---

## The Production Way: Choosing Your Base Image

The base image you choose for your runtime stage has a massive impact on size, security, and debugging capability. Here are your options, ranked from largest to smallest.

### Full Images

```dockerfile
FROM python:3.12
FROM node:20
FROM golang:1.22
```

These are Debian-based images with everything installed: build tools, compilers, package managers, shells, and utilities. They exist for building software, not for running it.

**Size:** 800MB - 1.6GB
**Use for:** Development. Never production.

### Slim Images

```dockerfile
FROM python:3.12-slim
FROM node:20-slim
```

These are stripped-down Debian images with only the minimum runtime libraries. No compilers. No build tools. Still has a shell and package manager (`apt-get`).

**Size:** 100MB - 200MB
**Use for:** Production when you need Debian compatibility or a debugging shell.

### Alpine Images

```dockerfile
FROM python:3.12-alpine
FROM node:20-alpine
```

Based on Alpine Linux instead of Debian. Uses `musl` libc instead of `glibc`. Includes a shell (`ash`) and package manager (`apk`). Much smaller than slim, but some Python/C packages fail to compile against musl.

**Size:** 50MB - 130MB
**Use for:** Production when your dependencies compile cleanly against musl. Test thoroughly.

### Distroless Images

```dockerfile
FROM gcr.io/distroless/python3
FROM gcr.io/distroless/nodejs20
FROM gcr.io/distroless/java21
```

Google's distroless images contain only your application and its runtime dependencies. No shell. No package manager. No OS utilities. Nothing extra.

**Size:** 30MB - 120MB
**Use for:** Production when security matters. Extremely small attack surface.

Distroless example:

```dockerfile
# ===== Builder =====
FROM python:3.12 AS builder

WORKDIR /app
COPY requirements.txt .
RUN pip install --no-cache-dir --prefix=/install -r requirements.txt

COPY . .

# ===== Runtime =====
FROM gcr.io/distroless/python3

WORKDIR /app
COPY --from=builder /install /usr/local
COPY --from=builder /app .

CMD ["app.py"]
```

**Important:** Distroless images have no shell. You cannot `docker exec -it container sh`. This is a feature, not a bug. If an attacker breaks into your container, they cannot get a shell either. For debugging, use ephemeral debug containers (Kubernetes `kubectl debug`) or structured logging.

When you absolutely need to debug a distroless container, you can attach a debug image:

```bash
# Kubernetes: attach a debug container
kubectl debug -it my-pod --image=busybox --target=my-container

# Docker: not directly possible -- use logs instead
docker logs my-container
```

### Scratch: The Empty Image

```dockerfile
FROM scratch
```

Nothing. Zero. An empty filesystem. You copy in your binary and nothing else. Only works for fully static binaries (Go, Rust with musl).

**Size:** 0MB + your binary
**Use for:** Maximum minimalism. No OS, no shell, no nothing.

### Base Image Comparison Table

| Base Image | Size | Shell | Package Manager | libc | Best For |
|---|---|---|---|---|---|
| `python:3.12` | 924MB | bash | apt | glibc | Building, never production |
| `python:3.12-slim` | 150MB | bash | apt | glibc | Safe production default |
| `python:3.12-alpine` | 50MB | ash | apk | musl | Smallest with debugging |
| `distroless/python3` | 45MB | none | none | glibc | Maximum security |
| `scratch` | 0MB | none | none | none | Static Go/Rust only |

---

## Best Practices

### 1. Name Your Stages

```dockerfile
# BAD -- hard to read, fragile with number references
FROM golang:1.22 AS builder
RUN go build -o /server .
FROM alpine:3.19
COPY --from=0 /server /server

# GOOD -- clear intent, stable references
FROM golang:1.22 AS builder
RUN go build -o /server .
FROM alpine:3.19 AS runtime
COPY --from=builder /server /server
```

### 2. Copy Dependencies Before Source Code

```dockerfile
# BAD -- any source change invalidates the dependency cache
FROM node:20 AS builder
WORKDIR /app
COPY . .
RUN npm ci

# GOOD -- dependencies cached independently of source changes
FROM node:20 AS builder
WORKDIR /app
COPY package.json package-lock.json ./
RUN npm ci
COPY . .
```

This is critical for build speed. We cover this in detail in Module 05 (Image Caching).

### 3. Use .dockerignore

Your `.dockerignore` file determines what gets sent to the Docker daemon. Even with multi-stage builds, a massive build context slows everything down.

```
# .dockerignore
.git
node_modules
target
*.md
.env
.env.*
__pycache__
*.pyc
.pytest_cache
coverage
dist
build
.DS_Store
```

### 4. Run as Non-Root

```dockerfile
# BAD -- runs as root
FROM python:3.12-slim
WORKDIR /app
COPY . .
CMD ["python", "app.py"]

# GOOD -- runs as unprivileged user
FROM python:3.12-slim
WORKDIR /app
COPY . .
RUN useradd --create-home --no-log-init appuser
USER appuser
CMD ["python", "app.py"]
```

If your base image does not have `useradd` (Alpine, distroless), use numeric UIDs:

```dockerfile
# Alpine
RUN adduser -D -u 1001 appuser
USER appuser

# Or just use a numeric UID (works everywhere)
USER 1001
```

### 5. Combine Multi-Stage with Specific Tags

```dockerfile
# BAD -- unpredictable over time
FROM python
FROM node

# GOOD -- pinned to a specific version
FROM python:3.12.4-slim-bookworm
FROM node:20.14-alpine3.20
```

Pin your base images. `python:3.12` might resolve to a different minor version next month. Your build should be reproducible.

### 6. Use Build Arguments for Flexibility

```dockerfile
ARG PYTHON_VERSION=3.12
ARG NODE_VERSION=20

FROM python:${PYTHON_VERSION}-slim AS python-builder
# ...

FROM node:${NODE_VERSION}-alpine AS node-builder
# ...

FROM python:${PYTHON_VERSION}-slim AS runtime
# ...
```

### 7. Clean Up in the Same Layer (When Multi-Stage Is Not Possible)

Sometimes you cannot use multi-stage builds (e.g., in CI environments that only support a single stage). In that case, clean up in the same layer:

```dockerfile
FROM python:3.12-slim

RUN apt-get update && \
    apt-get install -y --no-install-recommends gcc libffi-dev && \
    pip install --no-cache-dir cryptography && \
    apt-get purge -y --auto-remove gcc libffi-dev && \
    rm -rf /var/lib/apt/lists/*
```

Everything in one `RUN` so the install and cleanup happen in the same layer.

---

## Real-World Size Comparisons

Here are realistic numbers for actual applications, not hello-world toys.

### Python Web Application (FastAPI + SQLAlchemy)

```
Single-stage (python:3.12):          924MB
Multi-stage (python:3.12-slim):      155MB  (83% reduction)
Multi-stage (distroless/python3):     89MB  (90% reduction)
```

### Node.js API (Express + TypeScript + Prisma)

```
Single-stage (node:20):             1.1GB
Multi-stage (node:20-slim):          210MB  (81% reduction)
Multi-stage (node:20-alpine):        135MB  (88% reduction)
```

### Go Microservice

```
Single-stage (golang:1.22):         1.2GB
Multi-stage (alpine:3.19):            25MB  (98% reduction)
Multi-stage (scratch):                12MB  (99% reduction)
```

### Rust CLI Tool

```
Single-stage (rust:1.78):           1.6GB
Multi-stage (debian:bookworm-slim):  120MB  (93% reduction)
Multi-stage (scratch):                 8MB  (99.5% reduction)
```

### Java Spring Boot Service

```
Single-stage (maven + JDK):         1.4GB
Multi-stage (eclipse-temurin:21-jre): 320MB  (77% reduction)
Multi-stage (jlink custom runtime):   95MB  (93% reduction)
```

---

## Advanced Patterns

### Multi-Service Builds

Build multiple services in a single Dockerfile:

```dockerfile
# Shared builder
FROM golang:1.22 AS builder
WORKDIR /app
COPY go.mod go.sum ./
RUN go mod download
COPY . .

# Build API server
RUN CGO_ENABLED=0 go build -ldflags="-s -w" -o /api ./cmd/api
# Build worker
RUN CGO_ENABLED=0 go build -ldflags="-s -w" -o /worker ./cmd/worker

# API image
FROM scratch AS api
COPY --from=builder /api /api
ENTRYPOINT ["/api"]

# Worker image
FROM scratch AS worker
COPY --from=builder /worker /worker
ENTRYPOINT ["/worker"]
```

Build a specific target with `--target`:

```bash
docker build --target api -t myapp-api .
docker build --target worker -t myapp-worker .
```

### Cross-Compilation

Build for a different architecture without emulation:

```dockerfile
FROM --platform=$BUILDPLATFORM golang:1.22 AS builder

ARG TARGETOS
ARG TARGETARCH

WORKDIR /app
COPY . .
RUN CGO_ENABLED=0 GOOS=${TARGETOS} GOARCH=${TARGETARCH} \
    go build -ldflags="-s -w" -o /server ./cmd/server

FROM scratch
COPY --from=builder /server /server
ENTRYPOINT ["/server"]
```

Build for ARM64 from an x86 machine:

```bash
docker buildx build --platform linux/arm64 -t myapp-arm64 .
```

No QEMU emulation needed during the build. Go's cross-compiler does the work natively.

### Distroless with Debug Variant

Google provides debug variants of distroless images that include a busybox shell:

```dockerfile
# Production
FROM gcr.io/distroless/python3

# Debug (has a shell)
FROM gcr.io/distroless/python3:debug
```

Use the debug variant in staging, the regular one in production.

---

## Hands-On Lab

These exercises are self-contained. Run them in order.

### Exercise 1: Measure the Bloat

Create a simple Python app and measure how big a single-stage image is.

```bash
mkdir -p /tmp/exercise1 && cd /tmp/exercise1
```

Create `app.py`:

```python
from flask import Flask
app = Flask(__name__)

@app.route("/")
def hello():
    return "Hello from a bloated container!\n"

if __name__ == "__main__":
    app.run(host="0.0.0.0", port=5000)
```

Create `requirements.txt`:

```
flask==3.0.3
```

Create `Dockerfile.single`:

```dockerfile
FROM python:3.12

WORKDIR /app
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt
COPY . .

CMD ["python", "app.py"]
```

Build and measure:

```bash
docker build -t exercise1-single -f Dockerfile.single .
docker images exercise1-single
```

Write down the size. It will be around 1GB.

### Exercise 2: Shrink It with Multi-Stage

Create `Dockerfile.multi` in the same directory:

```dockerfile
# ===== Builder =====
FROM python:3.12 AS builder

WORKDIR /app
COPY requirements.txt .
RUN pip install --no-cache-dir --prefix=/install -r requirements.txt

# ===== Runtime =====
FROM python:3.12-slim

WORKDIR /app
COPY --from=builder /install /usr/local
COPY . .

CMD ["python", "app.py"]
```

Build and measure:

```bash
docker build -t exercise1-multi -f Dockerfile.multi .
docker images exercise1-multi
```

Compare the sizes. You should see a 5x or greater reduction.

### Exercise 3: Go to Scratch

Create a Go app that compiles to nearly nothing.

```bash
mkdir -p /tmp/exercise3 && cd /tmp/exercise3
```

Create `main.go`:

```go
package main

import (
	"fmt"
	"net/http"
)

func main() {
	http.HandleFunc("/", func(w http.ResponseWriter, r *http.Request) {
		fmt.Fprintln(w, "Hello from a tiny container!")
	})
	fmt.Println("Server starting on :8080")
	http.ListenAndServe(":8080", nil)
}
```

Create `go.mod`:

```
module exercise3

go 1.22
```

Create `Dockerfile`:

```dockerfile
FROM golang:1.22 AS builder

WORKDIR /app
COPY go.mod ./
COPY *.go ./
RUN CGO_ENABLED=0 go build -ldflags="-s -w" -o /server .

FROM scratch
COPY --from=builder /server /server
EXPOSE 8080
ENTRYPOINT ["/server"]
```

Build and measure:

```bash
docker build -t exercise3 .
docker images exercise3
```

The image should be around 8-12MB. Run it:

```bash
docker run -p 8080:8080 exercise3 &
curl http://localhost:8080
```

### Exercise 4: Node.js with Build Step

Build a TypeScript Node.js application with multi-stage.

```bash
mkdir -p /tmp/exercise4 && cd /tmp/exercise4
```

Create `package.json`:

```json
{
  "name": "exercise4",
  "version": "1.0.0",
  "scripts": {
    "build": "tsc",
    "start": "node dist/index.js"
  },
  "dependencies": {
    "express": "^4.19.2"
  },
  "devDependencies": {
    "@types/express": "^4.17.21",
    "typescript": "^5.4.5"
  }
}
```

Create `tsconfig.json`:

```json
{
  "compilerOptions": {
    "target": "ES2020",
    "module": "commonjs",
    "outDir": "./dist",
    "strict": true,
    "esModuleInterop": true
  },
  "include": ["src/**/*"]
}
```

Create `src/index.ts`:

```typescript
import express from "express";

const app = express();
const PORT = 3000;

app.get("/", (_req, res) => {
  res.send("Hello from TypeScript in a tiny container!");
});

app.listen(PORT, () => {
  console.log(`Server running on port ${PORT}`);
});
```

Create `Dockerfile`:

```dockerfile
# ===== Builder =====
FROM node:20-alpine AS builder

WORKDIR /app
COPY package.json package-lock.json* ./
RUN npm install

COPY tsconfig.json ./
COPY src ./src
RUN npm run build

# Prune dev dependencies
RUN npm prune --production

# ===== Runtime =====
FROM node:20-alpine

WORKDIR /app

COPY --from=builder /app/node_modules ./node_modules
COPY --from=builder /app/dist ./dist
COPY --from=builder /app/package.json ./

USER node

CMD ["node", "dist/index.js"]
```

Build and compare:

```bash
# Without multi-stage (for comparison)
docker build -t exercise4-single --target=builder .
docker images exercise4-single

# With multi-stage
docker build -t exercise4 .
docker images exercise4
```

### Exercise 5: Distroless

Take the Python app from Exercise 1 and rebuild it with distroless.

```bash
cd /tmp/exercise1
```

Create `Dockerfile.distroless`:

```dockerfile
FROM python:3.12 AS builder

WORKDIR /app
COPY requirements.txt .
RUN pip install --no-cache-dir --prefix=/install -r requirements.txt

COPY . .

FROM gcr.io/distroless/python3

WORKDIR /app
COPY --from=builder /install /usr/local
COPY --from=builder /app .

CMD ["app.py"]
```

Build and measure:

```bash
docker build -t exercise5-distroless -f Dockerfile.distroless .
docker images exercise5-distroless
```

Try to get a shell inside it:

```bash
docker run -it exercise5-distroless sh
# This will fail -- there is no shell. That is the point.
```

Check the logs instead:

```bash
docker run -p 5001:5000 exercise5-distroless &
curl http://localhost:5001
docker logs <container-id>
```

### Exercise 6: Challenge -- Shrink a Java App

Build the smallest possible Java container.

```bash
mkdir -p /tmp/exercise6 && cd /tmp/exercise6
```

Create `pom.xml`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<project xmlns="http://maven.apache.org/POM/4.0.0"
         xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
         xsi:schemaLocation="http://maven.apache.org/POM/4.0.0
         http://maven.apache.org/xsd/maven-4.0.0.xsd">
    <modelVersion>4.0.0</modelVersion>
    <groupId>com.example</groupId>
    <artifactId>exercise6</artifactId>
    <version>1.0</version>
    <properties>
        <maven.compiler.source>21</maven.compiler.source>
        <maven.compiler.target>21</maven.compiler.target>
    </properties>
</project>
```

Create `src/main/java/com/example/App.java`:

```java
package com.example;

import com.sun.net.httpserver.HttpServer;
import java.io.IOException;
import java.io.OutputStream;
import java.net.InetSocketAddress;

public class App {
    public static void main(String[] args) throws IOException {
        HttpServer server = HttpServer.create(new InetSocketAddress(8080), 0);
        server.createContext("/", exchange -> {
            String response = "Hello from a tiny Java container!\n";
            exchange.sendResponseHeaders(200, response.length());
            OutputStream os = exchange.getResponseBody();
            os.write(response.getBytes());
            os.close();
        });
        System.out.println("Server starting on :8080");
        server.start();
    }
}
```

Your task: write a multi-stage Dockerfile that uses `jlink` to create the smallest possible image. The image should contain only the JVM modules that `com.sun.net.httpserver` requires. Target under 100MB.

---

## Limitation

You have shrunk your images from 1GB to 12MB. That is a massive win. But watch what happens when you iterate on your code:

```bash
# Change one line in your source code
echo "// fixed a typo" >> app.py

# Rebuild
docker build -t myapp .
```

```
Step 8/12 : RUN pip install --no-cache-dir -r requirements.txt
 ---> Running in 4a8b3c2d1e5f
... (downloads and installs ALL dependencies again, even though nothing changed)
```

Every time you change a single line of source code, Docker rebuilds everything from that point forward. The `pip install` that takes 30 seconds runs again even though `requirements.txt` has not changed. The `go mod download` that fetches 200MB of dependencies runs again even though `go.mod` is identical.

Your images are small now, but your builds are slow. On a CI server, this wastes minutes on every push. On your laptop, it kills your flow.

Multi-stage builds solved the size problem. They did not solve the speed problem.

**Next module:** [05-image-caching](../05-image-caching/) -- How Docker layer caching works, how to structure your Dockerfiles to avoid unnecessary rebuilds, and how to make builds that take seconds instead of minutes.
