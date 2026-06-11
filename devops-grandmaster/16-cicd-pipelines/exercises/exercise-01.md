# Exercise 01: CI vs CD and Deployment Pipelines

**Type:** Conceptual
**Time:** 15 minutes
**Difficulty:** Easy

## Objective

Understand the difference between Continuous Integration, Continuous Delivery, and Continuous Deployment, and identify the stages that belong in a container deployment pipeline.

## Scenario

Your team at a growing startup has been deploying manually for two years. The CTO has mandated a move to automated CI/CD. Before writing any pipeline code, the team needs to agree on terminology, pipeline stages, and responsibilities. You have been asked to draft the foundational document that will guide the implementation.

## Tasks

### Part A: Define the Three CI/CD Concepts

Write a one-paragraph definition for each of the following. Your definitions must include what triggers the process, what the automated steps are, and what happens at the end.

1. **Continuous Integration (CI)**
2. **Continuous Delivery (CDelivery)**
3. **Continuous Deployment (CDeployment)**

For each definition, include one concrete example of when it would fail or produce a bad outcome if done manually.

<details>
<summary>Hint</summary>

Think about the difference between "we merged code" (CI), "we are ready to deploy" (CDelivery), and "it is live" (CDeployment). The trigger and the final action differ for each.

</details>

### Part B: Identify Pipeline Stages

A container-based application pipeline has the following candidate stages. Classify each stage into one of these categories: **Build**, **Test**, **Security**, **Publish**, **Deploy**, or **Verify**.

| Stage | Category |
|-------|----------|
| `docker build -t myapp:$SHA .` | ? |
| `npm test` inside the container | ? |
| `trivy image myapp:$SHA` | ? |
| `docker push registry.example.com/myapp:$SHA` | ? |
| `kubectl set image deployment/myapp myapp=registry.example.com/myapp:$SHA` | ? |
| `curl -f http://staging.example.com/health` | ? |
| `docker compose run --rm app npm run lint` | ? |
| `cosign sign registry.example.com/myapp:$SHA` | ? |
| `helm upgrade myapp ./chart --set image.tag=$SHA` | ? |
| `docker buildx build --platform linux/amd64,linux/arm64` | ? |

<details>
<summary>Hint</summary>

Consider what each stage accomplishes: producing an artifact (Build), validating correctness (Test), checking for vulnerabilities or signing (Security), making the artifact available (Publish), releasing to an environment (Deploy), or confirming the release works (Verify).

</details>

### Part C: Pipeline Failure Scenarios

For each scenario below, identify which pipeline stage should catch the problem. If no existing stage catches it, propose a new stage.

1. A developer pushes code that compiles but returns a non-zero exit code from the test suite.
2. A Docker image contains a known CVE with severity "Critical."
3. The staging environment is unreachable after deployment.
4. A developer accidentally commits a database migration that drops a production table.
5. The built image works on amd64 but crashes on arm64 nodes in the Kubernetes cluster.

<details>
<summary>Hint</summary>

Not all problems are caught by the same stage. Some are caught by tests, some by security scans, some by health checks, and some require entirely new safeguards like migration review or platform-specific testing.

</details>

### Part D: Map the Flow

Draw (in text) the pipeline flow for a single commit that goes from a developer's `git push` all the way to production. Your flow must include:

- At least 6 stages
- At least 1 parallel branch (two stages that can run simultaneously)
- At least 1 decision point (a gate that can pass or fail)
- A rollback path

Use arrow notation like `Stage A -> Stage B` or draw an ASCII diagram.

<details>
<summary>Hint</summary>

Linting and unit testing can often run in parallel. Security scanning can overlap with integration testing. A decision point might be "all tests pass?" or "no critical CVEs?"

</details>

## Success Criteria

- [ ] You can explain the difference between CI, Continuous Delivery, and Continuous Deployment in your own words.
- [ ] You correctly classified all 10 pipeline stages into their categories.
- [ ] You identified the appropriate pipeline stage (or proposed a new one) for all 5 failure scenarios.
- [ ] Your pipeline flow diagram includes parallel stages, a decision gate, and a rollback path.
- [ ] You can articulate why automated pipelines are superior to manual deployment scripts.

## What You Should Understand After This Exercise

CI/CD is not a single thing -- it is a chain of automated stages, each with a specific responsibility. CI catches broken code early by building and testing on every commit. CDelivery ensures that passing code is always in a deployable state. CDeployment removes the human gate and releases automatically. A well-designed pipeline has clear stage boundaries, parallelism where possible, decision gates that can halt a bad release, and rollback mechanisms when things go wrong in production.
