# Exercise 04: Handle Secrets and Drift Detection

**Type:** Challenge
**Time:** 45 minutes
**Difficulty:** Medium-Hard

## Objective

Design a strategy for managing secrets in a GitOps workflow and configure
drift detection that reverts unauthorized changes. This exercise addresses
the two hardest problems in GitOps: secrets management and cluster state
integrity.

## Scenario

Your team is adopting GitOps with ArgoCD. Two problems have emerged:

1. **Secrets:** Developers want to store Kubernetes Secrets in Git for
   reproducibility, but plain-text secrets in Git are a security risk.
2. **Drift:** Someone ran `kubectl scale deployment api --replicas=10`
   directly on the cluster, bypassing Git. Nobody noticed for 3 days.

## Tasks

### Part A: Compare Secret Management Approaches

Evaluate three approaches to managing secrets in GitOps:

| Approach | How It Works | Security | Complexity |
|----------|-------------|----------|------------|
| Plain-text in Git | Store secrets as base64 in YAML | ? | Low |
| Sealed Secrets | Encrypt secrets with a cluster-specific key | ? | Medium |
| External Secrets Operator | Reference secrets from Vault/AWS SM | ? | High |

For each approach, explain the security implications and when you would
choose it.

<details>
<summary>Hint</summary>

- Plain-text: Anyone with Git access sees the secrets. Base64 is not
  encryption. Never use this.
- Sealed Secrets: Encrypted with a public key. Only the cluster's
  Sealed Secrets controller can decrypt. Safe to commit to Git.
- External Secrets Operator: Secrets live in an external system (Vault,
  AWS Secrets Manager). The operator fetches them at runtime. Secrets
  never touch Git.

</details>

### Part B: Implement Sealed Secrets

Write the workflow for creating and using a Sealed Secret:
1. Install the Sealed Secrets controller
2. Create a Sealed Secret from a plain-text secret
3. Deploy the Sealed Secret via ArgoCD
4. Verify the secret is available in the cluster

<details>
<summary>Hint</summary>

```bash
# Install controller
kubectl apply -f https://github.com/bitnami-labs/sealed-secrets/releases/latest/download/controller.yaml

# Create sealed secret
echo -n mypassword | kubectl create secret generic db-password \
  --dry-run=client --from-file=password=/dev/stdin -o yaml | \
  kubeseal --controller-namespace kube-system \
  --controller-name sealed-secrets \
  -o yaml > sealed-db-password.yaml

# The sealed-db-password.yaml is safe to commit to Git
```

</details>

### Part C: Configure Drift Detection

Write the ArgoCD configuration that:
1. Detects when someone makes manual changes to the cluster
2. Automatically reverts the changes to match Git
3. Sends a notification when drift is detected

<details>
<summary>Hint 1</summary>

Enable `selfHeal` in the sync policy:

```yaml
syncPolicy:
  automated:
    prune: true
    selfHeal: true
```

With `selfHeal: true`, ArgoCD checks the cluster every 3 minutes. If the
actual state differs from Git, it applies the Git state.

</details>

<details>
<summary>Hint 2</summary>

For notifications, use ArgoCD Notifications:

```yaml
metadata:
  annotations:
    notifications.argoproj.io/subscribe.on-sync-status-unknown.slack: infra-alerts
```

</details>

### Part D: Design the Escape Hatch

Self-heal reverts ALL manual changes, including emergency hotfixes. Design
a process that allows emergency changes while maintaining GitOps discipline.

<details>
<summary>Hint</summary>

Consider:
1. **Emergency branch:** Create a hotfix branch, push the change, merge
   to main. The hotfix is in Git within minutes.
2. **Pause self-heal:** Temporarily disable self-heal for the Application,
   make the change, then create a Git PR to match.
3. **ArgoCD ignore rules:** Configure ArgoCD to ignore certain fields
   (e.g., replica count) that might be adjusted manually for scaling.

```yaml
ignoreDifferences:
  - group: apps
    kind: Deployment
    jsonPointers:
      - /spec/replicas
```

</details>

## Success Criteria

- [ ] You can evaluate three secret management approaches with trade-offs
- [ ] You can create and deploy a Sealed Secret via ArgoCD
- [ ] ArgoCD is configured with self-heal to revert manual changes
- [ ] Notifications alert the team when drift is detected
- [ ] You have an escape hatch for emergency changes that maintains GitOps

## What You Should Understand After This Exercise

Secrets and drift are the two hardest problems in GitOps. Sealed Secrets
let you store encrypted secrets in Git safely. Self-heal reverts manual
changes automatically. The escape hatch (emergency branch, pause self-heal,
or ignore rules) ensures that emergency changes are possible without
abandoning GitOps discipline.
