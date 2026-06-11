# Exercise 04: Fix the Production Pipeline

**Type:** Challenge
**Time:** 45 minutes
**Difficulty:** Medium-Hard

## Objective

Diagnose and fix a real-world pipeline that has security holes, performance
bottlenecks, and reliability issues. This exercise simulates the work of
reviewing and improving an existing pipeline in a production environment.

## Scenario

Your team's pipeline has been "working" for six months, but incidents are
increasing. The VP of Engineering asks you to audit and fix the pipeline.
Here is the current configuration:

```yaml
name: CI/CD

on: [push, pull_request]

env:
  REGISTRY: docker.io
  IMAGE: mycompany/myapp
  KUBECONFIG_DATA: ${{ secrets.KUBECONFIG }}

jobs:
  everything:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install dependencies
        run: |
          apt-get update
          apt-get install -y postgresql-client curl

      - name: Build
        run: cargo build --release

      - name: Test
        run: cargo test
        env:
          DATABASE_URL: postgres://postgres:password@localhost:5432/testdb

      - name: Lint
        run: cargo clippy -- -D warnings

      - name: Build Docker image
        run: |
          docker build -t $IMAGE:latest .
          echo "${{ secrets.DOCKER_PASSWORD }}" | docker login -u "${{ secrets.DOCKER_USERNAME }}" --password-stdin
          docker push $IMAGE:latest

      - name: Deploy
        if: github.ref == 'refs/heads/main'
        run: |
          echo "$KUBECONFIG_DATA" | base64 -d > /tmp/kubeconfig
          export KUBECONFIG=/tmp/kubeconfig
          kubectl set image deployment/myapp myapp=$IMAGE:latest
          kubectl rollout status deployment/myapp --timeout=60s
          rm /tmp/kubeconfig
```

## Tasks

### Part A: Security Audit

List every security vulnerability in this pipeline. For each one, explain
the risk and how an attacker could exploit it.

<details>
<summary>Hint 1</summary>

Look at how secrets are handled. Are they logged? Are they stored in
temporary files? Is the Docker login secure?

</details>

<details>
<summary>Hint 2</summary>

Consider: Does this pipeline run on pull requests from forks? What happens
if a malicious PR modifies the workflow file to exfiltrate secrets?

</details>

### Part B: Performance Audit

This pipeline takes 18 minutes. Identify at least three things that are
wasting time and propose fixes with estimated time savings.

<details>
<summary>Hint</summary>

Consider:
- Is anything running sequentially that could run in parallel?
- Are dependencies being reinstalled on every run?
- Is the Docker build using a cache?
- Are there stages that do not need to run on every push?

</details>

### Part C: Reliability Audit

This pipeline fails randomly about 10% of the time with non-deterministic
errors. Identify the reliability problems and propose fixes.

<details>
<summary>Hint</summary>

Think about:
- Is there a database service running for the tests, or are they relying
  on a host-level PostgreSQL that might not be ready?
- Does the deployment have a rollback strategy?
- What happens if the `kubectl rollout status` times out?
- Are there retry mechanisms for flaky operations?

</details>

### Part D: Rewrite the Pipeline

Write a corrected version of this pipeline that fixes all the issues you
identified. Your rewrite should:
- Fix all security vulnerabilities
- Reduce execution time to under 8 minutes
- Handle failures gracefully with rollback
- Use proper image tagging (not `latest`)

<details>
<summary>Hint</summary>

Key changes to make:
1. Split the monolithic job into parallel jobs with a quality gate
2. Use `actions/cache` for Cargo dependencies
3. Use Docker layer caching with `cache-from`/`cache-to`
4. Tag images with `${{ github.sha }}` instead of `latest`
5. Use GitHub environments for deployment approval
6. Add a rollback step if deployment health check fails
7. Use service containers for PostgreSQL instead of host-level install

</details>

## Success Criteria

- [ ] You identified at least 5 security vulnerabilities
- [ ] You found at least 3 performance bottlenecks with time estimates
- [ ] You identified at least 3 reliability issues
- [ ] Your rewritten pipeline fixes all identified issues
- [ ] Your rewritten pipeline uses parallel stages and proper caching

## What You Should Understand After This Exercise

A "working" pipeline is not the same as a good pipeline. Security holes,
performance bottlenecks, and reliability gaps accumulate silently until they
cause a production incident. Regular pipeline audits are as important as
code reviews.
