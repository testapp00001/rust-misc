# Module 07: Docker Compose

## The Problem: Your App Needs Friends

No real application runs alone. A typical web app needs:

- A backend API server
- A database (PostgreSQL, MySQL, MongoDB)
- A cache layer (Redis, Memcached)
- A reverse proxy / load balancer (Nginx, Traefik)
- Maybe a message queue (RabbitMQ, Kafka)
- Maybe a frontend (React, Vue)

In Module 06, you learned to pull images from registries. Now imagine deploying a full-stack application. You would need to:

```bash
# Create a network so containers can talk
docker network create myapp

# Start PostgreSQL
docker run -d --name postgres --network myapp \
  -e POSTGRES_PASSWORD=secret \
  -v pgdata:/var/lib/postgresql/data \
  postgres:16-alpine

# Wait for PostgreSQL to be ready... somehow

# Start Redis
docker run -d --name redis --network myapp \
  redis:7-alpine

# Start the backend API
docker run -d --name api --network myapp \
  -e DATABASE_URL=postgresql://postgres:secret@postgres:5432/mydb \
  -e REDIS_URL=redis://redis:6379 \
  -p 3000:3000 \
  myapp-api:1.0

# Start the frontend
docker run -d --name frontend --network myapp \
  -p 8080:80 \
  myapp-frontend:1.0

# Start Nginx reverse proxy
docker run -d --name nginx --network myapp \
  -p 80:80 -p 443:443 \
  -v ./nginx.conf:/etc/nginx/nginx.conf:ro \
  nginx:alpine
```

That is 5 containers, 5 `docker run` commands, a manual network setup, and you have to get the order right. Restarting everything means running `docker stop` and `docker rm` on each one. Want to see logs? Five separate `docker logs` commands.

This does not scale. One typo in a container name and the whole thing breaks. There has to be a better way.

## The Naive Way: Shell Scripts

The first instinct is to write a bash script:

**start.sh:**
```bash
#!/bin/bash
set -e

docker network create myapp 2>/dev/null || true

echo "Starting PostgreSQL..."
docker run -d --name postgres --network myapp \
  -e POSTGRES_PASSWORD=secret \
  -v pgdata:/var/lib/postgresql/data \
  postgres:16-alpine

echo "Waiting for PostgreSQL..."
sleep 5  # Hope it's ready by now...

echo "Starting Redis..."
docker run -d --name redis --network myapp redis:7-alpine

echo "Starting API..."
docker run -d --name api --network myapp \
  -e DATABASE_URL=postgresql://postgres:secret@postgres:5432/mydb \
  -e REDIS_URL=redis://redis:6379 \
  -p 3000:3000 \
  myapp-api:1.0

echo "All started!"
```

**Why this fails:**

1. **Sleep is not readiness.** `sleep 5` does not mean PostgreSQL is ready to accept connections. It might take 2 seconds or 30 seconds depending on the machine.
2. **No idempotency.** Run the script twice and `docker run --name postgres` fails because the container already exists.
3. **No cleanup.** You need a matching `stop.sh` that removes everything in the right order.
4. **No log aggregation.** You still cannot see all logs together.
5. **No build step.** The script assumes images already exist. Where is the `docker build`?
6. **Platform dependent.** Bash scripts break on Windows. Your teammate on a Mac might have different behavior.
7. **No declarative state.** The script says *how* to create containers, not *what* the desired state should be.

## The Right Way: Docker Compose

Docker Compose is a tool for defining and running multi-container applications. Instead of imperative commands ("do this, then this, then this"), you write a declarative YAML file that describes what you want:

**docker-compose.yml:**
```yaml
services:
  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_PASSWORD: secret
    volumes:
      - pgdata:/var/lib/postgresql/data

  redis:
    image: redis:7-alpine

  api:
    build: ./api
    ports:
      - "3000:3000"
    environment:
      DATABASE_URL: postgresql://postgres:secret@postgres:5432/mydb
      REDIS_URL: redis://redis:6379
    depends_on:
      - postgres
      - redis

  frontend:
    build: ./frontend
    ports:
      - "8080:80"

volumes:
  pgdata:
```

