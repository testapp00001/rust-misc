# Exercise 03: Multi-Stage Dockerfiles for Three Languages

**Type:** Independent
**Time:** 45 minutes
**Difficulty:** Medium

## Objective

Write multi-stage Dockerfiles from scratch for three different languages:
Go, Python, and Node.js. Each language has different build characteristics,
and the multi-stage pattern adapts accordingly. This exercise builds your
ability to apply the pattern independently.

## Setup

Create a working directory:

```bash
mkdir -p /tmp/exercise-03/{go-app,python-app,node-app}
```

---

## Part A: Go Application

### Application Code

Create `/tmp/exercise-03/go-app/main.go`:

```go
package main

import (
	"fmt"
	"net/http"
	"os"
)

func main() {
	port := os.Getenv("PORT")
	if port == "" {
		port = "8080"
	}
	http.HandleFunc("/", func(w http.ResponseWriter, r *http.Request) {
		hostname, _ := os.Hostname()
		fmt.Fprintf(w, "Hello from Go! Hostname: %s\n", hostname)
	})
	fmt.Printf("Server starting on :%s\n", port)
	http.ListenAndServe(":"+port, nil)
}
```

Create `/tmp/exercise-03/go-app/go.mod`:

```
module go-app

go 1.22
```

### Your Task

Write a multi-stage Dockerfile at `/tmp/exercise-03/go-app/Dockerfile` that:
- Builds a static binary with `CGO_ENABLED=0`
- Strips debug symbols with `-ldflags="-s -w"`
- Uses `FROM scratch` as the runtime image
- Copies CA certificates for HTTPS support
- Produces an image under 15MB

<details>
<summary>Hint 1: Builder Stage</summary>

Go compiles to a single binary. The builder needs `golang:1.22` and
your source code. Use `CGO_ENABLED=0` to produce a fully static binary
that does not depend on libc.

```dockerfile
FROM golang:1.22 AS builder
WORKDIR /app
COPY go.mod ./
COPY *.go ./
RUN CGO_ENABLED=0 GOOS=linux go build -ldflags="-s -w" -o /server .
```

</details>

<details>
<summary>Hint 2: Runtime Stage</summary>

`FROM scratch` is an empty image. You need to copy:
1. The binary from the builder
2. CA certificates (for any HTTPS calls your app makes)

```dockerfile
FROM scratch
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=builder /server /server
EXPOSE 8080
ENTRYPOINT ["/server"]
```

</details>

### Verify

```bash
cd /tmp/exercise-03/go-app
docker build -t exercise3-go .
docker images exercise3-go
# Expected: under 15MB
docker run -p 8080:8080 exercise3-go &
curl http://localhost:8080
```

---

## Part B: Python Application

### Application Code

Create `/tmp/exercise-03/python-app/requirements.txt`:

```
flask==3.0.3
gunicorn==22.0.0
```

Create `/tmp/exercise-03/python-app/app.py`:

```python
import os
from flask import Flask

app = Flask(__name__)

@app.route("/")
def hello():
    return f"Hello from Python! Hostname: {os.uname().nodename}\n"

@app.route("/health")
def health():
    return {"status": "ok"}

if __name__ == "__main__":
    app.run(host="0.0.0.0", port=5000)
```

### Your Task

