# Exercise 05: Full System Deployment

**Type:** Integration
**Time:** 60 minutes
**Difficulty:** Expert

## Objective

Integrate everything from Exercises 01-04 into a single, deployable
system. You will write a complete GitOps pipeline that deploys the
payment-api across two regions with all supporting infrastructure:
Kubernetes manifests, observability stack, backup procedures, chaos
experiments, and a production readiness checklist. This is the final
proof that you can build a system that does not break.

## Scenario

Your team has completed all the individual components. Now the CTO
asks: "Can you deploy the entire system end-to-end with a single
`git push`?" You must:

1. Structure the GitOps repository for multi-region deployment
2. Write the CI/CD pipeline that validates, tests, and deploys
3. Create a production readiness checklist that gates deployment
4. Write a smoke test suite that validates the deployment
5. Document the system for the next engineer who inherits it

## Tasks

### Part A: GitOps Repository Structure

Design the directory structure for a GitOps repository that manages:

- Two regions (us-east-1, eu-west-1)
- Application manifests (deployment, service, ingress, HPA, PDB)
- Infrastructure manifests (Prometheus, Grafana, Loki, Jaeger)
- Database manifests (primary, replica, backup CronJob)
- Chaos experiment manifests
- Environment-specific overrides (staging vs. production)

Draw the full directory tree with annotations explaining what each
directory contains and why.

<details>
<summary>Hint</summary>

Use a tool like Kustomize or Helm for environment-specific overrides.
The base manifests live in a `base/` directory. Each environment and
region overlays on top. The directory structure should make it obvious
what gets deployed where.

</details>

### Part B: CI/CD Pipeline

Write a GitHub Actions workflow that:

1. Validates all Kubernetes manifests (lint with `kubeval` or `kubeconform`)
2. Runs unit tests for the application
3. Builds and pushes the container image
4. Deploys to staging automatically
5. Runs smoke tests against staging
6. Deploys to production with manual approval
7. Runs smoke tests against production
8. Automatically rolls back if smoke tests fail

The pipeline must handle both regions and include a deployment strategy
(canary or blue-green).

<details>
<summary>Hint</summary>

The pipeline has three stages: build, staging, production. Staging is
automatic. Production requires manual approval. The smoke tests are
the gate -- if they fail, the pipeline rolls back. Use Argo Rollouts
for canary deployments with automatic rollback on error rate increase.

</details>

### Part C: Production Readiness Checklist

Write a checklist that must be completed before any production
deployment. Organize it into categories:

1. **Reliability** -- What must be true for the system to stay up?
2. **Observability** -- Can you see what the system is doing?
3. **Security** -- Is the system hardened against attacks?
4. **Recovery** -- Can you recover from failure?
5. **Scaling** -- Can the system handle 10x traffic?
6. **Operations** -- Can the on-call engineer manage it?

Each item must be a yes/no question with a link to the evidence
(monitoring dashboard, test result, manifest, or documentation).

<details>
<summary>Hint</summary>

The checklist is not a formality -- it is a gate. If any item is "no,"
the deployment is blocked. The checklist should be automated where
possible (e.g., "Are all pods healthy?" can be checked with `kubectl`).
Manual items should have a designated owner.

</details>

### Part D: Smoke Test Suite

Write a smoke test script that validates a production deployment. The
tests must verify:

1. **Health** -- All pods are running and ready
2. **Connectivity** -- The API responds to requests
3. **Functionality** -- A test payment succeeds end-to-end
4. **Performance** -- p99 latency is below 200ms
5. **Observability** -- Metrics are being scraped, logs are flowing
6. **Failover** -- The system survives a pod kill during the test

The script must exit with a clear pass/fail status and produce a
human-readable report.

<details>
<summary>Hint</summary>

Use `kubectl` for health checks, `curl` for connectivity, and a test
payment API call for functionality. For performance, run a quick load
test with `hey` or `k6` and check the p99. For observability, query
Prometheus and Loki APIs. For failover, kill a pod and verify recovery.

</details>

### Part E: System Documentation

Write the README for the production system. It must answer these
questions for an engineer who joins the team on their first day:

1. What does the system do? (one paragraph)
2. How is it deployed? (architecture diagram + deployment process)
3. How do you debug an incident? (runbook links + key dashboards)
4. How do you scale it? (scaling triggers + manual scaling commands)
5. How do you back it up? (backup schedule + restore procedure)
6. How do you update it? (deployment process + rollback procedure)
7. Who do you page? (on-call rotation + escalation path)

<details>
<summary>Hint</summary>

The README is not for you -- it is for the engineer who is paged at
3 AM and has never seen the system before. Every link must work. Every
command must be copy-pasteable. Every assumption must be stated
explicitly. If the README does not answer a question, the question
will be answered by guesswork at 3 AM.

</details>

## Success Criteria

- [ ] The GitOps directory structure supports multi-region, multi-environment deployment
- [ ] The CI/CD pipeline validates, tests, deploys, and auto-rolls back
- [ ] The production readiness checklist has at least 20 items across all 6 categories
- [ ] The smoke test script passes all 6 checks and produces a readable report
- [ ] The README answers all 7 questions with accurate links and commands
- [ ] All manifests are valid and deployable with `kubectl apply -k .` (Kustomize) or `helm install`

## What You Should Understand After This Exercise

Building an unbreakable system is not about any single technology or
technique. It is about the integration of all the pieces: infrastructure
that scales, deployments that do not break, monitoring that detects
problems, backups that actually restore, chaos tests that prove
resilience, and documentation that enables anyone to operate it. The
final system is only as strong as its weakest link. This exercise
ensures there are no weak links.
