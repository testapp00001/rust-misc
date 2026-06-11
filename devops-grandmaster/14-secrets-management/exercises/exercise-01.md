# Exercise 01: Why Environment Variables Fail for Secrets

**Type:** Conceptual
**Time:** 15 minutes
**Difficulty:** Easy

## Objective

Understand why environment variables -- the standard mechanism for application configuration -- are fundamentally unsuitable for secrets. This exercise trains you to think about the attack surface that environment variables create and why dedicated secret management exists.

## Background

Your team completed [Module 13: Environment Variables](../../13-environment-variables/) and adopted the 12-Factor approach: all configuration lives in environment variables. Now the security team has flagged a concern. Your production containers pass `DATABASE_PASSWORD`, `API_KEY`, and `JWT_SECRET` as environment variables through `docker-compose.yml`. The security team says this is not acceptable for production. You need to understand why before you can propose an alternative.

## Tasks

### Part A: The Attack Surface

List four distinct ways an attacker (or an unauthorized team member) can read environment variables from a running container. For each method, describe the access level required and the specific command or API call that exposes the secret.

<details>
<summary>Hint</summary>

Think about what `docker inspect` reveals, what `/proc` exposes inside a container, what the orchestrator API returns, and what happens in CI/CD logs. Consider both local and remote access scenarios.

</details>

### Part B: Why "Just Use env vars" Breaks the Security Chain

A senior developer argues: "We already use environment variables for non-secret config like `LOG_LEVEL` and `PORT`. Adding `DB_PASSWORD` to the same mechanism keeps things simple and consistent."

Write a response (4-6 sentences) explaining why treating secrets the same as config creates specific security risks. Address at least three distinct problems: visibility, persistence, and auditability.

<details>
<summary>Hint</summary>

Consider: Who can see env vars in a CI/CD pipeline? Where do env vars persist after a container stops? Can you track who read a specific secret? What happens when a developer runs `docker inspect` in production?

</details>

### Part C: Classify the Exposure Risk

Below are six scenarios where secrets might be exposed through environment variables. Classify each as **Low**, **Medium**, or **High** risk, and explain your reasoning.

1. A `docker-compose.yml` with `DB_PASSWORD=secret123` committed to a private GitHub repo.
2. A CI/CD pipeline that passes `API_KEY` as an environment variable to a build step, which logs all env vars on failure.
3. A production container running with `JWT_SECRET` as an env var, accessible only to the operations team via SSH.
4. A Docker image built with `ENV DB_PASSWORD=secret123` in the Dockerfile, pushed to a private registry.
5. A Kubernetes pod that injects `SMTP_PASSWORD` from a Secret object into a container env var.
6. A developer's laptop running `docker inspect` on a shared development container that has production credentials as env vars.

<details>
<summary>Hint</summary>

Risk depends on two factors: how many people can access the secret, and how long the secret persists. A secret in a git repo persists forever and is accessible to everyone with repo access. A secret in a running container is accessible only while the container runs and only to those with Docker access.

</details>

### Part D: Design a Better Pattern

Based on the module README, describe the "file-based secret injection" pattern in your own words. Draw or describe the flow from secret storage to application consumption, covering:

1. Where the secret is stored (not in the environment).
2. How it enters the container (not as an env var).
3. How the application reads it (not from `process.env` or `os.environ`).
4. Why this pattern is more secure than environment variables.

<details>
<summary>Hint</summary>

The pattern involves mounting secrets as files in `/run/secrets/`, reading them from the filesystem, and keeping them in tmpfs so they never touch disk. The module README covers Docker Secrets and the file-mount approach.

</details>

## Success Criteria

- [ ] You listed four distinct methods for reading environment variables from a container.
- [ ] You explained three specific security risks of treating secrets like config.
- [ ] You correctly classified all six exposure scenarios by risk level.
- [ ] You described the file-based secret injection pattern with all four components.
- [ ] You can articulate why environment variables are appropriate for config but not for secrets.

## What You Should Understand After This Exercise

Environment variables are the right tool for configuration, but they are the wrong tool for secrets. The distinction is not about complexity or preference -- it is about the fundamental properties of the storage mechanism. Environment variables are visible to anyone with container or orchestrator access, they persist in logs and build artifacts, and they provide no audit trail. Secrets require a different approach: encrypted storage, restricted access, audit logging, and runtime injection that keeps the secret out of the process environment.
