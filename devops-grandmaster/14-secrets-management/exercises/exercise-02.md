# Exercise 02: Docker Secrets for a Swarm Service

**Type:** Guided
**Time:** 30 minutes
**Difficulty:** Easy-Medium

## Objective

Create and manage Docker secrets in Swarm mode, then wire them into a service so the application reads secrets from files in `/run/secrets/` instead of environment variables. This exercise trains the mechanical skill of using Docker's built-in secret management.

## Scenario

Your team runs a PostgreSQL database and a web application in Docker Swarm. Currently, the database password is passed as an environment variable in the stack file. The security team requires you to migrate to Docker Secrets. You need to create the secrets, update the stack file, and verify that the application reads them correctly.

## Starting Code

### docker-compose.yml (insecure -- current state)

```yaml
version: '3.8'

services:
  db:
    image: postgres:15-alpine
    environment:
      POSTGRES_USER: appuser
      POSTGRES_PASSWORD: super-secret-password-123!
      POSTGRES_DB: myapp
    volumes:
      - db_data:/var/lib/postgresql/data

  app:
    image: nginx:alpine
    ports:
      - "8080:80"
    environment:
      DB_HOST: db
      DB_USER: appuser
      DB_PASSWORD: super-secret-password-123!
      DB_NAME: myapp

volumes:
  db_data:
```

## Tasks

### Part A: Initialize Docker Swarm

Docker secrets are only available in Swarm mode. Initialize a single-node Swarm on your local machine.

<details>
<summary>Hint</summary>

Run `docker swarm init`. On a single machine this creates a manager node. You do not need additional nodes for this exercise.

</details>

### Part B: Create Docker Secrets

Create two secrets using the Docker CLI:

1. `db_password` -- the database password (`super-secret-password-123!`).
2. `db_user` -- the database username (`appuser`).

Verify that the secrets were created by listing them. Confirm that you cannot read the secret value back from Docker.

<details>
<summary>Hint</summary>

Use `echo "value" | docker secret create name -` to create a secret from stdin. Use `docker secret ls` to list secrets. Try `docker secret inspect` -- notice that it shows metadata but not the secret value itself.

</details>

### Part C: Update the Stack File

Rewrite `docker-compose.yml` to use Docker secrets instead of environment variables for the sensitive values. The file should:

1. Define the secrets at the top level using `external: true`.
2. Mount the secrets into both services under `secrets:`.
3. Keep non-sensitive environment variables (`DB_HOST`, `DB_NAME`) as environment variables.
4. Add environment variables that tell the application where to find the secret files (e.g., `DB_PASSWORD_FILE=/run/secrets/db_password`).

<details>
<summary>Hint</summary>

The `secrets:` directive at the service level mounts each secret as a file at `/run/secrets/<secret_name>`. The `secrets:` directive at the top level references the pre-created secret by name with `external: true`. Do not put the actual password value in the compose file.

</details>

### Part D: Update the Application to Read Secret Files

Write a shell script (`read-secrets.sh`) that demonstrates how an application should read secrets from files. The script should:

1. Read the database password from `/run/secrets/db_password`.
2. Read the database user from `/run/secrets/db_user`.
3. Print a confirmation message showing the length of the password (not the password itself).
4. Exit with an error if either secret file is missing.

<details>
<summary>Hint</summary>

Use `cat /run/secrets/db_password` to read the file. Use `${#VARIABLE}` in bash to get the string length. Use `-f` to test if a file exists.

</details>

### Part E: Deploy and Verify

Deploy the stack and verify that:

1. The secrets are mounted in the containers.
2. The application can read the secrets.
3. `docker inspect` does not reveal the secret values.

Write the commands you would run to perform these checks.

<details>
<summary>Hint</summary>

Use `docker stack deploy -c docker-compose.yml myapp` to deploy. Use `docker exec` to list files in `/run/secrets/` inside a container. Use `docker inspect` and search for the password string -- it should not appear.

</details>

## Success Criteria

- [ ] Docker Swarm is initialized on your machine.
- [ ] Two secrets (`db_password`, `db_user`) are created and visible in `docker secret ls`.
- [ ] The stack file references secrets with `external: true` and mounts them via `secrets:`.
- [ ] No secret values appear in the compose file or in `docker inspect` output.
- [ ] The application reads secrets from `/run/secrets/` files.
- [ ] The stack deploys successfully and the application can access the database.

## What You Should Understand After This Exercise

Docker Swarm secrets provide a built-in mechanism for managing sensitive data. Secrets are encrypted at rest in the Swarm's Raft store, encrypted in transit over TLS, and mounted as tmpfs files that never touch the container's writable layer. The application reads secrets from files, not from the environment. This pattern keeps secrets out of `docker inspect`, out of build artifacts, and out of CI/CD logs. The trade-off is that it requires Swarm mode and that your application must be written to read from files rather than from environment variables.
