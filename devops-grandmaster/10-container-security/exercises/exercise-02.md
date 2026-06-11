# Exercise 02: Harden a Dockerfile

**Type:** Guided
**Time:** 30 minutes
**Difficulty:** Easy-Medium

## Objective

Take an insecure Dockerfile and apply security hardening best practices: non-root user, read-only filesystem support, dropped capabilities, no-new-privileges, and secrets management. You will transform a vulnerable container into one that follows the principle of least privilege.

## Scenario

Your team has a Node.js web application with the following Dockerfile. A security audit has flagged it as non-compliant. Your job is to fix every issue.

**The insecure Dockerfile:**

```dockerfile
FROM node:18

WORKDIR /app

COPY . .

RUN npm install

EXPOSE 3000

CMD ["node", "server.js"]
```

**The insecure docker-compose.yml:**

```yaml
version: "3.8"

services:
  app:
    build: .
    ports:
      - "3000:3000"
    environment:
      DATABASE_URL: "postgres://admin:supersecret@db:5432/myapp"
      API_KEY: "ak-12345-abcdef"
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock

  db:
    image: postgres:latest
    environment:
      POSTGRES_PASSWORD: "supersecret"
    ports:
      - "5432:5432"
```

## Tasks

### Part A: Harden the Dockerfile

Rewrite the Dockerfile to address these security issues:

1. The base image is bloated and has more packages than needed.
2. The container runs as root.
3. `COPY . .` before `npm install` means every code change invalidates the dependency cache (not a security issue, but fix it while you are here).
4. There is no `.dockerignore` consideration.
5. No health check is defined.

Your hardened Dockerfile should:
- Use a slim or Alpine base image.
- Create and use a non-root user (UID 1001).
- Support a read-only filesystem at runtime.
- Include a health check.
- Use multi-stage or ordered COPY for layer caching.

<details>
<summary>Hint 1: Base image</summary>

Replace `node:18` with `node:18-slim` or `node:18-alpine`. Alpine images are significantly smaller and have fewer installed packages, which means fewer potential vulnerabilities.

</details>

<details>
<summary>Hint 2: Non-root user</summary>

Use `addgroup` and `adduser` (Alpine) or `groupadd` and `useradd` (Debian slim) to create a dedicated user. Set ownership of the app directory with `--chown` on COPY instructions. Place the `USER` directive before `CMD`.

</details>

<details>
<summary>Hint 3: Read-only support</summary>

The application needs to be able to write to certain directories at runtime (for example, `/tmp` for temporary files). The Dockerfile itself does not enable read-only mode -- that is a runtime setting in docker-compose. But you should ensure the application does not write to unexpected paths.

</details>

### Part B: Harden the docker-compose.yml

Rewrite the docker-compose.yml to fix these issues:

1. Secrets are stored in environment variables.
2. The Docker socket is mounted.
3. No capability restrictions.
4. No resource limits.
5. No read-only filesystem.
6. The database image uses `:latest` tag.
7. The database port is exposed to the host.
8. No network isolation.

Your hardened compose file should:
- Use Docker secrets or file-based secrets for all sensitive values.
- Remove the Docker socket mount entirely.
- Drop all capabilities and add back only what is needed.
- Set memory and CPU limits.
- Enable read-only filesystem with tmpfs for writable paths.
- Pin the database image version.
- Use an internal network for the database.
- Apply `no-new-privileges`.

<details>
<summary>Hint 1: Secrets</summary>

Create secret files in a `secrets/` directory. Use the `secrets:` top-level key in Compose and reference them in the service. Set environment variables like `DATABASE_URL_FILE` that point to `/run/secrets/...`. Your application code must be updated to read from files (but for this exercise, focus on the Compose configuration).

</details>

<details>
<summary>Hint 2: Network isolation</summary>

Create two networks: `frontend` (for the app, accessible from outside) and `backend` (for database communication, marked as `internal: true`). The app connects to both; the database connects only to `backend`.

</details>

<details>
<summary>Hint 3: Capabilities</summary>

A typical web application that listens on port 3000 (above 1024) does not need any capabilities. Use `cap_drop: ["ALL"]` and do not add any back. The database may need a few (CHOWN, DAC_OVERRIDE, FOWNER, SETGID, SETUID) for its initialization process.

</details>

### Part C: Verify Your Hardening

After writing your hardened files, verify them:

```bash
# Build the hardened image
docker compose build

# Check that the container runs as non-root
docker compose run --rm app whoami

# Check that the filesystem is read-only (should fail)
docker compose run --rm app sh -c 'echo test > /usr/bin/nope'

# Check that /tmp is writable (should succeed via tmpfs)
docker compose run --rm app sh -c 'echo test > /tmp/ok'

# Check capabilities
docker compose run --rm app sh -c 'cat /proc/1/status | grep CapEff'
```

Document the results. If any check fails, revise your configuration.

---

## Success Criteria

- [ ] The Dockerfile uses a minimal base image and runs as a non-root user (UID 1001).
- [ ] No secrets appear in environment variables in docker-compose.yml.
- [ ] The Docker socket is not mounted into any container.
- [ ] All capabilities are dropped (`cap_drop: ALL`) for the app service.
- [ ] The database service is on an internal network and its port is not exposed to the host.

## What You Should Understand After This Exercise

Hardening a container is a systematic process: minimize the base image, run as non-root, drop capabilities, isolate networks, manage secrets properly, and remove unnecessary access (like the Docker socket). Each layer of defense reduces the blast radius if the container is compromised.
