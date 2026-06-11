# Exercise 02: GitHub Actions Pipeline for Docker

**Type:** Guided
**Time:** 30 minutes
**Difficulty:** Easy-Medium

## Objective

Build a working GitHub Actions workflow that builds a Docker image, runs tests inside the container, and pushes the image to a container registry on every push to the `main` branch.

## Scenario

Your team has a Node.js web application with the following structure:

```
myapp/
├── Dockerfile
├── package.json
├── src/
│   └── server.js
└── tests/
    └── server.test.js
```

The existing `Dockerfile` is:

```dockerfile
FROM node:20.11-alpine AS builder
WORKDIR /app
COPY package*.json ./
RUN npm ci --production
COPY src/ ./src/

FROM node:20.11-alpine
WORKDIR /app
COPY --from=builder /app .
EXPOSE 3000
CMD ["node", "src/server.js"]
```

The `package.json` includes a test script:

```json
{
  "name": "myapp",
  "version": "1.0.0",
  "scripts": {
    "start": "node src/server.js",
    "test": "jest --ci --reporters=default",
    "lint": "eslint src/"
  }
}
```

You need to create a GitHub Actions workflow that automates the build-test-push cycle.

## Tasks

### Part A: Create the Workflow File

Create the file `.github/workflows/ci.yml` with the following requirements:

1. Trigger on `push` to the `main` branch and on `pull_request` targeting `main`.
2. Use the `ubuntu-latest` runner.
3. Check out the repository code.
4. Set up Docker Buildx for advanced build features.
5. Build the Docker image using the existing Dockerfile.

Your workflow should produce a valid YAML file. Start with the basic structure and build up.

<details>
<summary>Hint</summary>

The workflow file starts with `name:`, then `on:`, then `jobs:`. Each job has `runs-on:` and `steps:`. The checkout action is `actions/checkout@v4`. Docker Buildx is set up with `docker/setup-buildx-action@v3`.

</details>

### Part B: Add the Test Stage

Extend the workflow to run tests inside the built Docker container before pushing. The requirements are:

1. Build the image with a test target (you may need to modify the workflow, not the Dockerfile).
2. Run `npm test` inside the built container.
3. Run `npm run lint` inside the built container.
4. If either step fails, the workflow must stop and report failure.

<details>
<summary>Hint</summary>

You can run commands inside a Docker container using `docker run --rm <image> <command>`. To build the image first and then run tests, use a multi-step approach: build the image with a tag, then run tests against that tag.

</details>

### Part C: Push to GitHub Container Registry

Add image push functionality to the workflow:

1. Authenticate to GitHub Container Registry (ghcr.io) using the built-in `GITHUB_TOKEN`.
2. Generate image tags based on the Git SHA and branch name.
3. Push the image only on pushes to `main` (not on pull requests).
4. Use the `docker/build-push-action@v5` or equivalent.

<details>
<summary>Hint</summary>

Use `docker/login-action@v3` with `registry: ghcr.io`, `username: ${{ github.actor }}`, and `password: ${{ secrets.GITHUB_TOKEN }}`. For tagging, use `docker/metadata-action@v5` to generate tags from Git metadata.

</details>

### Part D: Add Caching

Docker builds can be slow. Add build cache optimization to your workflow:

1. Enable GitHub Actions cache as the Buildx cache backend.
2. Ensure subsequent builds reuse unchanged layers.
3. Verify that a second run of the workflow is faster than the first.

<details>
<summary>Hint</summary>

In the `docker/build-push-action`, set `cache-from: type=gha` and `cache-to: type=gha,mode=max`. This uses the GitHub Actions cache to store and retrieve Docker build layers.

</details>

## Success Criteria

- [ ] Your workflow triggers on push to `main` and on pull requests targeting `main`.
- [ ] The workflow builds a Docker image using Buildx.
- [ ] Tests and linting run inside the container and failures halt the pipeline.
- [ ] The image is pushed to ghcr.io only on pushes to `main`, not on PRs.
- [ ] Image tags include the Git SHA for traceability.
- [ ] Build caching is configured to speed up subsequent builds.
- [ ] The complete workflow YAML is valid and follows GitHub Actions syntax.

## What You Should Understand After This Exercise

A GitHub Actions workflow for Docker is a sequence of jobs and steps defined in YAML. The workflow reacts to Git events (push, PR), runs in an isolated environment (the runner), and executes deterministic steps (checkout, build, test, push). Conditional execution (`if:`) controls which steps run on which events. Caching dramatically reduces build times for unchanged layers. The entire pipeline is version-controlled alongside the application code.
