# Cheatsheet: Dockerfile Instructions

## Instructions

| Instruction | Purpose | Example |
|------------|---------|---------|
| `FROM` | Base image | `FROM python:3.11-slim` |
| `WORKDIR` | Set working directory | `WORKDIR /app` |
| `COPY` | Copy files from host | `COPY . .` |
| `RUN` | Execute during build | `RUN pip install -r req.txt` |
| `CMD` | Default command | `CMD ["python", "app.py"]` |
| `ENTRYPOINT` | Main executable | `ENTRYPOINT ["python"]` |
| `ENV` | Environment variable | `ENV PORT=8080` |
| `ARG` | Build-time variable | `ARG VERSION=1.0` |
| `EXPOSE` | Document port | `EXPOSE 8080` |
| `USER` | Run as user | `USER appuser` |
| `HEALTHCHECK` | Health check | See below |

## Best Practices

```dockerfile
# 1. Use specific tags
FROM python:3.11.4-slim-bookworm  # Good
FROM python:latest                 # Bad

# 2. Order for caching (least changing first)
COPY requirements.txt .     # Dependencies (changes rarely)
RUN pip install -r requirements.txt
COPY . .                     # Code (changes often)

# 3. Minimize layers
RUN apt-get update && \
    apt-get install -y curl && \
    rm -rf /var/lib/apt/lists/*

# 4. Run as non-root
RUN adduser --disabled-password --gecos '' appuser
USER appuser

# 5. Use .dockerignore
# .git, node_modules, __pycache__, *.md, .env
```

## Health Check
```dockerfile
HEALTHCHECK --interval=30s --timeout=5s --retries=3 --start-period=10s \
  CMD curl -f http://localhost:8080/health || exit 1
```

## Build Command
```bash
docker build -t myapp:1.0 .
docker build -f Dockerfile.prod -t myapp:1.0 .
docker build --build-arg VERSION=1.0 -t myapp:1.0 .
docker build --no-cache -t myapp:1.0 .
```