Now instead of 5 commands, you run one:

```bash
docker compose up
```

That single command will:
- Create a dedicated network for all services
- Build images that need building (`api`, `frontend`)
- Pull images that need pulling (`postgres`, `redis`)
- Start all containers in the correct order
- Show interleaved logs from all containers
- Clean everything up when you press Ctrl+C

To stop and remove everything:

```bash
docker compose down
```

To stop and also remove the data volumes:

```bash
docker compose down -v
```

## Anatomy of docker-compose.yml

### The `services` Section

Each service is a container. The service name becomes the hostname other containers use to connect:

```yaml
services:
  # This service is reachable at hostname "postgres" from other containers
  postgres:
    image: postgres:16-alpine

  # This service is reachable at hostname "redis"
  redis:
    image: redis:7-alpine
```

Inside the `api` container, `postgres` resolves to the PostgreSQL container's IP address. No need to know IP addresses. No need to link containers manually. Compose handles DNS automatically.

### Building vs. Pulling

```yaml
services:
  # Pull a pre-built image from a registry
  redis:
    image: redis:7-alpine

  # Build from a Dockerfile in a local directory
  api:
    build: ./api
    # Equivalent to: docker build -t <project>-api ./api

  # Build with options
  frontend:
    build:
      context: ./frontend
      dockerfile: Dockerfile.prod
      args:
        NODE_ENV: production
```

### Port Mapping

```yaml
services:
  api:
    ports:
      - "3000:3000"        # host:container
      - "8080:80"          # Map host 8080 to container 80
      - "127.0.0.1:3000:3000"  # Only accessible from localhost
```

### Environment Variables

There are three ways to set environment variables, from simplest to most flexible:

**Inline (hardcoded):**
```yaml
services:
  api:
    environment:
      DATABASE_URL: postgresql://postgres:secret@postgres:5432/mydb
      NODE_ENV: production
```

**From a file (recommended for development):**
```yaml
services:
  api:
    env_file:
      - .env
```

**With a `.env` file for Compose-level variables:**

Create a `.env` file in the same directory as your `docker-compose.yml`:
```
POSTGRES_PASSWORD=supersecret
API_PORT=3000
POSTGRES_VERSION=16
```

Reference these in your compose file:
```yaml
services:
  postgres:
    image: postgres:${POSTGRES_VERSION}-alpine
    environment:
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD}

  api:
    ports:
      - "${API_PORT}:3000"
```

This `.env` file is for Compose interpolation. It is different from passing environment variables into containers. The `${POSTGRES_PASSWORD}` syntax is replaced by Compose before it starts the container.

### Volumes

```yaml
services:
  postgres:
    volumes:
      # Named volume (persists across restarts)
      - pgdata:/var/lib/postgresql/data

      # Bind mount (mount a host directory)
      - ./init.sql:/docker-entrypoint-initdb.d/init.sql:ro

  api:
    volumes:
      # Mount source code for development (hot reload)
      - ./api/src:/app/src

volumes:
  pgdata:
    # driver: local  # Default driver
```

### Networks

Compose creates a default network for your project. All services can talk to each other on this network. You only need custom networks for isolation:

```yaml
services:
  postgres:
    networks:
      - backend

  redis:
    networks:
      - backend

  api:
    networks:
      - backend
      - frontend

  frontend:
    networks:
      - frontend

networks:
  backend:
    driver: bridge
  frontend:
    driver: bridge
```

Here, `postgres` and `redis` are on the `backend` network only. They cannot be reached from the `frontend` network. The `api` service bridges both networks.

## Production Way: Health Checks and Dependency Ordering

### The Problem with `depends_on`

By default, `depends_on` only waits for a container to **start**, not to be **ready**. PostgreSQL might start its process but not yet be accepting connections when your API tries to connect.

