# Solution 03: Traefik with Auto-Discovery

## Complete Solution

### `docker-compose.yml`

```yaml
version: "3.8"

services:
  traefik:
    image: traefik:v3.0
    command:
      - "--api.dashboard=true"
      - "--providers.docker=true"
      - "--providers.docker.exposedbydefault=false"
      - "--entrypoints.web.address=:80"
    ports:
      - "80:80"
      - "8080:8080"
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock:ro
    labels:
      # Enable Traefik for itself
      - "traefik.enable=true"
      # Dashboard router
      - "traefik.http.routers.dashboard.rule=PathPrefix(`/dashboard`) || PathPrefix(`/api`)"
      - "traefik.http.routers.dashboard.service=api@internal"
      - "traefik.http.routers.dashboard.entrypoints=web"

  whoami-1:
    image: traefik/whoami
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.whoami1.rule=Host(`whoami.localhost`)"
      - "traefik.http.routers.whoami1.entrypoints=web"
      - "traefik.http.services.whoami1.loadbalancer.server.port=80"

  whoami-2:
    image: traefik/whoami
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.whoami2.rule=Host(`whoami2.localhost`)"
      - "traefik.http.routers.whoami2.entrypoints=web"
      - "traefik.http.services.whoami2.loadbalancer.server.port=80"
```

---

## Why It Works

### Traefik static configuration via CLI flags

```yaml
command:
  - "--api.dashboard=true"
```
Enables the Traefik dashboard (a web UI for monitoring routers, services, and middlewares).

```yaml
  - "--providers.docker=true"
```
Tells Traefik to watch Docker for new containers. When a container starts (or stops), Traefik automatically updates its routing configuration.

```yaml
  - "--providers.docker.exposedbydefault=false"
```
Security best practice. Only containers with `traefik.enable=true` are routed. Without this flag, every container on the Docker network would be exposed, even internal services like databases.

```yaml
  - "--entrypoints.web.address=:80"
```
Defines an entrypoint named `web` that listens on port 80. All routers reference this entrypoint.

### Docker socket mount

```yaml
volumes:
  - /var/run/docker.sock:/var/run/docker.sock:ro
```
Traefik needs read-only access to the Docker daemon to discover containers. This is how auto-discovery works — Traefik monitors Docker events (container start, stop, label changes) and updates its routing table in real time.

### Service labels

For `whoami-1`:

```yaml
- "traefik.enable=true"
```
Opts this container into Traefik routing (required because `exposedByDefault=false`).

```yaml
- "traefik.http.routers.whoami1.rule=Host(`whoami.localhost`)"
```
Creates a router named `whoami1` that matches requests with the `Host` header `whoami.localhost`.

```yaml
- "traefik.http.routers.whoami1.entrypoints=web"
```
Attaches this router to the `web` entrypoint (port 80). Without this, the router exists but has no entrypoint, so no traffic reaches it.

```yaml
- "traefik.http.services.whoami1.loadbalancer.server.port=80"
```
Tells Traefik which port the container listens on. The `traefik/whoami` image listens on port 80 by default. Without this label, Traefik does not know which port to forward to.

### Dashboard routing

```yaml
- "traefik.http.routers.dashboard.rule=PathPrefix(`/dashboard`) || PathPrefix(`/api`)"
- "traefik.http.routers.dashboard.service=api@internal"
```
The dashboard is not a container — it is an internal Traefik service. The `api@internal` reference points to Traefik's built-in API. The `PathPrefix` rules ensure both the dashboard UI (`/dashboard/`) and the API endpoints (`/api/`) are accessible.

### Dynamic auto-discovery test

When you add `whoami-3` and run `docker compose up -d`, Traefik detects the new container via the Docker socket. It reads the labels, creates a new router and service, and begins routing traffic — all without restarting. This is the core value of Traefik: infrastructure-as-code with zero-downtime updates.

---

## Common Mistakes

### 1. `traefik.enable=true` is missing

**Symptom:** The service is running but `curl` returns 404 from Traefik.

**Cause:** With `exposedByDefault=false`, Traefik ignores any container without `traefik.enable=true`. This is the most common Traefik misconfiguration.

### 2. Entry point not specified on the router

**Symptom:** The router appears in the dashboard but has no entry point attached.

**Cause:** Missing `traefik.http.routers.NAME.entrypoints=web`. The router exists but is not bound to any entry point, so no traffic reaches it.

### 3. Wrong port in `loadbalancer.server.port`

**Symptom:** Traefik shows the service as "down" or returns a 502 error.

**Cause:** The port in the label does not match the port the container actually listens on. For `traefik/whoami`, the port is 80. For a custom Flask app, it might be 5000.

### 4. Dashboard returns 404

**Symptom:** `curl http://localhost:8080/dashboard` returns 404.

**Cause:** Two possible issues:
- Missing trailing slash. Traefik redirects `/dashboard` to `/dashboard/`, but some HTTP clients (including `curl`) do not follow redirects by default. Use `curl -L http://localhost:8080/dashboard/`.
- The dashboard router is not attached to the correct entrypoint. Ensure `traefik.http.routers.dashboard.entrypoints=web` or use a separate entrypoint on port 8080.

### 5. Mounting Docker socket without `:ro`

**Symptom:** Security concern, not a functional error.

**Cause:** Without `:ro`, a compromised Traefik container could modify Docker containers (start, stop, create). Always mount the socket as read-only: `/var/run/docker.sock:/var/run/docker.sock:ro`.

### 6. Confusing router names

**Symptom:** One service works, the other does not.

**Cause:** Traefik router names must be unique across the entire Docker Compose project. If two services both use `traefik.http.routers.app.rule=...`, the second overwrites the first. Use distinct names like `whoami1` and `whoami2`.
