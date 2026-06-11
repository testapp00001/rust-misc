# Exercise 01: Anatomy of a Broken Pipeline

**Type:** Conceptual
**Time:** 15 minutes
**Difficulty:** Easy

## Objective

Analyze a CI/CD pipeline configuration to identify missing stages, misconfigured
dependencies, and anti-patterns. This exercise trains you to evaluate pipeline
design critically -- a skill you need before building your own.

## Scenario

A junior developer on your team submits this GitHub Actions pipeline for review.
The team's Rust web service needs to go from code push to production deployment.

```yaml
name: Deploy to Production

on:
  push:
    branches: [main]

jobs:
  build-and-deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Build
        run: cargo build --release

      - name: Deploy to production
        run: |
          docker build -t myapp:latest .
          docker push registry.example.com/myapp:latest
          kubectl set image deployment/myapp myapp=registry.example.com/myapp:latest
```

## Tasks

### Part A: List What Is Missing

This pipeline skips critical stages. List every stage that a production-grade
pipeline should include but is absent from this configuration.

<details>
<summary>Hint</summary>

Think about the full path from code to production:
- What validates correctness?
- What validates security?
- What validates the artifact works in a real environment?
- What prevents bad code from reaching users?

</details>

### Part B: Identify the Anti-Patterns

Even for the stages that are present, the implementation has problems.
List every anti-pattern or mistake you can find.

<details>
<summary>Hint</summary>

Consider:
- The `latest` tag and what it means for reproducibility
- Whether there are any gates between build and deploy
- Whether the deployment is safe (can it be rolled back?)
- Whether this runs on every push to main without any human approval

</details>

### Part C: Draw the Correct Pipeline

Using ASCII art, draw the ideal pipeline flow for this service. Show which
stages run in parallel and which must run sequentially. Include all gates.

<details>
<summary>Hint</summary>

A good pipeline has these properties:
- Independent stages run in parallel (unit tests, integration tests, security scans)
- A quality gate blocks deployment unless all checks pass
- Deployment goes to staging first, then production with a manual approval
- Artifacts are versioned with commit SHA, not `latest`

</details>

### Part D: Explain the Consequences

For each anti-pattern you identified in Part B, describe a specific scenario
where it would cause a production incident. Be concrete.

<details>
<summary>Hint</summary>

For example: "Using the `latest` tag means that if you push two commits in
quick succession, the first deployment might pull the second commit's image
due to caching or race conditions."

</details>

## Success Criteria

- [ ] You identified at least 4 missing pipeline stages
- [ ] You found at least 4 anti-patterns in the existing configuration
- [ ] Your ASCII diagram shows parallel stages and quality gates
- [ ] You described concrete failure scenarios for each anti-pattern
- [ ] You understand why each stage exists and what risk it mitigates

## What You Should Understand After This Exercise

A CI/CD pipeline is not just "build and deploy." Each stage exists to catch a
specific category of defect before it reaches users. Skipping stages does not
save time -- it borrows time from your future self who will debug production
incidents at 3 AM.
