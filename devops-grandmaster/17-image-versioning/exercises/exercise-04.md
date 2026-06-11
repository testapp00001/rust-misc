# Exercise 04: Implement Rollback Strategy with Image Digests

**Type:** Challenge
**Time:** 45 minutes
**Difficulty:** Medium-Hard

## Objective

Implement a production-grade rollback strategy that uses image digests (not tags) as the source of truth, supports automatic rollback on failed health checks, and maintains a deployment history that makes rollback decisions deterministic.

## Scenario

Your team deploys `myapp:v1.2.3` to production. Thirty minutes later, error rates spike. The team decides to roll back. But they discover that `v1.2.3` was pushed with the wrong code due to a CI caching bug -- the tag points to the wrong image. They need a rollback system that does not depend on tags being correct. Image digests (sha256 content hashes) are the answer.

## Tasks

### Part A: Understand Image Digests

Answer these questions before implementing anything:

1. What is an image digest, and how does it differ from a tag?
2. If you push `myapp:v1.2.3` twice with different content, what happens to the tag? What happens to the digest?
3. How do you pull an image by digest? Write the exact `docker pull` command.
4. Can two different images ever have the same digest? Why or why not?
5. How do you find the digest of a running container's image?

<details>
<summary>Hint</summary>

An image digest is a sha256 hash of the image manifest. It is content-addressable: identical content always produces the same digest, and different content always produces a different digest. Pull by digest with `docker pull myapp@sha256:abc123...`.

</details>

### Part B: Create a Deployment History Tracker

Create a shell script `deploy.sh` that:

1. Accepts an image reference (tag or digest) and an environment name.
2. Deploys the image using docker-compose.
3. Records the deployment in a `deployments.log` file with:
   - Timestamp (ISO 8601)
   - Environment name
   - Image reference (what was requested)
   - Image digest (what was actually deployed)
   - Git SHA (extracted from image labels)
   - Deployer (current user)
4. Keeps the last 20 deployments per environment.
5. Verifies the deployment with a health check before recording it as successful.

The log format should be a structured format (JSON lines or CSV) that can be parsed by other scripts.

<details>
<summary>Hint</summary>

Use `docker inspect --format='{{index .RepoDigests 0}}'` to get the digest of a pulled image. Use `docker inspect --format='{{index .Config.Labels "org.opencontainers.image.revision"}}'` to get the Git SHA from OCI labels. Store logs as JSON lines for easy parsing with `jq`.

</details>

### Part C: Create a Rollback Script

Create a shell script `rollback.sh` that:

1. Accepts an environment name and optionally a target version or digest.
2. If no target is specified, rolls back to the previous successful deployment from `deployments.log`.
3. If a target digest is specified, deploys that exact digest.
4. If a target version tag is specified, resolves it to a digest first (to verify the tag has not been tampered with).
5. Performs a health check after rollback.
6. Records the rollback as a new entry in `deployments.log` (with a "rollback" annotation).
7. Prints a clear summary: what was running, what it rolled back to, and whether the rollback succeeded.

The script must handle these error cases:

- The target image is not found in the registry.
- The health check fails after rollback.
- There is no previous deployment to roll back to.

<details>
<summary>Hint</summary>

Parse `deployments.log` with `jq` to find the second-to-last entry for the given environment. Use `docker pull myapp@sha256:...` to deploy by digest. Use `docker inspect` to resolve a tag to its digest before deploying.

</details>

### Part D: Add Digest Pinning to Docker Compose

Modify your `docker-compose.yml` to support deploying by digest. The compose file must:

1. Support both tag-based references (`myapp:v1.2.3`) and digest-based references (`myapp@sha256:abc123...`).
2. Use an `IMAGE_REF` variable that can be either a tag or a digest.
3. Fail if `IMAGE_REF` is not set.
4. Include a health check that the deployment script can use.

Write the compose file and show how to use it for both tag-based and digest-based deployments.

<details>
<summary>Hint</summary>

Docker supports `image: myapp@sha256:abc123...` natively. Use `${IMAGE_REF:?IMAGE_REF is required}` in the compose file. The deploy script sets `IMAGE_REF` to either a tag or a full digest reference.

</details>

### Part E: Write a CI/CD Rollback Workflow

Create a GitHub Actions workflow `.github/workflows/rollback.yml` that:

1. Can be triggered manually with `workflow_dispatch`.
2. Accepts inputs: `environment` (staging/production), `target` (digest, tag, or "previous").
3. Validates that the target image exists in the registry.
4. Deploys the target image to the specified environment.
5. Runs health checks for 60 seconds.
6. If health checks fail, does NOT roll back again (to avoid rollback loops).
7. Posts a summary to the workflow run with: previous image, new image, health check result.

Write the complete workflow YAML.

<details>
<summary>Hint</summary>

Use `workflow_dispatch` with `inputs:` for the manual trigger. Use `docker manifest inspect` to validate that the image exists before deploying. Use a retry loop with `curl` for health checks. Use `concurrency:` to prevent simultaneous rollbacks to the same environment.

</details>

## Success Criteria

- [ ] You can explain the difference between a tag and a digest and why digests are more reliable for rollback.
- [ ] `deploy.sh` records every deployment with its digest, Git SHA, timestamp, and deployer.
- [ ] `rollback.sh` can roll back to the previous deployment or a specific digest.
- [ ] `rollback.sh` resolves tags to digests before deploying (to catch tampered tags).
- [ ] The docker-compose file supports both tag-based and digest-based image references.
- [ ] The rollback workflow can be triggered manually and validates images before deploying.
- [ ] Health checks run after every rollback and the result is recorded.
- [ ] Rollback entries are clearly annotated in the deployment history log.

## What You Should Understand After This Exercise

Tags are human-friendly aliases that can be reassigned. Digests are content-addressable hashes that are mathematically guaranteed to identify a specific image. Rollback by digest is the only reliable rollback strategy because it does not depend on tags being correct. A deployment history log that records digests creates an audit trail and makes rollback targets deterministic. The "rollback to previous" pattern is simple but powerful -- it means you always have a known-good target. CI/CD rollback workflows should validate images before deploying and avoid rollback loops.