Write a multi-stage Dockerfile at `/tmp/exercise-03/python-app/Dockerfile` that:
- Uses `python:3.12` in the builder stage to install packages
- Uses `python:3.12-slim` as the runtime base image
- Installs packages with `--prefix=/install` in the builder
- Copies installed packages to the runtime stage
- Runs as a non-root user
- Uses gunicorn as the production server (not Flask's dev server)

<details>
<summary>Hint 1: Builder Stage</summary>

Use `--prefix=/install` to install packages into a separate directory.
This makes it easy to copy them to the runtime stage without copying
the entire Python installation.

```dockerfile
FROM python:3.12 AS builder
WORKDIR /app
COPY requirements.txt .
RUN pip install --no-cache-dir --prefix=/install -r requirements.txt
```

</details>

<details>
<summary>Hint 2: Runtime Stage</summary>

Copy the installed packages from `/install` in the builder to `/usr/local`
in the runtime image. This puts them on the Python path automatically.

```dockerfile
FROM python:3.12-slim
WORKDIR /app
COPY --from=builder /install /usr/local
COPY . .
RUN useradd --create-home appuser
USER appuser
CMD ["gunicorn", "--bind", "0.0.0.0:5000", "app:app"]
```

</details>

### Verify

```bash
cd /tmp/exercise-03/python-app
docker build -t exercise3-python .
docker images exercise3-python
# Expected: under 200MB
docker run -p 5000:5000 exercise3-python &
curl http://localhost:5000
curl http://localhost:5000/health
```

---

## Part C: Node.js Application

### Application Code

Create `/tmp/exercise-03/node-app/package.json`:

```json
{
  "name": "exercise3-node",
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

Create `/tmp/exercise-03/node-app/tsconfig.json`:

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

Create `/tmp/exercise-03/node-app/src/index.ts`:

```typescript
import express from "express";
import os from "os";

const app = express();
const PORT = process.env.PORT || 3000;

app.get("/", (_req, res) => {
  res.send(`Hello from Node.js! Hostname: ${os.hostname()}\n`);
});

app.get("/health", (_req, res) => {
  res.json({ status: "ok" });
});

app.listen(PORT, () => {
  console.log(`Server running on port ${PORT}`);
});
```

### Your Task

Write a multi-stage Dockerfile at `/tmp/exercise-03/node-app/Dockerfile` that:
- Uses `node:20-alpine` in the builder stage
- Installs ALL dependencies (including devDependencies)
- Compiles TypeScript to JavaScript
- Prunes devDependencies after building
- Uses `node:20-alpine` as the runtime base
- Copies only `dist/`, `node_modules/`, and `package.json`
- Runs as the `node` user (built into the Alpine image)

<details>
<summary>Hint 1: Builder Stage</summary>

Install all dependencies first (TypeScript is a devDependency), then
build, then prune.

```dockerfile
FROM node:20-alpine AS builder
WORKDIR /app
COPY package.json package-lock.json* ./
RUN npm ci
COPY tsconfig.json ./
COPY src ./src
RUN npm run build
RUN npm prune --production
```

</details>

<details>
<summary>Hint 2: Runtime Stage</summary>

Only copy the compiled output and production dependencies.

```dockerfile
FROM node:20-alpine
WORKDIR /app
COPY --from=builder /app/node_modules ./node_modules
COPY --from=builder /app/dist ./dist
COPY --from=builder /app/package.json ./
USER node
CMD ["node", "dist/index.js"]
```

</details>

### Verify

```bash
cd /tmp/exercise-03/node-app
docker build -t exercise3-node .
docker images exercise3-node
# Expected: under 150MB
docker run -p 3000:3000 exercise3-node &
curl http://localhost:3000
```

---

## Final Comparison

After building all three, run:

```bash
docker images | grep exercise3
```

Record the sizes:

| Language | Image Size | Runtime Base |
|---|---|---|
| Go | ??? MB | scratch |
| Python | ??? MB | python:3.12-slim |
| Node.js | ??? MB | node:20-alpine |

## Success Criteria

- [ ] Go image is under 15MB using `FROM scratch`
- [ ] Python image is under 200MB using `python:3.12-slim`
- [ ] Node.js image is under 150MB using `node:20-alpine`
- [ ] All three applications respond to HTTP requests
- [ ] All three Dockerfiles use named stages (`AS builder`)
- [ ] The Node.js Dockerfile prunes devDependencies
- [ ] The Python Dockerfile runs as a non-root user

## What You Should Understand After This Exercise

Each language has its own multi-stage pattern. Go and Rust compile to
static binaries and can use `FROM scratch`. Python needs a runtime
interpreter, so you use slim images and copy installed packages. Node.js
needs to separate dev dependencies from production dependencies. The
principle is the same everywhere: build in a fat image, run in a thin one.
