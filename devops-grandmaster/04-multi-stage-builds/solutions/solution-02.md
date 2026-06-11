# Solution 02: Convert Single-Stage to Multi-Stage

## Part A: Build-Time vs Runtime Instructions

| Instruction | Category | Reason |
|---|---|---|
| `FROM node:20` | Both (base image) | Builder needs the full toolchain; runtime uses a different base |
| `WORKDIR /app` | Both | Both stages need a working directory |
| `COPY package.json package-lock.json ./` | Build-time | Only needed to install dependencies during build |
| `RUN npm ci` | Build-time | Installs ALL dependencies including devDependencies (TypeScript, type defs, test frameworks). Runtime only needs production deps. |
| `COPY . .` | Build-time (in builder) | Source code is needed to compile TypeScript. Runtime needs only the compiled output. |
| `RUN npm run build` | Build-time | Compiles TypeScript to JavaScript. The compiler is a devDependency. |
| `EXPOSE 3000` | Runtime | Documents the listening port for the running application |
| `CMD ["node", "dist/index.js"]` | Runtime | Starts the compiled application |

The critical insight: `npm ci` installs everything listed in `package.json`, including `devDependencies`. TypeScript, `@types/*` packages, test runners, and linting tools are all devDependencies. The running application does not need any of them -- it only needs the compiled JavaScript in `dist/` and the production `node_modules`.

---

## Part B: The Builder Stage

```dockerfile
FROM node:20 AS builder

WORKDIR /app
COPY package.json package-lock.json ./
RUN npm ci

COPY . .
RUN npm run build
```

**Why this works:**
- `AS builder` names the stage so later stages can reference it with `COPY --from=builder`.
- `npm ci` installs ALL dependencies (including TypeScript) because the build step needs them.
- `COPY . .` comes after `npm ci` so that code changes do not invalidate the dependency cache.
- `npm run build` compiles TypeScript to JavaScript in `dist/`.

---

## Part C: The Runtime Stage

```dockerfile
FROM node:20-slim

WORKDIR /app

# Copy production dependencies and compiled output
COPY --from=builder /app/node_modules ./node_modules
COPY --from=builder /app/dist ./dist
COPY --from=builder /app/package.json ./

# Run as non-root
RUN useradd --create-home appuser
USER appuser

EXPOSE 3000
CMD ["node", "dist/index.js"]
```

**Files copied from builder:**

1. `node_modules/` -- but this still contains devDependencies (we fix this in Part D).
2. `dist/` -- the compiled JavaScript output from `tsc`.
3. `package.json` -- needed for metadata and potentially for `npm start`.

**Why `node:20-slim`:**
- `node:20` is 1.1GB (full Debian with build tools).
- `node:20-slim` is ~200MB (minimal Debian with only Node.js runtime).
- The runtime stage does not need compilers, so slim is sufficient.

---

## Part D: The Prune Step

Add `npm prune --production` to the builder stage AFTER the build step:

```dockerfile
FROM node:20 AS builder

WORKDIR /app
COPY package.json package-lock.json ./
RUN npm ci

COPY . .
RUN npm run build

# Remove devDependencies before copying to runtime
RUN npm prune --production
```

**Why this matters:**

A typical TypeScript project has devDependencies like:
- `typescript` (~70MB)
- `@types/express` (~1MB)
- `@types/node` (~3MB)
- Test frameworks, linters, formatters (~50MB+)

The running application uses none of these. `npm prune --production` removes everything listed under `devDependencies` in `package.json`, leaving only the packages the application imports at runtime.

Without pruning, you copy ~130MB of unnecessary packages into the runtime image. With pruning, you copy only ~30-50MB of production dependencies.

---

## Part E: Choosing the Right Runtime Base

| Base Image | Size | libc | Shell | Best For |
|---|---|---|---|---|
| `node:20` | 1.1GB | glibc | bash | Development only |
| `node:20-slim` | 200MB | glibc | bash | Safe production default |
| `node:20-alpine` | 130MB | musl | ash | Smallest with debugging |

**Recommendation for production: `node:20-slim`**

Reasons:
- Uses glibc, which is what most Node.js native modules are compiled against.
- Has a shell for debugging (`docker exec -it container bash`).
- Has `apt-get` for installing runtime dependencies if needed (e.g., `curl` for health checks).
- Predictable behavior -- no musl compatibility surprises.

**When to use `node:20-alpine`:**
- When image size is critical (saves ~70MB).
- When you have tested all native modules against musl.
- When your application has no native addons or only addons known to work with musl.

**When NOT to use Alpine:**
- When your application uses native Node.js addons (bcrypt, sharp, sqlite3) that may not have musl-compatible prebuilt binaries.
- When you need to debug inside the container and are unfamiliar with `ash` and `apk`.
- When your team is not experienced with Alpine-specific troubleshooting.

---

## Part F: Final Dockerfile

```dockerfile
# ===== Stage 1: Builder =====
FROM node:20 AS builder

WORKDIR /app

# Copy dependency manifests first for cache efficiency
COPY package.json package-lock.json ./
RUN npm ci

# Copy source code and build
COPY . .
RUN npm run build

# Remove devDependencies
RUN npm prune --production

# ===== Stage 2: Runtime =====
FROM node:20-slim

WORKDIR /app

# Copy only what is needed to run
COPY --from=builder /app/node_modules ./node_modules
COPY --from=builder /app/dist ./dist
COPY --from=builder /app/package.json ./

# Security: run as non-root
RUN useradd --create-home appuser
USER appuser

EXPOSE 3000
CMD ["node", "dist/index.js"]
```

**Expected size comparison:**

| Version | Image Size | Reduction |
|---|---|---|
| Original (single-stage `node:20`) | ~1.1GB | baseline |
| Multi-stage with `node:20-slim` | ~210MB | ~81% |
| Multi-stage with `node:20-alpine` | ~135MB | ~88% |

Build and compare:

```bash
# Build the single-stage version (stop at builder)
docker build -t exercise2-single --target=builder .
docker images exercise2-single
# Expected: ~1.1GB

# Build the multi-stage version
docker build -t exercise2-multi .
docker images exercise2-multi
# Expected: ~210MB
```

---

## Common Mistakes

1. **Not pruning devDependencies.** If you skip `npm prune --production`, you copy TypeScript, test frameworks, and type definitions into the production image. This wastes 50-130MB.

2. **Copying the entire builder `node_modules` without pruning first.** Even with multi-stage, if you copy `node_modules` before pruning, all devDependencies end up in the runtime image.

3. **Using `npm install` instead of `npm ci` in the builder.** `npm ci` installs from `package-lock.json` exactly, producing reproducible builds. `npm install` may update the lock file and produce different results on different machines.

4. **Copying `package-lock.json` to the runtime stage.** The lock file is only needed during `npm ci`. The runtime stage does not need it.

5. **Forgetting to set `WORKDIR` in the runtime stage.** Without it, files land in `/` (the root directory), and `CMD` may not find the application.

6. **Running as root in the runtime stage.** Always create a non-root user and switch to it with `USER`. The `node` user is built into Alpine images if you want to skip the `useradd` step.

7. **Copying `src/` to the runtime stage.** The TypeScript source code is not needed at runtime. Only `dist/` (compiled JavaScript) is needed.

8. **Not copying `package.json` to the runtime stage.** Some applications read `package.json` at runtime for version information or metadata. Copy it to be safe.
