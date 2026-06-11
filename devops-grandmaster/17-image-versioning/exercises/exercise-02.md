# Exercise 02: Implement Semantic Versioning for Docker Images

**Type:** Guided
**Time:** 30 minutes
**Difficulty:** Easy-Medium

## Objective

Implement a complete semantic versioning system for Docker images, including a version management script, a versioned Dockerfile with build metadata, and a docker-compose setup that uses versioned images.

## Scenario

Your team has decided to adopt semantic versioning for all container images. You need to build the tooling that makes this work: a script to bump versions, a Dockerfile that bakes version metadata into the image, and a docker-compose file that requires an explicit version at deploy time.

## Tasks

### Part A: Create a Version Management Script

Create a shell script `version.sh` that manages a `VERSION` file containing a semantic version string (e.g., `1.0.0`).

The script must support these commands:

```
./version.sh show          # Print the current version
./version.sh patch         # Bump patch: 1.0.0 -> 1.0.1
./version.sh minor         # Bump minor: 1.0.1 -> 1.1.0
./version.sh major         # Bump major: 1.1.0 -> 2.0.0
./version.sh set 3.2.1     # Set an arbitrary version
```

Requirements:

1. The script reads from and writes to a file called `VERSION` in the current directory.
2. If `VERSION` does not exist, the script defaults to `0.0.0`.
3. `minor` resets `patch` to 0. `major` resets both `minor` and `patch` to 0.
4. The script prints the old and new version when bumping (e.g., "Version bumped: v1.0.0 -> v1.0.1").
5. Invalid version strings for `set` should produce an error.

<details>
<summary>Hint</summary>

Use `IFS='.'` to split the version string into major, minor, and patch components. Use `case` for the command dispatch. For validation, check that the version string matches the pattern `N.N.N` where N is a non-negative integer.

</details>

### Part B: Create a Versioned Dockerfile

Create a `Dockerfile` for a simple Node.js application that:

1. Uses a pinned base image (specific version, not a floating tag).
2. Accepts `APP_VERSION`, `GIT_SHA`, and `BUILD_DATE` as build arguments.
3. Injects those arguments as environment variables so the running application can read them.
4. Adds OCI-compliant labels (`org.opencontainers.image.version`, `org.opencontainers.image.revision`, `org.opencontainers.image.created`).
5. Runs as a non-root user.

Write the Dockerfile and a minimal `server.js` that returns the version information as JSON at the root endpoint.

<details>
<summary>Hint</summary>

Use `ARG` for build-time variables and `ENV` to make them available at runtime. Use `LABEL` with the OCI annotation keys. For the pinned base image, use something like `FROM node:18.19.0-alpine3.19` instead of `FROM node:18-alpine`.

</details>

### Part C: Create a Build Script

Create a shell script `build.sh` that:

1. Reads the version from the `VERSION` file.
2. Captures the current git SHA (short form).
3. Captures the build date in ISO 8601 format.
4. Builds the Docker image with all three build arguments.
5. Tags the image with four tags:
   - `myapp:<version>` (e.g., `myapp:v1.2.3`)
   - `myapp:<git-sha>` (e.g., `myapp:a1b2c3d`)
   - `myapp:<version>-<git-sha>` (e.g., `myapp:v1.2.3-a1b2c3d`)
   - `myapp:latest`
6. Prints a summary of the build (version, SHA, date, tags created).

The script should accept an optional version argument: `./build.sh 2.0.0` overrides the `VERSION` file.

<details>
<summary>Hint</summary>

Use `git rev-parse --short HEAD` for the SHA. Use `date -u +%Y-%m-%dT%H:%M:%SZ` for the ISO date. Use multiple `-t` flags in `docker build` to create all tags in one command.

</details>

### Part D: Create a Versioned Docker Compose Setup

Create three docker-compose files:

1. **docker-compose.yml** -- Base configuration that uses `${APP_VERSION}` as a required variable.
2. **docker-compose.staging.yml** -- Staging overrides (different port, `NODE_ENV=staging`).
3. **docker-compose.production.yml** -- Production overrides (replicas, resource limits, `NODE_ENV=production`).

Requirements:

1. `APP_VERSION` must be required -- if not set, docker-compose should fail with a clear error.
2. The base image name should use a `REGISTRY` variable with a default.
3. Each environment file should override only what differs from the base.

<details>
<summary>Hint</summary>

Use `${APP_VERSION:?APP_VERSION is required}` syntax in docker-compose to enforce required variables. Use `extends` or separate compose files with `-f` for environment overrides.

</details>

### Part E: Test the Versioning System

Write out the sequence of commands you would run to:

1. Initialize the version at `1.0.0`.
2. Build version `1.0.0`.
3. Bump to `1.0.1` and build.
4. Bump to `1.1.0` and build.
5. Deploy `1.0.0` to staging and verify it reports the correct version.
6. Deploy `1.1.0` to production and verify.
7. List all images and confirm all four tags exist for each version.

Write the commands and explain what each one does.

<details>
<summary>Hint</summary>

Use `APP_VERSION=1.0.0 docker compose -f docker-compose.yml -f docker-compose.staging.yml up -d` to deploy with a specific version. Use `curl http://localhost:3000/` to verify the version in the response.

</details>

## Success Criteria

- [ ] `version.sh` correctly bumps patch, minor, and major versions and persists to `VERSION`.
- [ ] `version.sh set` accepts valid versions and rejects invalid ones.
- [ ] The Dockerfile uses a pinned base image and bakes version metadata as build args, env vars, and labels.
- [ ] `build.sh` creates all four tags for each build.
- [ ] The docker-compose files require `APP_VERSION` and fail without it.
- [ ] You can deploy different versions to different environments simultaneously.
- [ ] The running container reports its version, git SHA, and build date at the root endpoint.

## What You Should Understand After This Exercise

Semantic versioning communicates the nature of changes (breaking, feature, fix) at a glance. Baking version metadata into the image at build time makes every container self-describing. Multiple tags per image serve different purposes: SemVer for readability, Git SHA for traceability, combined for both. Required environment variables in docker-compose prevent accidental deployment of unversioned images. The `VERSION` file becomes the single source of truth for what version to build.
