# Solution 01: Push vs Pull Deployment

## Part A: Draw Both Architectures

### Push-Based Deployment

```
  Developer                CI System              Kubernetes Cluster
     |                        |                         |
     |  git push              |                         |
     |----------------------->|                         |
     |                        |  kubectl apply          |
     |                        |  (kubeconfig)           |
     |                        |------------------------>|
     |                        |                         |
     |                        |  cluster credentials    |
     |                        |  stored in CI secrets   |
```

### Pull-Based Deployment (GitOps)

```
  Developer                Git Repo              Kubernetes Cluster
     |                        |                         |
     |  git push              |                         |
     |----------------------->|                         |
     |                        |                         |
     |                        |  git pull (poll/webhook)|
     |                        |<------------------------|
     |                        |                         |
     |                        |  kubectl apply          |
     |                        |------------------------>|
     |                        |                         |
     |                        |  git credentials        |
     |                        |  stored in cluster      |
```

### Why This Matters

The direction of the connection is the fundamental difference. In
push-based, the external system (CI) connects to the cluster. In
pull-based, the cluster connects to the external system (Git). This
reversal has profound security implications.

## Part B: Security Comparison

| Aspect | Push-Based | Pull-Based |
|--------|-----------|------------|
| **Credential location** | CI system has `kubeconfig` | Cluster has Git credentials |
| **CI compromise impact** | Attacker gets cluster access | Attacker can only push to Git |
| **Git compromise impact** | Attacker can read manifests | Attacker can deploy to cluster |
| **Cluster access surface** | CI system + anyone with CI access | Only the controller in the cluster |
| **Credential scope** | Full cluster access (typically) | Read-only Git access (typically) |
| **Audit trail** | CI logs (can be tampered with) | Git history (immutable) |

### Key Security Insight

In push-based, the CI system is a high-value target because it has
cluster credentials. A CI compromise (malicious PR, compromised action)
gives the attacker direct cluster access.

In pull-based, the CI system never touches the cluster. A CI compromise
means the attacker can push malicious code to Git, but ArgoCD's sync
policy (manual approval for production) provides a second gate.

The cluster controller only needs read access to Git. Even if Git is
compromised, the attacker can only deploy what is in the repo, not
execute arbitrary commands on the cluster.

## Part C: Drift Detection

**Configuration drift** is when the actual cluster state diverges from the
desired state in Git. Examples:
- Someone runs `kubectl scale deployment api --replicas=10` (Git says 3)
- Someone edits a ConfigMap directly (`kubectl edit`)
- A controller auto-scales pods, changing the replica count

| Approach | Drift Detection | Drift Correction |
|----------|----------------|------------------|
| **Push-based** | None. CI only runs on Git pushes. | Manual: someone notices and fixes it. |
| **Pull-based** | Continuous. Controller compares Git to cluster every 3 minutes. | Automatic with `selfHeal: true`. Controller reverts manual changes. |

### Why This Works

The pull-based controller is a reconciliation loop:

```
1. Read desired state from Git
2. Read actual state from cluster
3. Diff the two
4. Apply the diff (create, update, or delete)
5. Wait, repeat
```

This loop runs continuously, so drift is detected and corrected within
minutes. In push-based, drift accumulates silently until someone notices.

## Part D: Rollback Comparison

### Push-Based Rollback

1. Identify the previous commit that was working
2. Create a revert commit: `git revert HEAD`
3. Push the revert commit
4. Wait for CI pipeline to run (build, test, scan, deploy)
5. Wait for deployment to complete

**Time:** 5-15 minutes (CI pipeline duration)

### Pull-Based Rollback

**Option 1: Git revert**
1. Create a revert commit: `git revert HEAD`
2. Push the revert commit
3. ArgoCD detects the change and syncs automatically

**Time:** 1-4 minutes (ArgoCD sync interval + apply time)

**Option 2: ArgoCD UI rollback**
1. Open ArgoCD UI
2. Click on the Application
3. Click "History and Rollback"
4. Select the previous revision
5. Click "Rollback"

**Time:** < 1 minute (no Git operation needed)

### Why This Works

ArgoCD stores the full history of every sync. Rollback to any previous
revision is a single click. The rollback is applied immediately, without
waiting for a CI pipeline. This is significantly faster than push-based
rollback.

## Common Mistakes to Avoid

- **Assuming pull-based is always better.** Pull-based requires a
  controller running in the cluster, which adds operational complexity.
  For small teams with simple deployments, push-based may be sufficient.
- **Not considering network access.** Pull-based requires the cluster
  to have outbound access to Git. In air-gapped environments, this may
  not be possible.
- **Ignoring the CI system's role.** Even in GitOps, CI still builds and
  tests code. The difference is that CI does not deploy -- it only
  produces artifacts and updates Git.

## Key Takeaway

The pull model is the foundation of GitOps. By reversing the direction of
the deployment connection (cluster pulls from Git instead of CI pushing
to cluster), you get better security (no cluster credentials in CI),
drift detection (continuous reconciliation), and faster rollback (ArgoCD
history). The trade-off is operational complexity, but for production
systems, the benefits far outweigh the costs.
