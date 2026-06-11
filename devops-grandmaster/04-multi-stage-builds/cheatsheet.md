# Cheatsheet: Multi-Stage Builds

## Core Concept

```dockerfile
# Stage 1: Build
FROM golang:1.22 AS builder
WORKDIR /app
COPY go.mod go.sum ./
RUN go mod download
COPY . .
RUN CGO_ENABLED=0 go build -o /server .

# Stage 2: Production
FROM gcr.io/distroless/static
COPY --from=builder /server /server
ENTRYPOINT ["/server"]
```

## Pattern: Language-Specific

### Python
```dockerfile
FROM python:3.12-slim AS builder
WORKDIR /app
COPY requirements.txt .
RUN pip install --user --no-cache-dir -r requirements.txt
COPY . .

FROM python:3.12-slim
COPY --from=builder /root/.local /root/.local
COPY --from=builder /app /app
ENV PATH=/root/.local/bin:$PATH
CMD ["python", "app.py"]
```

### Node.js
```dockerfile
FROM node:20-alpine AS builder
WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
RUN npm run build

FROM node:20-alpine
WORKDIR /app
COPY --from=builder /app/dist ./dist
COPY --from=builder /app/node_modules ./node_modules
CMD ["node", "dist/index.js"]
```

### Rust
```dockerfile
FROM rust:1.77 AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main(){}" > src/main.rs && cargo build --release && rm -rf src
COPY . .
RUN touch src/main.rs && cargo build --release

FROM gcr.io/distroless/static
COPY --from=builder /app/target/release/myapp /myapp
ENTRYPOINT ["/myapp"]
```

## Key Commands

```bash
# Build specific stage
docker build --target builder -t myapp:dev .

# Build final image only
docker build -t myapp:prod .

# Use external image as source
COPY --from=nginx:alpine /etc/nginx/nginx.conf /etc/nginx/
```

## Size Comparison

| Approach | Python App | Node App | Go App |
|----------|-----------|----------|--------|
| Single-stage | ~900MB | ~1.1GB | ~1.2GB |
| Multi-stage | ~120MB | ~180MB | ~12MB |
| Multi-stage + distroless | ~80MB | ~150MB | ~8MB |

## Common Mistakes

```dockerfile
# BAD: Copying everything too early
COPY . .                    # Cache busted on every code change
RUN pip install -r requirements.txt

# GOOD: Copy deps first
COPY requirements.txt .     # Only changes when deps change
RUN pip install -r requirements.txt
COPY . .                    # Code changes don't rebuild deps
```