The naive fix is `sleep`. The right fix is health checks:

```yaml
services:
  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_PASSWORD: secret
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U postgres"]
      interval: 5s
      timeout: 3s
      retries: 5
      start_period: 10s

  redis:
    image: redis:7-alpine
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 5s
      timeout: 3s
      retries: 5

  api:
    build: ./api
    ports:
      - "3000:3000"
    environment:
      DATABASE_URL: postgresql://postgres:secret@postgres:5432/mydb
      REDIS_URL: redis://redis:6379
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_healthy
```

Now `api` will not start until PostgreSQL passes `pg_isready` and Redis responds to `ping`. No `sleep`. No guessing.

### Health Check Commands by Database

| Database    | Health Check Command                                      |
|-------------|-----------------------------------------------------------|
| PostgreSQL  | `pg_isready -U postgres`                                  |
| MySQL       | `mysqladmin ping -h localhost`                            |
| MongoDB     | `mongosh --eval 'db.runCommand("ping").ok' --quiet`      |
| Redis       | `redis-cli ping`                                          |
| Elasticsearch | `curl -f http://localhost:9200/_cluster/health`         |
| RabbitMQ    | `rabbitmq-diagnostics -q ping`                           |

## Scaling Services

Need 3 API instances behind a load balancer?

```yaml
services:
  api:
    build: ./api
    # No ports here — let nginx handle it

  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
    depends_on:
      - api
```

```bash
docker compose up --scale api=3
```

This starts 3 instances of the `api` service. Since each gets a unique hostname on the default network, Nginx can load-balance across them using upstream configuration:

```nginx
# nginx.conf
upstream api_backend {
    server api-1:3000;
    server api-2:3000;
    server api-3:3000;
}
```

Note: You cannot use `ports` mapping with `--scale` because only one container can bind to a given host port. Let Nginx or another load balancer handle external traffic.

## Compose Profiles

Sometimes you want optional services. A debugging tool, a monitoring stack, or a test database. Profiles let you tag services that only start when explicitly requested:

```yaml
services:
  api:
    build: ./api
    profiles: ["production"]

  postgres:
    image: postgres:16-alpine
    profiles: ["production"]

  redis:
    image: redis:7-alpine
    profiles: ["production", "dev"]

  # Debug tools — only start when you need them
  adminer:
    image: adminer
    ports:
      - "8081:8080"
    profiles: ["debug"]

  redis-commander:
    image: rediscommander/redis-commander
    ports:
      - "8082:8081"
    profiles: ["debug"]
```

```bash
# Start only production services
docker compose --profile production up

# Start production + debug tools
docker compose --profile production --profile debug up

# Start everything
docker compose --profile production --profile dev --profile debug up
```

A service with no `profiles` key is always started (default behavior).

## Override Files

Compose automatically reads two files if they exist:

- `docker-compose.yml` — The base configuration
- `docker-compose.override.yml` — Overrides for development

This is perfect for separating development and production concerns:

**docker-compose.yml (base):**
```yaml
services:
  api:
    build: ./api
    environment:
      DATABASE_URL: postgresql://postgres:secret@postgres:5432/mydb
    depends_on:
      postgres:
        condition: service_healthy

  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_PASSWORD: secret
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U postgres"]
      interval: 5s
      timeout: 3s
      retries: 5
```

**docker-compose.override.yml (development only):**
```yaml
services:
  api:
    ports:
      - "3000:3000"
    volumes:
      - ./api/src:/app/src  # Hot reload
    environment:
      NODE_ENV: development
      DEBUG: "true"

  postgres:
    ports:
      - "5432:5432"  # Expose DB to host for pgAdmin

  adminer:
    image: adminer
    ports:
      - "8081:8080"
```

**docker-compose.prod.yml (production overrides):**
```yaml
services:
  api:
    restart: always
    deploy:
      replicas: 3
      resources:
        limits:
          cpus: "0.50"
          memory: 512M

  postgres:
    restart: always
    volumes:
      - pgdata:/var/lib/postgresql/data

volumes:
  pgdata:
```

