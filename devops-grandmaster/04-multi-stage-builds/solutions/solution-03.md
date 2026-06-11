# Solution 03: Multi-Stage Dockerfiles for Three Languages

## Part A: Go Application

### Dockerfile

```dockerfile
# ===== Stage 1: Builder =====
FROM golang:1.22 AS builder

WORKDIR /app

# Copy dependency manifest first (cache-friendly)
COPY go.mod ./

# Copy source code
COPY *.go ./

# Build a fully static binary
# CGO_ENABLED=0: no libc dependency, fully static
# -ldflags="-s -w": strip debug symbols and DWARF info
RUN CGO_ENABLED=0 GOOS=linux go build -ldflags="-s -w" -o /server .

# ===== Stage 2: Runtime =====
FROM scratch

# Copy CA certificates for HTTPS support
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

# Copy the compiled binary
COPY --from=builder /server /server

EXPOSE 8080
ENTRYPOINT ["/server"]
```

**Why this works:**

Go compiles to a single static binary. There is no interpreter, no runtime, no shared libraries needed. `CGO_ENABLED=0` ensures the binary is fully static -- it does not link against libc at all. This means you can copy the binary into `FROM scratch` (an empty filesystem) and it will run.

**Key flags explained:**
- `CGO_ENABLED=0`: Disables CGo, which would link against the system's C library. Without this, the binary depends on libc.so and cannot run in `scratch`.
- `-ldflags="-s -w"`: `-s` strips the symbol table, `-w` strips DWARF debugging info. Reduces binary size by 20-30%.
- `GOOS=linux`: Explicitly targets Linux. Redundant when building inside a Linux container, but makes the intent clear.

**Why `FROM scratch`:**
- Scratch is an empty image -- 0 bytes.
- The only thing in the final image is your binary (~8-12MB) and CA certificates (~200KB).
- No shell, no package manager, no OS. Nothing for an attacker to exploit.

**Why copy CA certificates:**
- If your Go application makes HTTPS calls (to APIs, databases, etc.), it needs the system's CA certificate bundle to verify TLS certificates.
- The builder stage (`golang:1.22`) has these certificates. Scratch has nothing.
- Copying them from the builder ensures HTTPS works in the final image.

**Expected size: 8-12MB**

---

## Part B: Python Application

### Dockerfile

```dockerfile
# ===== Stage 1: Builder =====
FROM python:3.12 AS builder

WORKDIR /app

# Install packages into a separate prefix
COPY requirements.txt .
RUN pip install --no-cache-dir --prefix=/install -r requirements.txt

# ===== Stage 2: Runtime =====
FROM python:3.12-slim

WORKDIR /app

# Copy installed packages from builder to runtime
# /install maps to /usr/local, which is on Python's default path
COPY --from=builder /install /usr/local

# Copy application code
COPY . .

# Create non-root user
RUN useradd --create-home appuser
USER appuser

EXPOSE 5000
CMD ["gunicorn", "--bind", "0.0.0.0:5000", "app:app"]
```

**Why `--prefix=/install`:**

By default, `pip install` puts packages in `/usr/local/lib/python3.12/site-packages`. If you copy from that path, you also copy pip metadata, the pip binary itself, and other cruft. Using `--prefix=/install` puts everything in a clean directory that maps cleanly to `/usr/local` when copied.

The directory structure under `/install` is:
```
/install/
  lib/python3.12/site-packages/   (the actual packages)
  bin/                             (console scripts like gunicorn)
```

When you `COPY --from=builder /install /usr/local`, the packages land in `/usr/local/lib/python3.12/site-packages/` -- exactly where Python looks for them.

**Why `python:3.12-slim` for runtime:**
- `python:3.12` is ~924MB (includes gcc, make, headers).
- `python:3.12-slim` is ~150MB (Python runtime only, no build tools).
- Flask and gunicorn do not need C compilers at runtime -- they were compiled in the builder stage.

**Why gunicorn instead of Flask's dev server:**
- Flask's built-in server (`app.run()`) is single-threaded and not designed for production.
- Gunicorn is a pre-fork WSGI server that handles concurrent requests properly.
- The dev server warns you about this if you start it directly.

