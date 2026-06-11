# Solution 03: Dockerfiles for Every Language

## Scenario A: Node.js Express API

### Complete Dockerfile

```dockerfile
FROM node:18-alpine

WORKDIR /app

COPY package.json package-lock.json ./
RUN npm ci --only=production

COPY . .

RUN addgroup -g 1001 appgroup && \
    adduser -u 1001 -G appgroup -s /bin/sh -D appuser
USER appuser

EXPOSE 3000

CMD ["node", "server.js"]
```

### Why This Works

**Base image:** `node:18-alpine` is the standard choice for production Node.js. Alpine keeps the image small (~170MB vs ~1GB for the full variant). Pinning to `18` ensures a specific major version.

**Dependency caching:** `package.json` and `package-lock.json` are copied first, then `npm ci` installs from the lockfile. This layer is cached and only rebuilds when dependencies change. `npm ci` (not `npm install`) ensures deterministic installs from the lockfile.

**Non-root user on Alpine:** Alpine uses BusyBox utilities, not GNU coreutils. The `adduser` syntax is different:
- `-u 1001` sets the user ID
- `-G appgroup` assigns the group
- `-s /bin/sh` sets the shell
- `-D` disables password prompting (equivalent to `--disabled-password` on Debian)

**CMD:** `node server.js` runs the application directly. The exec form ensures proper signal handling.

### Common Mistakes to Avoid

1. **Using `npm install` instead of `npm ci`.** `npm install` can update `package-lock.json`, leading to non-deterministic builds. `npm ci` installs exactly what the lockfile specifies.

2. **Not copying the lockfile.** If you only copy `package.json`, npm may resolve different versions on different builds.

3. **Using the full Node image.** `node:18` is over 1GB. `node:18-alpine` is ~170MB. There is rarely a reason to use the full image in production.

4. **Using Debian `adduser` syntax on Alpine.** The flags are different. Using `--disabled-password` on Alpine will fail.

---

## Scenario B: Go HTTP Server

### Complete Dockerfile

```dockerfile
# Build stage
FROM golang:1.21-alpine AS builder

WORKDIR /app

COPY go.mod go.sum ./
RUN go mod download

COPY . .
RUN CGO_ENABLED=0 GOOS=linux go build -o server .

# Run stage
FROM alpine:3.18

RUN apk --no-cache add ca-certificates

WORKDIR /app
COPY --from=builder /app/server .

EXPOSE 8080

CMD ["./server"]
```

### Why This Works

**Multi-stage build:** The build stage (`golang:1.21-alpine`) contains the Go compiler and all build tools (~800MB). The run stage (`alpine:3.18`) is ~5MB. By copying only the compiled binary, the final image is tiny.

**Dependency caching:** `go.mod` and `go.sum` are copied first, then `go mod download` fetches dependencies. This layer is cached and only rebuilds when dependencies change.

**Static binary:** `CGO_ENABLED=0` disables C library linking, producing a fully static binary. `GOOS=linux` ensures it targets Linux (important if building on macOS). The resulting binary has zero runtime dependencies.

**ca-certificates:** The Alpine run stage adds `ca-certificates` so the binary can make HTTPS requests. Without this, TLS connections would fail.

**COPY --from=builder:** This copies the compiled binary from the build stage. The name `builder` comes from `AS builder` in the first `FROM` instruction.

### Common Mistakes to Avoid

1. **Not using multi-stage builds.** Without them, your Go image includes the entire Go toolchain (~800MB) even though you only need the binary (~10MB).

2. **Forgetting `CGO_ENABLED=0`.** Without this, the binary links against glibc dynamically. If the run stage uses Alpine (which has musl, not glibc), the binary will not run.

3. **Not copying `go.sum`.** The `go.sum` file contains checksums for dependency integrity verification. Without it, `go mod download` may fail or behave differently.

4. **Using `FROM scratch` without ca-certificates.** `scratch` is the smallest possible image (empty), but it has no CA certificates. If your app makes HTTPS requests, they will fail. Either add `ca-certificates` or use `alpine`.

5. **Forgetting WORKDIR in the run stage.** The binary path in CMD is relative to the working directory. Without WORKDIR, it would look for `./server` in `/` (the root directory).

---

## Scenario C: Static Website with Nginx

### Complete Dockerfile

```dockerfile
FROM nginx:alpine

RUN rm /etc/nginx/conf.d/default.conf

COPY nginx.conf /etc/nginx/conf.d/
COPY dist/ /usr/share/nginx/html/

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
```

### Why This Works

**Base image:** `nginx:alpine` provides Nginx on a minimal Alpine base (~40MB). No need for a programming language runtime since this is a static site.

**Remove default config:** Nginx ships with a default configuration in `/etc/nginx/conf.d/default.conf`. Removing it first prevents conflicts with your custom config.

**Config and content:** Nginx reads config files from `/etc/nginx/conf.d/` and serves HTML from `/usr/share/nginx/html/`. These are the standard paths that do not need to be changed.

**CMD:** `nginx -g "daemon off;"` runs Nginx in the foreground. By default, Nginx daemonizes (runs in the background), which would cause the container to exit immediately since Docker expects the main process to stay in the foreground.

### Common Mistakes to Avoid

1. **Not removing the default config.** Without removing it, Nginx serves its default welcome page instead of your site, or your config conflicts with the default.

2. **Using `nginx` without `-g "daemon off;"`.** The container starts Nginx, Nginx forks to the background, and Docker sees the main process exit. The container stops immediately.

3. **Not using Alpine.** The full `nginx` image is ~180MB. `nginx:alpine` is ~40MB. For static content, there is no reason to use the larger image.

4. **Exposing the wrong port.** Nginx listens on port 80 by default. Make sure your EXPOSE and `-p` flag match. If your custom config changes the listen port, update EXPOSE accordingly.

5. **COPY dist/ to the wrong path.** Nginx serves from `/usr/share/nginx/html/` by default. Copying to a different path without updating the config will result in 404 errors.

---

## Pattern Summary

Despite different languages and ecosystems, all three Dockerfiles follow the same pattern:

```
FROM     -- choose the right base image
WORKDIR  -- set the working directory
COPY     -- dependency files first (for caching)
RUN      -- install dependencies
COPY     -- application code
RUN      -- security (non-root user)
USER     -- switch to non-root
EXPOSE   -- document ports
CMD      -- start the application
```

The only variation is Go, which uses a multi-stage build because Go compiles to a binary. Interpreted languages (Python, Node) and static content (Nginx) do not need compilation, so a single stage suffices.