**Usage:**
```bash
# Development (reads docker-compose.yml + docker-compose.override.yml automatically)
docker compose up

# Production (reads docker-compose.yml + docker-compose.prod.yml)
docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d
```

You can create as many override files as you need:

```bash
# Staging environment
docker compose -f docker-compose.yml -f docker-compose.staging.yml up -d

# Testing with test database
docker compose -f docker-compose.yml -f docker-compose.test.yml up -d
```

## Common Commands Reference

### Lifecycle

```bash
# Start all services (foreground — see logs)
docker compose up

# Start all services (background — detached)
docker compose up -d

# Start and rebuild images
docker compose up --build

# Start specific services only
docker compose up postgres redis

# Stop all services
docker compose down

# Stop and remove volumes (DELETES DATA)
docker compose down -v

# Stop and remove volumes + images
docker compose down -v --rmi all

# Restart a specific service
docker compose restart api
```

### Logs

```bash
# View logs from all services (follow mode)
docker compose logs -f

# View logs from specific service
docker compose logs -f api

# View last 100 lines
docker compose logs --tail=100

# View logs with timestamps
docker compose logs -f -t

# View logs from multiple services
docker compose logs -f api postgres
```

### Status and Inspection

```bash
# List running services
docker compose ps

# List all services (including stopped)
docker compose ps -a

# Show resource usage
docker compose top

# See the resolved compose config (after variable substitution)
docker compose config
```

### Executing Commands in Running Containers

```bash
# Open a shell in the api container
docker compose exec api sh

# Run a one-off command
docker compose exec api npm run migrate

# Access PostgreSQL CLI
docker compose exec postgres psql -U postgres

# Access Redis CLI
docker compose exec redis redis-cli

# Run a command without allocating a TTY (for scripts)
docker compose exec -T api npm test
```

### Building

```bash
# Build all images
docker compose build

# Build with no cache
docker compose build --no-cache

# Build a specific service
docker compose build api

# Build and start
docker compose up --build
```

### One-Off Commands

```bash
# Run a one-off container (not a service)
docker compose run api npm run seed

# Run with a different command and auto-remove after
docker compose run --rm api npm run test

# Run in a different environment
docker compose run -e NODE_ENV=test api npm test
```

## Hands-On Lab: Full-Stack Application

Let us build a complete application with React frontend, Node.js API, PostgreSQL database, Redis cache, and Nginx reverse proxy.

### Directory Structure

```
fullstack-app/
  docker-compose.yml
  .env
  nginx/
    nginx.conf
  api/
    Dockerfile
    package.json
    server.js
    db/
      init.sql
  frontend/
    Dockerfile
    package.json
    src/
      App.jsx
    public/
      index.html
```

### Step 1: Create the Project

```bash
mkdir -p fullstack-app/{nginx,api/db,frontend/src,frontend/public}
cd fullstack-app
```

### Step 2: Environment File

**.env:**
```
POSTGRES_USER=appuser
POSTGRES_PASSWORD=changeme_in_production
POSTGRES_DB=fullstack
REDIS_URL=redis://redis:6379
DATABASE_URL=postgresql://appuser:changeme_in_production@postgres:5432/fullstack
API_PORT=3000
```

### Step 3: Database Initialization

**api/db/init.sql:**
```sql
CREATE TABLE IF NOT EXISTS todos (
    id SERIAL PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    completed BOOLEAN DEFAULT false,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO todos (title, completed) VALUES
    ('Learn Docker Compose', false),
    ('Build a full-stack app', false),
    ('Deploy to production', false);
```

### Step 4: The API

**api/package.json:**
```json
{
  "name": "fullstack-api",
  "version": "1.0.0",
  "scripts": {
    "start": "node server.js"
  },
  "dependencies": {
    "express": "^4.18.2",
    "pg": "^8.11.3",
    "redis": "^4.6.12",
    "cors": "^2.8.5"
  }
}
```

