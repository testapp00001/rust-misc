# Exercise 04: Implement a Security-Hardened Container Pipeline

**Type:** Challenge
**Time:** 45 minutes
**Difficulty:** Medium-Hard

## Objective

Build a complete CI/CD pipeline that automatically builds, scans, and deploys a container image with security gates. The pipeline must reject images that fail security checks. You will implement this as a shell script that can be integrated into any CI system (GitHub Actions, GitLab CI, Jenkins).

## Scenario

Your team needs an automated pipeline that enforces container security standards. The pipeline should:

1. Build the Docker image.
2. Scan it for vulnerabilities.
3. Verify security configuration (non-root, no secrets in layers).
4. Sign the image if all checks pass.
5. Fail the pipeline if any check fails.

You are given a sample application Dockerfile that has issues. The pipeline must catch them.

**Starting Dockerfile (has intentional issues):**

```dockerfile
FROM node:18

WORKDIR /app

COPY package*.json ./
RUN npm ci

COPY . .

EXPOSE 3000

CMD ["node", "server.js"]
```

**Starting docker-compose.yml (has intentional issues):**

```yaml
version: "3.8"

services:
  app:
    build: .
    ports:
      - "3000:3000"
    environment:
      SECRET_KEY: "hardcoded-secret-value"
```

## Tasks

### Part A: Fix the Application Files

Before building the pipeline, fix the Dockerfile and docker-compose.yml so they pass security checks:

1. The Dockerfile must use a non-root user.
2. The Dockerfile must use a pinned, minimal base image.
3. The docker-compose.yml must not contain hardcoded secrets.
4. The docker-compose.yml must include security hardening (cap_drop, read_only, resource limits).

<details>
<summary>Hint</summary>

Apply everything from Exercise 02. The Dockerfile should use `node:18-slim` or `node:18-alpine`, create a user with UID 1001, and switch to it. The compose file should use secrets or bind-mounted secret files.

</details>

### Part B: Create the Security Gate Script

Write a shell script called `security-gate.sh` that performs these checks and exits with a non-zero code if any check fails:

**Check 1: Build the image.**
```bash
docker build -t "$IMAGE_NAME" .
```

**Check 2: Scan for CRITICAL vulnerabilities.**
Use Trivy to scan the image. Fail if any CRITICAL vulnerabilities are found.
```bash
trivy image --exit-code 1 --severity CRITICAL "$IMAGE_NAME"
```

**Check 3: Verify non-root user.**
Run the image and check that `whoami` does not return `root`.
```bash
docker run --rm "$IMAGE_NAME" whoami
```

**Check 4: Check for secrets in image layers.**
Inspect the image history for strings that look like secrets (passwords, API keys, tokens).
```bash
docker history "$IMAGE_NAME" --no-trunc
```

**Check 5: Verify no sensitive files are baked into the image.**
Check that files like `.env`, `.git`, `node_modules/.cache` are not in the image.
```bash
docker run --rm "$IMAGE_NAME" sh -c 'ls -la /app/.env 2>/dev/null && echo "FAIL: .env found" || echo "OK"'
```

**Check 6: Verify read-only filesystem support.**
Start the container with `--read-only` and verify it starts successfully.

Your script should:
- Print a clear pass/fail status for each check.
- Exit with code 0 only if ALL checks pass.
- Exit with code 1 if ANY check fails.
- Print a summary at the end.

<details>
<summary>Hint 1: Script structure</summary>

Use a counter variable to track failures. Increment it on each failed check. At the end, exit with code 1 if the counter is greater than 0. Use color codes (green for pass, red for fail) for readability.

</details>

<details>
<summary>Hint 2: Secret detection</summary>

For a basic secret scan, use `docker history --no-trunc` and grep for patterns like `PASSWORD`, `SECRET`, `API_KEY`, `TOKEN`. This is a simple heuristic -- production tools like `trufflehog` or `detect-secrets` are more thorough.

</details>

### Part C: Create the Full Pipeline Script

Extend the security gate into a full pipeline script called `deploy-pipeline.sh` that:

1. Runs all security gate checks.
2. If all checks pass, tags the image with a version and a SHA.
3. Optionally signs the image with Cosign (if available).
4. Prints a deployment-ready message.
5. If any check fails, prints a clear error and exits.

The script should accept arguments:
```bash
./deploy-pipeline.sh --image my-app --version v1.0.0 --registry myregistry.io
```

<details>
<summary>Hint</summary>

Use `getopts` or manual argument parsing for the flags. For signing, check if `cosign` is installed with `command -v cosign`. If it is, generate a key pair (if not present) and sign the image. If it is not installed, print a warning and skip signing.

</details>

### Part D: Test the Pipeline

1. Run the pipeline against the fixed Dockerfile:

```bash
chmod +x deploy-pipeline.sh
./deploy-pipeline.sh --image my-app --version v1.0.0
```

2. Verify that it passes all checks.

3. Introduce a deliberate failure (e.g., add `USER root` back to the Dockerfile) and verify that the pipeline catches it and exits with code 1.

---

## Success Criteria

- [ ] The pipeline script runs all six security checks and reports pass/fail for each.
- [ ] The pipeline exits with code 0 when all checks pass.
- [ ] The pipeline exits with code 1 when any check fails.
- [ ] The pipeline tags the image with version and SHA on success.
- [ ] The pipeline prints a clear error message identifying which check failed.

## What You Should Understand After This Exercise

Security is only effective when it is automated and enforced. A manual security review is error-prone and inconsistent. By building security checks into the CI/CD pipeline, you ensure that no insecure image reaches production. The pipeline acts as a gatekeeper -- it is the single point of enforcement for your container security policy.
