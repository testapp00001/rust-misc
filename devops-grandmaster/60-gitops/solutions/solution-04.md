# Solution 04: Handle Secrets and Drift Detection

## Part A: Compare Secret Management Approaches

| Approach | Security | Complexity | When to Use |
|----------|----------|------------|-------------|
| **Plain-text in Git** | Terrible. Anyone with Git read access sees secrets. Base64 is encoding, not encryption. | Very low | Never in production. Only for local development with fake secrets. |
| **Sealed Secrets** | Good. Secrets are encrypted with a public key. Only the cluster's controller can decrypt. Safe to commit to Git. | Medium | Small to medium teams. Secrets are static (DB passwords, API keys). |
| **External Secrets Operator** | Excellent. Secrets live in Vault, AWS Secrets Manager, or similar. The operator fetches them at runtime. Secrets never touch Git. | High | Large teams, compliance requirements, secrets that rotate frequently. |

### Why This Matters

The fundamental tension in GitOps is: everything should be in Git, but
secrets should not be in Git. Sealed Secrets solve this by encrypting
secrets so they are safe to commit. External Secrets Operator avoids
putting secrets in Git entirely.

## Part B: Implement Sealed Secrets

### Step 1: Install the Controller

```bash
# Install Sealed Secrets controller
kubectl apply -f https://github.com/bitnami-labs/sealed-secrets/releases/latest/download/controller.yaml

# Wait for it to be ready
kubectl wait --for=condition=available deployment/sealed-secrets-controller \
  -n kube-system --timeout=120s

# Install the kubeseal CLI
curl -LO https://github.com/bitnami-labs/sealed-secrets/releases/latest/download/kubeseal-linux-amd64
chmod +x kubeseal-linux-amd64
sudo mv kubeseal-linux-amd64 /usr/local/bin/kubeseal
```

### Step 2: Create a Sealed Secret

```bash
# Create the plain-text secret (dry-run, never applied to cluster)
echo -n "my-secure-password" | kubectl create secret generic db-password \
  --dry-run=client \
  --from-file=password=/dev/stdin \
  -o yaml > /tmp/db-password.yaml

# Seal it with the cluster's public key
kubeseal \
  --controller-namespace kube-system \
  --controller-name sealed-secrets \
  --format yaml \
  < /tmp/db-password.yaml \
  > sealed-db-password.yaml

# Clean up the plain-text secret
rm /tmp/db-password.yaml
```

### Step 3: The Sealed Secret (Safe to Commit)

```yaml
# sealed-db-password.yaml (safe to commit to Git)
apiVersion: bitnami.com/v1alpha1
kind: SealedSecret
metadata:
  name: db-password
  namespace: production
spec:
  encryptedData:
    password: AgBy3i4OJSWK+PiTySYZZA9rO43cGDEqAy+nGyPdjJwTRE5lQ2v0Z...
  template:
    metadata:
      name: db-password
      namespace: production
    type: Opaque
```

### Step 4: Deploy via ArgoCD

```yaml
# In the ArgoCD Application, add the sealed secret to the resources:
# base/secrets/kustomization.yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization
resources:
  - sealed-db-password.yaml
```

ArgoCD syncs the SealedSecret to the cluster. The Sealed Secrets
controller detects it, decrypts it using the cluster's private key, and
creates the actual Kubernetes Secret.

### Step 5: Verify

```bash
# Check the SealedSecret was created
kubectl get sealedsecret db-password -n production

# Check the Secret was created by the controller
kubectl get secret db-password -n production

# Verify the secret value
kubectl get secret db-password -n production -o jsonpath='{.data.password}' | base64 -d
# Output: my-secure-password
```

### Why This Works

The Sealed Secrets controller generates a public/private key pair. The
public key is fetched by `kubeseal` to encrypt secrets locally. The
private key stays in the cluster and is used by the controller to decrypt.
Only someone with cluster access (not just Git access) can read the
secrets.

## Part C: Configure Drift Detection

```yaml
# argocd-application-with-selfheal.yaml
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: api-service
  namespace: argocd
  annotations:
    notifications.argoproj.io/subscribe.on-sync-status-unknown.slack: infra-alerts
    notifications.argoproj.io/subscribe.on-sync-failed.slack: infra-alerts
    notifications.argoproj.io/subscribe.on-health-degraded.slack: infra-alerts
spec:
  project: default
  source:
    repoURL: https://github.com/myorg/gitops-repo.git
    targetRevision: main
    path: overlays/production/api
  destination:
    server: https://kubernetes.default.svc
    namespace: production
  syncPolicy:
    automated:
      prune: true
      selfHeal: true
    syncOptions:
      - CreateNamespace=true
```