**api/server.js:**
```javascript
const express = require('express');
const { Pool } = require('pg');
const redis = require('redis');
const cors = require('cors');

const app = express();
app.use(cors());
app.use(express.json());

// PostgreSQL connection
const pool = new Pool({
  connectionString: process.env.DATABASE_URL,
});

// Redis connection
const redisClient = redis.createClient({
  url: process.env.REDIS_URL,
});

redisClient.on('error', (err) => console.log('Redis error:', err));

// Connect to Redis on startup
(async () => {
  await redisClient.connect();
  console.log('Connected to Redis');
})();

// Health check endpoint
app.get('/health', async (req, res) => {
  try {
    await pool.query('SELECT 1');
    await redisClient.ping();
    res.json({ status: 'healthy', postgres: 'ok', redis: 'ok' });
  } catch (err) {
    res.status(503).json({ status: 'unhealthy', error: err.message });
  }
});

// Get all todos (with Redis cache)
app.get('/api/todos', async (req, res) => {
  try {
    // Check cache first
    const cached = await redisClient.get('todos');
    if (cached) {
      return res.json({ source: 'cache', data: JSON.parse(cached) });
    }

    // Query database
    const result = await pool.query('SELECT * FROM todos ORDER BY created_at DESC');

    // Store in cache for 60 seconds
    await redisClient.setEx('todos', 60, JSON.stringify(result.rows));

    res.json({ source: 'database', data: result.rows });
  } catch (err) {
    res.status(500).json({ error: err.message });
  }
});

// Create a todo
app.post('/api/todos', async (req, res) => {
  try {
    const { title } = req.body;
    const result = await pool.query(
      'INSERT INTO todos (title) VALUES ($1) RETURNING *',
      [title]
    );

    // Invalidate cache
    await redisClient.del('todos');

    res.status(201).json(result.rows[0]);
  } catch (err) {
    res.status(500).json({ error: err.message });
  }
});

// Toggle todo completion
app.patch('/api/todos/:id', async (req, res) => {
  try {
    const { id } = req.params;
    const result = await pool.query(
      'UPDATE todos SET completed = NOT completed WHERE id = $1 RETURNING *',
      [id]
    );

    // Invalidate cache
    await redisClient.del('todos');

    res.json(result.rows[0]);
  } catch (err) {
    res.status(500).json({ error: err.message });
  }
});

const PORT = process.env.PORT || 3000;
app.listen(PORT, '0.0.0.0', () => {
  console.log(`API running on port ${PORT}`);
});
```

**api/Dockerfile:**
```dockerfile
FROM node:20-alpine

WORKDIR /app

COPY package.json package-lock.json* ./
RUN npm install

COPY . .

# Add curl for health check
RUN apk add --no-cache curl

EXPOSE 3000

HEALTHCHECK --interval=10s --timeout=5s --retries=3 \
  CMD curl -f http://localhost:3000/health || exit 1

CMD ["node", "server.js"]
```

### Step 5: The Frontend

**frontend/public/index.html:**
```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Full-Stack Todos</title>
</head>
<body>
  <div id="root"></div>
</body>
</html>
```

