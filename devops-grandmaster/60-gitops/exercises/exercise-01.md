# Exercise 01: Push vs Pull Deployment

**Type:** Conceptual
**Time:** 15 minutes
**Difficulty:** Easy

## Objective

Compare push-based deployment (CI pipeline applies changes to the cluster)
with pull-based deployment (a controller in the cluster pulls changes from
Git). This exercise trains you to understand why GitOps uses the pull model
and what problems it solves.

## Scenario

Your team currently deploys using a GitHub Actions pipeline that runs
`kubectl apply` directly to the production cluster:

```yaml
# .github/workflows/deploy.yaml
name: Deploy
on:
  push:
    branches: [main]
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Setup kubeconfig
        run: |
          echo "${{ secrets.KUBECONFIG }}" | base64 -d > ~/.kube/config
      - name: Deploy
        run: kubectl apply -f k8s/
```

A colleague suggests switching to ArgoCD (pull-based). The VP of Engineering
asks you to explain the difference.

## Tasks

### Part A: Draw Both Architectures

Draw ASCII diagrams showing:
1. Push-based: CI pipeline pushes changes to the cluster
2. Pull-based: Controller in the cluster pulls changes from Git

Label the direction of network connections and where credentials live.

<details>
<summary>Hint</summary>

In push-based, the CI system has cluster credentials and initiates
connections to the cluster. In pull-based, the cluster has Git credentials
and initiates connections to the Git repo.

</details>

### Part B: Security Comparison

Compare the security implications of each approach. Consider:
- Where are credentials stored?
- What happens if the CI system is compromised?
- What happens if the Git repo is compromised?
- Who can access the cluster?

<details>
<summary>Hint</summary>

In push-based, the CI system needs `kubeconfig` with write access. If
the CI system is compromised, the attacker has cluster access. In
pull-based, the cluster does not expose any credentials to the CI system.
The controller runs inside the cluster and only needs Git read access.

</details>

### Part C: Drift Detection

Explain what "configuration drift" is and how each approach handles it.
Consider: what happens if someone runs `kubectl edit` manually on the
cluster?

<details>
<summary>Hint</summary>

Drift is when the actual cluster state diverges from the desired state
in Git. In push-based, nobody detects drift -- the CI pipeline only
runs on Git pushes. In pull-based, the controller continuously compares
the cluster state to Git and can automatically revert manual changes.

</details>

### Part D: Rollback Comparison

Compare how each approach handles rollback:
1. Push-based rollback requires what steps?
2. Pull-based rollback requires what steps?

<details>
<summary>Hint</summary>

Push-based: create a new commit reverting the change, push, wait for CI
to run, wait for deployment. Pull-based: `git revert`, push, the
controller automatically applies the revert. Or even faster: ArgoCD
can roll back to a previous revision directly from the UI without a
new commit.

</details>

## Success Criteria

- [ ] You can draw both push and pull deployment architectures
- [ ] You can explain the security advantages of pull-based deployment
- [ ] You can define configuration drift and explain how GitOps detects it
- [ ] You can compare rollback processes for both approaches
- [ ] You understand why GitOps uses the pull model

## What You Should Understand After This Exercise

GitOps uses the pull model because it is more secure (no cluster
credentials in CI), supports drift detection (continuous reconciliation),
and simplifies rollback (git revert). The trade-off is that you need
a controller running in the cluster, but this is a small cost for the
security and reliability benefits.
