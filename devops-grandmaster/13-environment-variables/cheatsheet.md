# Cheatsheet: Environment Variables

## Docker
```bash
# Inline
docker run -e APP_ENV=production -e PORT=8080 my-app

# From file
docker run --env-file .env my-app

# Dockerfile default
ENV APP_ENV=production
ENV PORT=8080
```

## Docker Compose
```yaml
services:
  web:
    image: my-app
    environment:
      - APP_ENV=production
      - PORT=8080
    env_file:
      - .env
```

## .env File
```bash
APP_ENV=production
PORT=8080
DATABASE_URL=postgresql://user:pass@db:5432/myapp
REDIS_URL=redis://redis:6379
```

## 12-Factor App Config
```
1. Store config in environment variables
2. Never bake config into code
3. Separate config per environment
4. Treat config as code (version control)
```

## Config Hierarchy (later overrides earlier)
```
Dockerfile ENV → .env file → docker-compose environment → docker run -e
```

## Security
```bash
# NEVER do this:
ENV DATABASE_PASSWORD=secret123

# DO this instead:
# Use secrets management (Module 14)
```