**Why non-root user:**
- Running as root means a container escape gives the attacker root on the host.
- `useradd --create-home appuser` creates a user with a home directory.
- `USER appuser` switches all subsequent commands and the runtime process to that user.

**Expected size: ~170MB**

---

## Part C: Node.js Application

### Dockerfile

```dockerfile
# ===== Stage 1: Builder =====
FROM node:20-alpine AS builder

WORKDIR /app

# Install ALL dependencies (including devDependencies like TypeScript)
COPY package.json package-lock.json* ./
RUN npm ci

# Copy TypeScript config and source
COPY tsconfig.json ./
COPY src ./src

# Compile TypeScript to JavaScript
RUN npm run build

# Remove devDependencies (TypeScript, type defs, etc.)
RUN npm prune --production

# ===== Stage 2: Runtime =====
FROM node:20-alpine

WORKDIR /app

# Copy only what is needed to run
COPY --from=builder /app/node_modules ./node_modules
COPY --from=builder /app/dist ./dist
COPY --from=builder /app/package.json ./

# Run as the built-in 'node' user (non-root)
USER node

CMD ["node", "dist/index.js"]
```

**Why `node:20-alpine` for both stages:**
- Alpine is much smaller than the full `node:20` image (130MB vs 1.1GB).
- Using the same base for both stages avoids libc compatibility issues between builder and runtime.
- Node.js native modules compiled in the builder must work with the same libc in the runtime stage.

**The three-step build process:**
1. `npm ci` installs ALL dependencies, including TypeScript (a devDependency needed to compile).
2. `npm run build` runs `tsc`, which compiles `.ts` files to `.js` files in `dist/`.
3. `npm prune --production` removes TypeScript and other devDependencies from `node_modules`.

**Why prune AFTER build:**
- TypeScript must be installed during `npm ci` because `npm run build` needs it.
- After the build completes, TypeScript is no longer needed.
- Pruning before the build would remove TypeScript and cause `tsc` to fail.

**Why `USER node` instead of `useradd`:**
- Alpine-based Node.js images include a pre-created `node` user (UID 1000).
- No need to install `shadow` package for `useradd`.
- Just `USER node` is sufficient.

**What gets copied to runtime:**
- `node_modules/` -- production dependencies only (express, etc.).
- `dist/` -- compiled JavaScript output.
- `package.json` -- metadata, potentially read by the application.

What does NOT get copied:
- `src/` -- TypeScript source code (not needed at runtime).
- `tsconfig.json` -- TypeScript configuration (not needed at runtime).
- devDependencies (TypeScript, @types/*, etc.).

**Expected size: ~130MB**

---

## Common Mistakes

1. **Not using `CGO_ENABLED=0` for Go.** Without this flag, the Go binary links against libc and cannot run in `FROM scratch`. The container will crash with "not found" because the dynamic linker is missing.

2. **Forgetting CA certificates for Go in scratch.** If your Go app makes any HTTPS calls and you did not copy the CA certificates, TLS verification will fail with certificate errors.

3. **Not using `--prefix=/install` for Python.** Without the prefix, packages install to the default location in the builder. Copying from that location pulls in pip's own metadata and caches, making the image larger than necessary.

4. **Using Flask's dev server in production.** The `app.run()` server is single-threaded and not hardened for production use. Always use gunicorn (or uWSGI) for Python web applications.

5. **Pruning Node.js dependencies before building.** If you run `npm prune --production` before `npm run build`, TypeScript is removed and the build fails. The order must be: install all deps, build, then prune.

6. **Not copying `package.json` to the Node.js runtime stage.** Some applications and frameworks read `package.json` at runtime. Missing it can cause runtime errors.

7. **Using different Alpine versions for builder and runtime.** If the builder uses `node:20-alpine3.19` and the runtime uses `node:20-alpine3.18`, native modules compiled against one musl version may not work with the other. Always match the Alpine version.

8. **Installing build dependencies in the Python runtime stage.** If you `apt-get install gcc` in the runtime stage, you defeat the purpose of multi-stage builds. Build dependencies belong in the builder only.
