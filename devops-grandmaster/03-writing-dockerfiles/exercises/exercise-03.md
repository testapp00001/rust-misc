# Exercise 03: Dockerfiles for Every Language

**Type:** Independent
**Time:** 45 minutes
**Difficulty:** Medium

## Objective

Write Dockerfiles for three different programming languages from scratch.
Each language has its own ecosystem, dependency management, and conventions.
This exercise teaches you to apply Dockerfile patterns to any language.

## Instructions

For each of the three scenarios below, write a complete Dockerfile that
follows best practices. Each Dockerfile must:

- Use a specific base image tag (no `latest`)
- Set a working directory
- Install dependencies separately from code (for caching)
- Run as a non-root user
- Expose the correct port
- Use the exec form for CMD

### Scenario A: Node.js Express API

You have a Node.js application with the following files:

**server.js:**
```javascript
const express = require('express');
const app = express();
const PORT = process.env.PORT || 3000;

app.get('/', (req, res) => {
    res.json({ message: 'Node.js API running', status: 'ok' });
});

app.get('/health', (req, res) => {
    res.json({ status: 'healthy' });
});

app.listen(PORT, () => {
    console.log(`Server running on port ${PORT}`);
});
```

**package.json:**
```json
{
    "name": "my-node-api",
    "version": "1.0.0",
    "dependencies": {
        "express": "^4.18.2"
    }
}
```

**package-lock.json:** (exists, assume it is present)

Write a Dockerfile that:
- Uses `node:18-alpine` as the base image
- Copies dependency files first, then runs `npm ci --only=production`
- Uses a non-root user (Alpine uses `adduser` with `-D` flag)
- Exposes port 3000
- Runs `node server.js`

<details>
<summary>Hint 1: Dependency Caching</summary>

For Node.js, copy `package.json` and `package-lock.json` first, then run
`npm ci`. This way the dependency install layer is cached and only re-runs
when those files change.

```dockerfile
COPY package.json package-lock.json ./
RUN npm ci --only=production
```

</details>

<details>
<summary>Hint 2: Non-Root User on Alpine</summary>

Alpine Linux uses BusyBox `adduser`, which has different syntax than Debian:

```dockerfile
RUN addgroup -g 1001 appgroup && \
    adduser -u 1001 -G appgroup -s /bin/sh -D appuser
USER appuser
```

The `-D` flag disables password prompting (equivalent to `--disabled-password`
on Debian).

</details>

---

### Scenario B: Go HTTP Server

You have a Go application with the following files:

**main.go:**
```go
package main

import (
    "encoding/json"
    "log"
    "net/http"
)

type Response struct {
    Message string `json:"message"`
    Status  string `json:"status"`
}

func handler(w http.ResponseWriter, r *http.Request) {
    w.Header().Set("Content-Type", "application/json")
    json.NewEncoder(w).Encode(Response{Message: "Go server running", Status: "ok"})
}

func healthHandler(w http.ResponseWriter, r *http.Request) {
    w.Header().Set("Content-Type", "application/json")
    json.NewEncoder(w).Encode(Response{Status: "healthy"})
}

func main() {
    http.HandleFunc("/", handler)
    http.HandleFunc("/health", healthHandler)
    log.Println("Server starting on :8080")
    log.Fatal(http.ListenAndServe(":8080", nil))
}
```

**go.mod:**
```
module my-go-server

go 1.21
```

**go.sum:** (exists, assume it is present)

Write a Dockerfile that:
- Uses a multi-stage build: `golang:1.21-alpine` for building, `alpine:3.18` for running
- Copies `go.mod` and `go.sum` first, then runs `go mod download`
- Builds a statically linked binary with `CGO_ENABLED=0`
- Copies only the compiled binary to the final image
- Exposes port 8080

<details>
<summary>Hint 1: Multi-Stage Build Structure</summary>

A multi-stage build uses multiple `FROM` instructions. The first stage
compiles the code; the second stage copies only the binary:

```dockerfile
FROM golang:1.21-alpine AS builder
# ... build steps ...

FROM alpine:3.18
# ... copy binary from builder ...
```

</details>

<details>
<summary>Hint 2: Static Binary</summary>

Go can compile to a static binary that has no runtime dependencies:

```dockerfile
RUN CGO_ENABLED=0 GOOS=linux go build -o server .
```

`CGO_ENABLED=0` disables C library linking, producing a fully static binary
that runs on the minimal `alpine` image without needing libc.

</details>

<details>
<summary>Hint 3: Copying from Builder</summary>

Use `COPY --from=builder` to copy files from the build stage:

```dockerfile
COPY --from=builder /app/server .
```

The `builder` name comes from `AS builder` in the first `FROM` instruction.

</details>

---

### Scenario C: Static Website with Nginx

You have a static website built with any framework (React, Vue, plain HTML).
The build output is in a `dist/` directory.

**dist/index.html:**
```html
<!DOCTYPE html>
<html>
<head><title>My Site</title></head>
<body><h1>Hello from Nginx!</h1></body>
</html>
```

**nginx.conf:**
```nginx
server {
    listen 80;
    server_name localhost;

    location / {
        root /usr/share/nginx/html;
        index index.html;
        try_files $uri $uri/ /index.html;
    }

    location /health {
        return 200 '{"status":"healthy"}';
        add_header Content-Type application/json;
    }
}
```

Write a Dockerfile that:
- Uses `nginx:alpine` as the base image
- Removes the default Nginx config
- Copies your custom `nginx.conf` to `/etc/nginx/conf.d/`
- Copies the `dist/` directory to the Nginx html root
- Exposes port 80
- Uses `CMD ["nginx", "-g", "daemon off;"]` to run in the foreground

<details>
<summary>Hint</summary>

Nginx expects config files in `/etc/nginx/conf.d/` and serves HTML from
`/usr/share/nginx/html/`. Remove the default config first to avoid conflicts:

```dockerfile
RUN rm /etc/nginx/conf.d/default.conf
COPY nginx.conf /etc/nginx/conf.d/
COPY dist/ /usr/share/nginx/html/
```

</details>

## Success Criteria

- [ ] You have written three Dockerfiles, one per scenario
- [ ] Each Dockerfile uses a specific base image tag
- [ ] Each Dockerfile follows the dependency-before-code caching pattern
- [ ] The Go Dockerfile uses a multi-stage build
- [ ] The Nginx Dockerfile uses a custom config
- [ ] Each Dockerfile builds without errors (test with `docker build`)
- [ ] Each container runs and responds to requests

## What You Should Understand After This Exercise

Different languages have different conventions, but the Dockerfile pattern
is the same: base image, working directory, dependencies, code, security,
command. Multi-stage builds are essential for compiled languages like Go
because the build tools do not need to be in the final image.