### How Self-Heal Works

```
ArgoCD reconciliation loop (every 3 minutes):

1. Read desired state from Git (main branch, overlays/production/api/)
2. Read actual state from cluster (kubectl get ...)
3. Diff the two states
4. If diff exists:
   a. Log the drift
   b. Apply the Git state to the cluster
   c. Send notification (if configured)
5. Wait 3 minutes, repeat
```

When someone runs `kubectl scale deployment api --replicas=10` but Git
says `replicas: 3`:

1. ArgoCD detects the diff (10 replicas vs 3)
2. ArgoCD scales back to 3
3. ArgoCD logs: "Deployment api: spec.replicas changed from 10 to 3"
4. Slack notification: "api-service sync status: OutOfSync"

### Why This Works

The controller is the source of truth enforcer. It does not care who
made the change or why. If the cluster state differs from Git, it
reverts. This is the core principle of GitOps: Git is the single source
of truth.

## Part D: Design the Escape Hatch

### Option 1: Emergency Branch (Recommended)

```bash
# During an incident, create a hotfix branch
git checkout -b hotfix/api-scale-up
cd overlays/production/api
kustomize edit set replicas 10
git add .
git commit -m "emergency: scale api to 10 replicas for incident INC-1234"
git push origin hotfix/api-scale-up

# Create a PR for immediate review
gh pr create --title "Emergency: Scale API for INC-1234" \
  --body "Incident INC-1234: API is overloaded. Scaling to 10 replicas." \
  --reviewer team-lead

# If team-lead approves immediately:
gh pr merge --merge
```

**Time:** 2-5 minutes. The change is in Git, audited, and reviewed.

### Option 2: Pause Self-Heal Temporarily

```bash
# Disable self-heal for the Application
kubectl patch application api-service -n argocd --type merge \
  -p '{"spec":{"syncPolicy":{"automated":{"selfHeal":false}}}}'

# Make the emergency change
kubectl scale deployment api --replicas=10

# Create a Git PR to match
# ... (same as Option 1)

# Re-enable self-heal after the PR is merged
kubectl patch application api-service -n argocd --type merge \
  -p '{"spec":{"syncPolicy":{"automated":{"selfHeal":true}}}}'
```

**Risk:** If you forget to re-enable self-heal, drift accumulates.

### Option 3: Ignore Specific Fields

```yaml
# For fields that are frequently adjusted manually (e.g., replica count
# for horizontal pod autoscaler)
spec:
  ignoreDifferences:
    - group: apps
      kind: Deployment
      name: api
      jsonPointers:
        - /spec/replicas
```

**When to use:** When a Horizontal Pod Autoscaler (HPA) manages replica
count. The HPA changes replicas based on load, and you do not want
ArgoCD to revert it.

**Risk:** If someone manually scales the deployment (not via HPA), ArgoCD
will not revert it.

### Why This Works

The escape hatch balances GitOps discipline with operational reality.
Emergency branches keep changes in Git. Pausing self-heal is a manual
override with a clear re-enable step. Ignore rules handle legitimate
manual changes (HPA). The key is that every escape hatch has a path
back to GitOps discipline.

## Common Mistakes to Avoid

- **Storing plain-text secrets in Git.** Base64 is not encryption.
  Anyone with Git access can decode the secret. Use Sealed Secrets or
  External Secrets Operator.
- **Self-healing replica counts with HPA.** If HPA scales the deployment
  to 10 replicas, self-heal reverts it to 3. Use `ignoreDifferences`
  for fields managed by controllers.
- **No notification on drift.** Without notifications, drift happens
  silently. Always configure notifications for sync failures and health
  degradation.
- **Forgetting to re-enable self-heal.** If you pause self-heal for an
  emergency, set a reminder to re-enable it. Otherwise, drift accumulates.

## Key Takeaway

Secrets and drift are the two hardest problems in GitOps. Sealed Secrets
encrypt secrets so they are safe in Git. Self-heal reverts manual changes
automatically. The escape hatch (emergency branches, pause self-heal,
ignore rules) ensures that emergency changes are possible without
abandoning GitOps discipline. The goal is not to prevent all manual
changes, but to ensure every change has a path back to Git.
