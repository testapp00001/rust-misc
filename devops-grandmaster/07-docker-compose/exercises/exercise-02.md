# Exercise 02: Build a Three-Service Stack

**Type:** Guided
**Time:** 30 minutes
**Difficulty:** Easy-Medium

## Objective

Write a complete `docker-compose.yml` file for a web application with a database
and a cache. This exercise trains you to define services, configure networking,
pass environment variables, and use volumes.

## Scenario

You are building a task management API. The architecture:

```
Browser --> API (Node.js, port 3000) --> PostgreSQL (port 5432)
                                  \---> Redis (port 6379)
```

The API needs:
- A PostgreSQL 16 database named `taskdb` with user `taskuser` and password `taskpass`
- A Redis 7 cache
- Environment variables to connect to both services
- Port 3000 exposed to the host

The PostgreSQL data must survive container restarts.

## Tasks

### Part A: Write the Compose File

Create a file called `docker-compose.yml` with three services: `postgres`, `redis`,
and `api`. Follow these requirements:

1. Use `postgres:16-alpine` and `redis:7-alpine` images.
2. For the `api` service, use `image: node:20-alpine` with a `command` that sleeps
   forever (we will not build a real app yet -- just get the stack running).
3. Set environment variables for PostgreSQL: `POSTGRES_DB`, `POSTGRES_USER`,
   `POSTGRES_PASSWORD`.
4. Set `DATABASE_URL` and `REDIS_URL` on the `api` service so it can connect
   to the other containers by hostname.
5. Create a named volume for PostgreSQL data.
6. Expose port 3000 on the `api` service.

<details>
<summary>Hint 1: Environment Variables</summary>

The `api` container connects to other services using their **service names** as
hostnames. The `DATABASE_URL` should look like:

```
postgresql://taskuser:taskpass@postgres:5432/taskdb
```

The hostname `postgres` resolves to the PostgreSQL container because Compose
creates a shared network and DNS entries automatically.

</details>

<details>
<summary>Hint 2: Volume Declaration</summary>

Named volumes must be declared in two places:
1. Under the service that uses them (the mount)
2. At the top level of the compose file (the `volumes:` key)

```yaml
services:
  postgres:
    volumes:
      - pgdata:/var/lib/postgresql/data

volumes:
  pgdata:
```

</details>

### Part B: Validate and Start

Run the following commands and record the output:

```bash
# Validate the compose file syntax
docker compose config

# Start all services
docker compose up -d

# Check that all three services are running
docker compose ps
```

<details>
<summary>Hint</summary>

If `docker compose config` shows errors, read the error message carefully.
Common issues:
- Indentation (YAML is whitespace-sensitive)
- Missing colons after keys
- Forgetting to declare named volumes at the bottom

</details>

### Part C: Test Connectivity

Verify that the services can reach each other:

```bash
# Open a shell in the api container
docker compose exec api sh

# Inside the container, test DNS resolution
ping postgres
ping redis

# Exit the shell
exit
```

<details>
<summary>Hint</summary>

If `ping` is not available in the Alpine image, install it:

```bash
apk add --no-cache busybox-extras
```

Or use `nslookup` instead:

```bash
nslookup postgres
nslookup redis
```

</details>

### Part D: Inspect the Network

From the host, inspect the network Compose created:

```bash
# List networks
docker network ls | grep <your-project-name>

# Inspect the network to see connected containers
docker network inspect <project-name>_default
```

Answer these questions:
1. What is the name of the network Compose created?
2. How many containers are connected to it?
3. What IP addresses were assigned?

<details>
<summary>Hint</summary>

Compose names the default network using the project name (the directory name)
plus `_default`. So if your directory is `myproject`, the network is
`myproject_default`.

</details>

### Part E: Clean Up

Stop and remove everything:

```bash
docker compose down
```

Verify the volume still exists:

```bash
docker volume ls | grep pgdata
```

Then remove the volume too:

```bash
docker compose down -v
```

## Success Criteria

- [ ] Your `docker-compose.yml` passes `docker compose config` without errors
- [ ] All three services start and show as "running" in `docker compose ps`
- [ ] The `api` container can resolve `postgres` and `redis` by hostname
- [ ] PostgreSQL data persists after `docker compose down` (but not after `down -v`)
- [ ] You can explain why service names work as hostnames

## What You Should Understand After This Exercise

A Compose file defines the *desired state* of your application stack. Compose
handles network creation, DNS registration, and volume management automatically.
Service names become hostnames on the shared default network -- no manual `docker
network create` or `--link` required.
