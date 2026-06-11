# Solution 02: Harden a Dockerfile

## Part A: Hardened Dockerfile

```dockerfile
FROM node:18-slim

# Create non-root user
RUN groupadd --gid 1001 appgroup && \
    useradd --uid 1001 --gid appgroup --shell /bin/sh --create-home appuser

WORKDIR /app

# Copy dependency files first (layer caching)
COPY --chown=appuser:appgroup package*.json ./

# Install dependencies
RUN npm ci --production && \
    # Remove npm cache to reduce image size
    npm cache clean --force

# Copy application code
COPY --chown=appuser:appgroup . .

# Switch to non-root user
USER appuser

EXPOSE 3000

HEALTHCHECK --interval=30s --timeout=5s --retries=3 \
  CMD ["node", "-e", "require('http').get('http://localhost:3000/health', (r) => { process.exit(r.statusCode === 200 ? 0 : 1) })"]

CMD ["node", "server.js"]
```

**Why this works:**

- **`node:18-slim`** is based on Debian but has far fewer installed packages than the full `node:18` image. Fewer packages means fewer potential vulnerabilities and a smaller attack surface.
- **`groupadd` and `usercreate`** create a dedicated user with a known UID (1001). Using a numeric UID avoids dependency on `/etc/passwd` entries and works consistently across base images.
- **`COPY --chown=appuser:appgroup`** ensures the application files are owned by the non-root user, so the process can read them without running as root.
- **`COPY package*.json` before `COPY .`** means that `npm ci` only re-runs when dependencies change, not on every code change. This is a caching optimization, not a security measure, but it is a Dockerfile best practice.
- **`USER appuser` before `CMD`** ensures the process starts as the non-root user. If `USER` is placed after `CMD`, the container would still run as root.
- **`HEALTHCHECK`** allows Docker and orchestrators to detect when the application is unhealthy and restart it.

---

## Part B: Hardened docker-compose.yml

```yaml
version: "3.8"

services:
  app:
    build: .
    image: my-app:latest
    user: "1001:1001"
    read_only: true
    security_opt:
      - no-new-privileges:true
    cap_drop:
      - ALL
    tmpfs:
      - /tmp:size=100M,noexec,nosuid
    networks:
      - frontend
      - backend
    ports:
      - "3000:3000"
    secrets:
      - db_url
      - api_key
    environment:
      DATABASE_URL_FILE: /run/secrets/db_url
      API_KEY_FILE: /run/secrets/api_key
    deploy:
      resources:
        limits:
          memory: 512M
          cpus: '0.5'
    healthcheck:
      test: ["CMD", "node", "-e", "require('http').get('http://localhost:3000/health', (r) => { process.exit(r.statusCode === 200 ? 0 : 1) })"]
      interval: 30s
      timeout: 5s
      retries: 3
    logging:
      driver: json-file
      options:
        max-size: "10m"
        max-file: "3"

  db:
    image: postgres:16
    user: "999:999"
    read_only: true
    security_opt:
      - no-new-privileges:true
    cap_drop:
      - ALL
    cap_add:
      - CHOWN
      - DAC_OVERRIDE
      - FOWNER
      - SETGID
      - SETUID
    tmpfs:
      - /tmp:size=50M,noexec,nosuid
      - /var/run/postgresql:size=1M
    volumes:
      - pg-data:/var/lib/postgresql/data
    networks:
      - backend
    secrets:
      - db_password
    environment:
      POSTGRES_PASSWORD_FILE: /run/secrets/db_password
    shm_size: '256mb'
    deploy:
      resources:
        limits:
          memory: 1G
          cpus: '1.0'

networks:
  frontend:
    driver: bridge
  backend:
    driver: bridge
    internal: true

volumes:
  pg-data:
    driver: local

secrets:
  db_url:
    file: ./secrets/db_url.txt
  api_key:
    file: ./secrets/api_key.txt
  db_password:
    file: ./secrets/db_password.txt
```

**Why this works:**

- **Secrets via files, not ENV:** The `secrets:` section mounts secret files at `/run/secrets/`. The environment variables `DATABASE_URL_FILE` and `API_KEY_FILE` tell the application where to find the secrets. The secrets themselves are never in environment variables or `docker inspect` output.
- **`cap_drop: ALL`** removes all Linux capabilities. The app service adds none back because it listens on port 3000 (above 1024) and does not need any capabilities. The database service adds back the minimum needed for PostgreSQL initialization (file ownership operations).
- **`read_only: true`** makes the filesystem read-only. `tmpfs` provides writable space for `/tmp` and the PostgreSQL socket directory.
- **`no-new-privileges: true`** prevents processes from gaining additional privileges through setuid binaries or other mechanisms.
- **`internal: true` on the backend network** means containers on that network cannot reach the internet. The database is only accessible from the backend network, not from the outside.
- **Resource limits** prevent a single container from consuming all host resources (memory/CPU exhaustion attacks or bugs).
- **`postgres:16`** is pinned to a minor version, not `:latest`, ensuring reproducible builds.

---

## Common Mistakes

- **Forgetting `USER` before `CMD`:** If `USER` comes after `CMD` in the Dockerfile, the container still runs as root. The `USER` directive affects all subsequent instructions, including `CMD`.
- **Using `npm install` instead of `npm ci`:** `npm install` can modify `package-lock.json` and is not deterministic. `npm ci` installs from the lockfile exactly and is the correct command for CI/CD and Docker builds.
- **Dropping ALL capabilities but not considering the database:** PostgreSQL needs specific capabilities for its initialization process (CHOWN, SETUID, etc.). Dropping ALL and adding back the minimum is the correct approach, not skipping the drop entirely.
- **Exposing database ports to the host:** The database should only be accessible from the backend network. Exposing port 5432 to the host means any process on the host (or any container with host network access) can connect to it.
- **Using `:latest` tag:** The `:latest` tag is a moving target. It can change between builds, making your deployment non-reproducible. Always pin to a specific version.
