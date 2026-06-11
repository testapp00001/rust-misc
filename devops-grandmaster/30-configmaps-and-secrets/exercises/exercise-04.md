# Exercise 04: Configuration Management Across Environments

**Type:** Challenge
**Time:** 45 minutes
**Difficulty:** Advanced

## Objective

Design a configuration management strategy for an application that runs in
three environments (dev, staging, production) using ConfigMaps, Secrets, and
Kubernetes-native patterns like immutable resources and hash-triggered
restarts.

---

## Background

Your team maintains an e-commerce platform with the following environments:

| Environment | DB Host | Log Level | Replicas | Feature Flags | Secret Source |
|-------------|---------|-----------|----------|---------------|---------------|
| Dev | localhost | debug | 1 | all enabled | Literal values |
| Staging | staging-db.internal | info | 2 | new-checkout: true | Literal values |
| Production | prod-db.internal | warn | 5 | new-checkout: false | External vault |

The platform has 3 configuration concerns:

1. **Application config** (non-sensitive): log level, feature flags, DB host.
2. **Database credentials** (sensitive): username and password.
3. **TLS certificate** (sensitive): for the ingress.

---

## Tasks

### Task 1: Environment-Specific ConfigMaps

Create a ConfigMap for each environment. Each ConfigMap must have:

- `DB_HOST` key
- `LOG_LEVEL` key
- `FEATURE_FLAG_NEW_CHECKOUT` key

Name them `app-config-dev`, `app-config-staging`, `app-config-prod`.

Write all three YAML manifests. Identify what is shared across all three and
what differs.

<details>
<summary>Hint 1: Naming convention</summary>

Use a consistent naming pattern: `app-config-{env}`. This lets you reference
the correct ConfigMap in each environment's Deployment by interpolating the
environment name into the ConfigMap reference. In raw YAML, this means
separate Deployment files per environment (or a templating tool).

</details>

### Task 2: Immutable ConfigMaps for Production

The production ConfigMap should be immutable to prevent accidental changes.

1. Add the `immutable: true` field to the production ConfigMap.
2. What happens if you try to `kubectl edit` an immutable ConfigMap?
3. What is the required workflow to change an immutable ConfigMap's values?
4. Why does Kubernetes offer this feature? Name two benefits.

<details>
<summary>Hint 2: immutable field</summary>

The `immutable` field on a ConfigMap or Secret prevents any updates to the
`data` or `binaryData` fields after creation. To change values, you must:

1. Delete the ConfigMap.
2. Create a new ConfigMap with updated values.
3. Restart Pods that reference it (or deploy a new ReplicaSet with a
   different ConfigMap name).

</details>

### Task 3: Hash-Triggered Rolling Restarts

When a ConfigMap changes, Pods using it as environment variables do not
restart automatically. Design a mechanism that triggers a rolling restart
when the ConfigMap changes, without manual `kubectl rollout restart`.

The approach should:

1. Work with plain `kubectl apply` (no Helm, no Kustomize).
2. Cause Kubernetes to detect a Pod template change and trigger a rollout.
3. Not require an external controller or operator.

Write the Deployment YAML that implements this mechanism.

<details>
<summary>Hint 3: Annotation-based change detection</summary>

Kubernetes triggers a rolling update when the Pod template changes. The Pod
template includes `metadata.annotations`. If you put a hash of the ConfigMap
contents in an annotation, changing the ConfigMap changes the annotation,
which changes the Pod template, which triggers a rollout.

The challenge is computing the hash. In a CI/CD pipeline, you can compute it
with:

```bash
CONFIG_HASH=$(kubectl get configmap app-config -o yaml | sha256sum | cut -d' ' -f1)
```

Then substitute it into the Deployment YAML.

</details>

### Task 4: Secret Separation Strategy

Your team debates two approaches for managing secrets across environments:

**Approach A:** Store all secrets as Kubernetes Secrets in Git, encrypted
with Sealed Secrets or SOPS.

**Approach B:** Store only references in Git; actual secret values live in an
external vault (HashiCorp Vault, AWS Secrets Manager).

For each approach, fill in the comparison table:

| Concern | Approach A | Approach B |
|---------|-----------|-----------|
| Secrets in Git? | ? | ? |
| Rotation workflow | ? | ? |
| Audit trail | ? | ? |
| Operational complexity | ? | ? |
| Disaster recovery | ? | ? |

Recommend one approach for production and explain why.

<details>
<summary>Hint 4: Sealed Secrets vs. External Secrets</summary>

- **Sealed Secrets:** Encrypts Secret manifests with a cluster-specific key.
  The encrypted blob is safe to commit to Git. The Sealed Secrets controller
  in the cluster decrypts it into a regular Secret. Secrets are still
  Kubernetes Secrets at runtime.
- **External Secrets Operator:** Syncs secrets from an external provider
  (Vault, AWS SM, GCP SM) into Kubernetes Secrets. The actual values never
  touch Git. Rotation happens in the external provider and propagates
  automatically.

</details>

---

## Success Criteria

- [ ] Three environment-specific ConfigMaps with correct values.
- [ ] Production ConfigMap is immutable with a documented update workflow.
- [ ] Deployment YAML includes a mechanism for automatic restart on config
      change.
- [ ] A reasoned recommendation on secret management approach with a
      filled-in comparison table.

---

## Hints

<details>
<summary>Hint 5: Why separate ConfigMaps per environment?</summary>

A single ConfigMap cannot hold conflicting values for different environments.
Using one ConfigMap per environment lets you:

- Apply each independently with `kubectl apply`.
- Set different RBAC policies per ConfigMap.
- Make some immutable and others mutable.
- Roll back one environment's config without affecting others.

</details>
