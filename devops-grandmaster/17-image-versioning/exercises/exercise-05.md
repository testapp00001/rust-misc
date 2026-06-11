# Exercise 05: Design a Versioning and Release Strategy

**Type:** Integration
**Time:** 60 minutes
**Difficulty:** Hard

## Objective

Design and document a complete versioning and release strategy for a production application that covers image tagging, base image management, release automation, environment promotion, and rollback -- integrating all concepts from this module.

## Scenario

Your company is launching a new SaaS product with three environments (development, staging, production) and a team of eight developers. The application is containerized and deployed via CI/CD. You have been asked to design the versioning and release strategy that will govern how images are built, tagged, promoted, and rolled back across all environments.

The strategy must answer:

- How are images tagged?
- How do you promote an image from staging to production?
- How do you handle hotfixes?
- How do you roll back?
- How do you prevent someone from accidentally deploying the wrong image?
- How do you audit what is running in production?

## Tasks

### Part A: Tagging Strategy Design

Design the complete tagging strategy. For each tag type, specify:

1. The tag format.
2. When it is applied (which trigger).
3. Whether it is mutable or immutable.
4. Who or what creates it.
5. Its primary use case.

Tag types to define:

- Semantic version tags (e.g., `v1.2.3`)
- Git SHA tags (e.g., `a1b2c3d`)
- Combined tags (e.g., `v1.2.3-a1b2c3d`)
- Environment tags (e.g., `v1.2.3-staging`, `v1.2.3-production`)
- Branch tags (e.g., `main`, `feature-auth`)
- The `latest` tag
- Date-based tags (e.g., `20240115`)
- Base image tags (how you pin your `FROM` images)

Create a table summarizing all tag types and their properties.

<details>
<summary>Hint</summary>

Not every image needs every tag type. Semantic version tags are created on releases. Git SHA tags are created on every build. Environment tags are applied during promotion. Branch tags are for development. The `latest` tag has specific, limited use.

</details>

### Part B: Release Workflow Design

Design the release workflow from commit to production. Your design must include:

1. **Development flow:** How developers build and test locally.
2. **CI flow:** What happens on every push to `main`.
3. **Release flow:** How a release is created (who triggers it, what tags are created).
4. **Promotion flow:** How an image moves from staging to production.
5. **Hotfix flow:** How a critical bug fix bypasses the normal release cycle.

For each flow, specify:

- The trigger (manual, automatic, tag-based).
- The CI/CD steps that run.
- The tags that are created or moved.
- The environments affected.
- The approval gates (if any).

Draw the complete flow as an ASCII diagram or a numbered sequence.

<details>
<summary>Hint</summary>

A common pattern: developers push to feature branches (CI builds and tests). Merges to `main` trigger staging deployment. A release manager creates a Git tag (`v1.2.3`) which triggers the production release pipeline. Hotfixes branch from the release tag, fix the bug, and produce a patch release (`v1.2.4`).

</details>

### Part C: Environment Promotion Strategy

Design how images move between environments. Your design must address:

1. **Immutable promotion:** Once an image is tested in staging, the exact same image (by digest) must be deployed to production. No rebuilding.
2. **Approval gates:** What approvals are required before promotion to production?
3. **Environment-specific configuration:** How do you handle environment-specific settings (database URLs, API keys) without rebuilding the image?
4. **Rollback per environment:** Rolling back staging should not affect production, and vice versa.

Write the promotion workflow as a GitHub Actions workflow or as a detailed step-by-step process.

<details>
<summary>Hint</summary>

The key insight is: build once, deploy everywhere. The image is built and tagged once in CI. Promotion to staging and production deploys the same image (by digest) with different configuration. Use docker-compose overrides or Kubernetes ConfigMaps for environment-specific settings.

</details>

### Part D: Base Image Management

Design the strategy for managing base images (the `FROM` line in Dockerfiles). Your design must address:

1. **Pinning:** How base images are pinned (tag only, tag + digest, or digest only).
2. **Updates:** How and when base images are updated (manual, Dependabot, scheduled).
3. **Security patches:** What happens when a CVE is found in a base image.
4. **Compatibility testing:** How you verify that a base image update does not break your application.
5. **Documentation:** Where base image versions are documented and who is responsible for updates.

Write a base image policy document (5-10 bullet points).

<details>
<summary>Hint</summary>

Pin to a specific version and digest in production (e.g., `FROM node:18.19.0-alpine3.19@sha256:abc123...`). Use Dependabot or Renovate to propose updates. Updates go through the normal CI pipeline (build, test, scan). Security patches get expedited through a shortened release cycle.

</details>

### Part E: Rollback and Disaster Recovery

Design the rollback strategy for three failure scenarios:

**Scenario 1: Bad application code**

A new release introduces a bug. You need to roll back to the previous version.

**Scenario 2: Bad base image update**

A base image update introduces a subtle incompatibility. You need to roll back the base image without rolling back application code.

**Scenario 3: Registry outage**

Your container registry is unreachable. New deployments and rollbacks are blocked.

For each scenario, specify:

1. How you detect the problem.
2. How you identify the rollback target.
3. The exact rollback procedure.
4. How you verify the rollback succeeded.
5. How you prevent the problem from recurring.

<details>
<summary>Hint</summary>

Scenario 1: Roll back by deploying the previous image digest from deployment history. Scenario 2: Rebuild the current application code with the previous base image digest. Scenario 3: Maintain a mirror registry or local image cache. Use `docker save`/`docker load` as a last resort.

</details>

### Part F: Audit and Compliance

Design the audit system that answers these questions at any time:

1. What exact image (by digest) is running in production right now?
2. What code (by Git SHA) produced that image?
3. When was it deployed, and by whom?
4. What base image was used, and when was it last updated?
5. What vulnerabilities were known at the time of deployment?
6. What was the previous production image, and when was it replaced?

Write the audit requirements and specify where each piece of information is recorded (deployment log, image labels, CI/CD logs, registry metadata).

<details>
<summary>Hint</summary>

OCI labels on the image store version, SHA, and build date. The deployment log stores who deployed what and when. The registry stores image digests and scan results. CI/CD logs store the full pipeline output. Combine all four sources for a complete audit trail.

</details>

## Success Criteria

- [ ] Your tagging strategy defines at least 6 tag types with clear creation triggers and immutability rules.
- [ ] Your release workflow covers development, CI, release, promotion, and hotfix flows.
- [ ] Your promotion strategy ensures the same image (by digest) runs in staging and production.
- [ ] Your base image policy specifies pinning, update cadence, and security patch procedures.
- [ ] Your rollback strategy addresses bad code, bad base images, and registry outages.
- [ ] Your audit system can answer all six compliance questions at any time.
- [ ] The complete strategy is implementable as GitHub Actions workflows, shell scripts, and docker-compose configurations.
- [ ] You can explain why "build once, deploy everywhere" is the foundation of a reliable release strategy.

## What You Should Understand After This Exercise

A versioning and release strategy is more than just tagging images. It is a system that connects code changes to running containers with full traceability. Semantic versioning communicates intent. Git SHA tags provide traceability. Digest-based promotion ensures consistency. Base image pinning prevents drift. Deployment history enables deterministic rollback. OCI labels make images self-describing. The entire strategy must be enforceable by automation -- policies that depend on humans following them will eventually be broken.