**frontend/src/App.jsx:**
```jsx
import React, { useState, useEffect } from 'react';

function App() {
  const [todos, setTodos] = useState([]);
  const [newTodo, setNewTodo] = useState('');
  const [source, setSource] = useState('');

  const fetchTodos = async () => {
    const res = await fetch('/api/todos');
    const data = await res.json();
    setTodos(data.data);
    setSource(data.source);
  };

  useEffect(() => { fetchTodos(); }, []);

  const addTodo = async (e) => {
    e.preventDefault();
    if (!newTodo.trim()) return;
    await fetch('/api/todos', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ title: newTodo }),
    });
    setNewTodo('');
    fetchTodos();
  };

  const toggleTodo = async (id) => {
    await fetch(`/api/todos/${id}`, { method: 'PATCH' });
    fetchTodos();
  };

  return (
    <div style={{ maxWidth: 600, margin: '40px auto', fontFamily: 'sans-serif' }}>
      <h1>Todo List</h1>
      <p><small>Data source: {source}</small></p>
      <form onSubmit={addTodo}>
        <input
          value={newTodo}
          onChange={(e) => setNewTodo(e.target.value)}
          placeholder="Add a todo..."
          style={{ padding: 8, width: '70%' }}
        />
        <button type="submit" style={{ padding: 8 }}>Add</button>
      </form>
      <ul>
        {todos.map((todo) => (
          <li key={todo.id} style={{ margin: '8px 0' }}>
            <span
              onClick={() => toggleTodo(todo.id)}
              style={{
                textDecoration: todo.completed ? 'line-through' : 'none',
                cursor: 'pointer',
              }}
            >
              {todo.title}
            </span>
          </li>
        ))}
      </ul>
    </div>
  );
}

export default App;
```

**frontend/package.json:**
```json
{
  "name": "fullstack-frontend",
  "version": "1.0.0",
  "scripts": {
    "start": "react-scripts start",
    "build": "react-scripts build"
  },
  "dependencies": {
    "react": "^18.2.0",
    "react-dom": "^18.2.0",
    "react-scripts": "5.0.1"
  }
}
```

**frontend/Dockerfile:**
```dockerfile
FROM node:20-alpine

WORKDIR /app

COPY package.json package-lock.json* ./
RUN npm install

COPY . .

EXPOSE 3000

CMD ["npm", "start"]
```

### Step 6: Nginx Configuration

**nginx/nginx.conf:**
```nginx
events {
    worker_connections 1024;
}

http {
    upstream frontend {
        server frontend:3000;
    }

    upstream api {
        server api:3000;
    }

    server {
        listen 80;

        # Serve frontend
        location / {
            proxy_pass http://frontend;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
        }

        # Proxy API requests
        location /api/ {
            proxy_pass http://api;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
        }

        # Health check
        location /nginx-health {
            return 200 'ok';
            add_header Content-Type text/plain;
        }
    }
}
```

### Step 7: The Compose File

**docker-compose.yml:**
```yaml
services:
  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_USER: ${POSTGRES_USER}
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD}
      POSTGRES_DB: ${POSTGRES_DB}
    volumes:
      - pgdata:/var/lib/postgresql/data
      - ./api/db/init.sql:/docker-entrypoint-initdb.d/init.sql:ro
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U ${POSTGRES_USER} -d ${POSTGRES_DB}"]
      interval: 5s
      timeout: 3s
      retries: 5
      start_period: 10s

  redis:
    image: redis:7-alpine
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 5s
      timeout: 3s
      retries: 5

  api:
    build: ./api
    environment:
      DATABASE_URL: ${DATABASE_URL}
      REDIS_URL: ${REDIS_URL}
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_healthy
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
      interval: 10s
      timeout: 5s
      retries: 3

  frontend:
    build: ./frontend
    environment:
      REACT_APP_API_URL: http://api:3000
    depends_on:
      api:
        condition: service_healthy

  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
    volumes:
      - ./nginx/nginx.conf:/etc/nginx/nginx.conf:ro
    depends_on:
      - api
      - frontend

volumes:
  pgdata:
```

### Step 8: Run It

```bash
# Start everything
docker compose up

# Or run in detached mode
docker compose up -d

# Watch the logs
docker compose logs -f

# Check that everything is healthy
docker compose ps
```

Visit `http://localhost` in your browser. You should see the Todo list app. The data flows:

```
Browser -> Nginx (port 80) -> React Frontend
Browser -> Nginx (port 80) /api/* -> Node.js API -> PostgreSQL + Redis
```

### Step 9: Experiment

