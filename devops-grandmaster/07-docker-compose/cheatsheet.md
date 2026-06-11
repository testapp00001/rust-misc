# Cheatsheet: Docker Compose

## Essential Commands
```bash
docker compose up -d              # Start all services (detached)
docker compose down               # Stop and remove all
docker compose down -v            # Also remove volumes
docker compose ps                 # List running services
docker compose logs               # View all logs
docker compose logs -f service    # Follow specific service
docker compose exec service bash  # Shell into service
docker compose build              # Build images
docker compose build --no-cache   # Build without cache
docker compose pull               # Pull latest images
docker compose restart            # Restart all services
docker compose restart service    # Restart specific service
docker compose up -d --scale api=3  # Scale service
```

## docker-compose.yml Structure
```yaml
version: '3.8'

services:
  web:
    image: nginx:alpine
    ports:
      - "80:80"
    volumes:
      - ./html:/usr/share/nginx/html
    environment:
      - NODE_ENV=production
    env_file:
      - .env
    depends_on:
      db:
        condition: service_healthy
    networks:
      - frontend
    restart: unless-stopped

  db:
    image: postgres:15
    environment:
      POSTGRES_PASSWORD: secret
    volumes:
      - postgres_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U postgres"]
      interval: 10s
      timeout: 5s
      retries: 5
    networks:
      - backend

volumes:
  postgres_data:

networks:
  frontend:
  backend:
```

## Key Concepts

| Concept | Purpose |
|---------|---------|
| services | Container definitions |
| volumes | Persistent data storage |
| networks | Container communication |
| depends_on | Startup order |
| healthcheck | Container health verification |
| restart | Restart policy |
| env_file | Load environment from file |
| build | Build from Dockerfile |
