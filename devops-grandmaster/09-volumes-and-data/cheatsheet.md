# Cheatsheet: Volumes & Data

## Volume Types

| Type | Description | Use Case |
|------|-------------|----------|
| Named volume | Managed by Docker | Production data |
| Bind mount | Host directory | Development |
| tmpfs | In-memory only | Temporary sensitive data |

## Common Commands
```bash
docker volume create my-volume          # Create volume
docker volume ls                        # List volumes
docker volume inspect my-volume         # Inspect volume
docker volume rm my-volume              # Remove volume
docker volume prune                     # Remove unused
```

## Usage
```bash
# Named volume
docker run -v my-data:/app/data postgres

# Bind mount
docker run -v /host/path:/container/path my-app

# Read-only mount
docker run -v my-data:/app/data:ro my-app

# tmpfs (in-memory)
docker run --tmpfs /app/temp my-app
```

## Docker Compose
```yaml
services:
  db:
    image: postgres
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./init.sql:/docker-entrypoint-initdb.d/init.sql:ro

volumes:
  postgres_data:
```

## Backup & Restore
```bash
# Backup volume
docker run --rm -v my-data:/source -v $(pwd):/backup alpine \
  tar czf /backup/my-data-backup.tar.gz -C /source .

# Restore volume
docker run --rm -v my-data:/target -v $(pwd):/backup alpine \
  tar xzf /backup/my-data-backup.tar.gz -C /target
```