```bash
# Access PostgreSQL directly
docker compose exec postgres psql -U appuser -d fullstack -c "SELECT * FROM todos;"

# Access Redis directly
docker compose exec redis redis-cli GET todos

# Watch API logs only
docker compose logs -f api

# Rebuild just the API after a code change
docker compose up --build api

# Scale the API to 3 instances (remove port mapping from api first)
docker compose up --scale api=3

# Stop everything and clean up
docker compose down

# Stop and delete all data
docker compose down -v
```

## Compose File Best Practices

### 1. Always Set Health Checks

```yaml
# BAD — depends_on only waits for container to start
depends_on:
  - postgres

# GOOD — waits for the service to actually be ready
depends_on:
  postgres:
    condition: service_healthy
```

### 2. Use `.env` for Secrets in Development, Never Commit It

```gitignore
# .gitignore
.env
docker-compose.override.yml
```

### 3. Pin Image Versions

```yaml
# BAD — could break any day
image: postgres:latest

# GOOD — predictable
image: postgres:16.2-alpine
```

### 4. Use Named Volumes for Persistent Data

```yaml
# BAD — anonymous volume, hard to manage
volumes:
  - /var/lib/postgresql/data

# GOOD — named volume, survives `docker compose down`
volumes:
  - pgdata:/var/lib/postgresql/data
```

### 5. Use `restart` Policies

```yaml
services:
  api:
    restart: unless-stopped
    # Options: no, always, on-failure, unless-stopped
```

### 6. Set Resource Limits in Production

```yaml
services:
  api:
    deploy:
      resources:
        limits:
          cpus: "1.0"
          memory: 512M
        reservations:
          cpus: "0.25"
          memory: 128M
```

### 7. Use `.dockerignore` in Each Build Context

```
# api/.dockerignore
node_modules
npm-debug.log
.env
.git
```

## Compose Version History

You might see older tutorials with a `version` key at the top:

```yaml
version: "3.8"  # OLD — no longer needed
services:
  ...
```

As of Docker Compose V2 (the current standard), the `version` key is ignored and optional. Do not include it in new files. Docker Compose V2 is invoked as `docker compose` (a space), while the legacy V1 used `docker-compose` (a hyphen).

```bash
# V2 (current — use this)
docker compose up

# V1 (deprecated)
docker-compose up
```

## Troubleshooting

### Container exits immediately

```bash
# Check the logs
docker compose logs <service-name>

# Check exit code
docker compose ps -a
```

### "Port already in use"

```bash
# Find what's using the port
lsof -i :80

# Change the host port in docker-compose.yml
ports:
  - "8080:80"  # Use 8080 on the host instead
```

### Service cannot connect to another service

```bash
# Verify the service is running
docker compose ps

# Test connectivity from inside a container
docker compose exec api sh
# Inside the container:
ping postgres
curl http://redis:6379
```

### Changes not reflected after `docker compose up`

```bash
# Rebuild images
docker compose up --build

# Nuclear option — rebuild everything from scratch
docker compose down -v
docker compose build --no-cache
docker compose up
```

## Checklist

- [ ] I can write a `docker-compose.yml` from scratch
- [ ] I understand services, networks, and volumes in Compose
- [ ] I use `depends_on` with health checks, not `sleep`
- [ ] I know how to use `.env` files for variable interpolation
- [ ] I can use override files for different environments
- [ ] I know the common commands: `up`, `down`, `logs`, `exec`, `build`
- [ ] I can scale services with `--scale`
- [ ] I understand Compose profiles for optional services

## Limitation: How Do Containers Find Each Other?

Compose automatically creates a network and DNS entries so that `postgres` resolves to the PostgreSQL container. But how does that DNS actually work? What happens when containers are on different networks? What happens when you need to expose a service to the host but also keep it accessible to other containers? What is the difference between a bridge network and a host network?

Compose gives you networking for free. But when things break -- when containers cannot reach each other, when ports conflict, when you need to isolate services -- you need to understand what is happening under the hood.

**Next problem:** How does container networking actually work?

**Next module:** [08-container-networking](../08-container-networking/) -- Bridge, host, overlay networks, DNS resolution, and port mapping internals
