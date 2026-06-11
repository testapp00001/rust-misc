# Exercise 01: ConfigMap vs Secret -- Know the Difference

**Type:** Conceptual
**Time:** 20 minutes
**Difficulty:** Beginner

## Objective

Understand when to use a ConfigMap versus a Secret, how each stores data
internally, and what happens when you update one that a running Pod is
consuming.

---

## Background

Your team deploys a microservice that needs the following configuration at
runtime:

| Item | Example Value |
|------|---------------|
| Database hostname | `postgres.production.svc` |
| Database port | `5432` |
| Log level | `info` |
| Database password | `s3cur3P@ss!` |
| TLS certificate | A PEM-encoded certificate |
| API key | `sk-live-abc123xyz` |
| Feature flag: dark mode | `true` |
| JWT signing secret | A 256-bit key |

You need to decide which items belong in a ConfigMap and which belong in a
Secret, and justify each choice.

---

## Tasks

### Task 1: Classification Table

Classify each item from the Background table as ConfigMap or Secret. Fill in
the "Where" and "Why" columns.

| Item | Where (ConfigMap / Secret) | Why |
|------|---------------------------|-----|
| Database hostname | ? | ? |
| Database port | ? | ? |
| Log level | ? | ? |
| Database password | ? | ? |
| TLS certificate | ? | ? |
| API key | ? | ? |
| Feature flag: dark mode | ? | ? |
| JWT signing secret | ? | ? |

### Task 2: Storage Differences

Answer each question with a specific, concrete answer.

1. How does Kubernetes encode data internally in a ConfigMap?
2. How does Kubernetes encode data internally in a Secret?
3. Is base64 encoding the same as encryption? Why or why not?
4. What must you configure to encrypt Secrets at rest in etcd?

### Task 3: Update Behavior

A Deployment consumes a ConfigMap in two different ways:

- **Pod A** uses the ConfigMap as environment variables via `envFrom`.
- **Pod B** mounts the ConfigMap as a volume at `/etc/config`.

You run: `kubectl edit configmap my-config` and change a value.

1. Does Pod A pick up the change? Why or why not?
2. Does Pod B pick up the change? Why or why not?
3. What command forces Pod A to see the new value?
4. Roughly how long does it take for Pod B to see the new value?

---

## Success Criteria

- [ ] All 8 items are correctly classified with a technical justification.
- [ ] You can explain the difference between encoding and encryption.
- [ ] You can describe the update behavior for both env vars and volume mounts.
- [ ] You know how to force a Pod to pick up ConfigMap/Secret changes.

---

## Hints

<details>
<summary>Hint 1: Classification Rule of Thumb</summary>

Ask: "If this value appeared in a Git commit or a log file, would it cause a
security incident?" If yes, it belongs in a Secret. If no, it belongs in a
ConfigMap.

</details>

<details>
<summary>Hint 2: Base64 Is Not Encryption</summary>

Base64 is a reversible encoding scheme designed for transporting binary data
over text-only channels. Anyone with the encoded string can decode it with
`echo "encoded" | base64 -d`. Encryption requires a key and an algorithm
(e.g., AES-CBC).

</details>

<details>
<summary>Hint 3: Env Vars Are Set Once</summary>

When a container starts, Kubernetes injects environment variables from
ConfigMaps and Secrets. The container process reads them once at startup.
Changing the source ConfigMap or Secret has no effect on a running process --
the process would need to be restarted.

</details>

<details>
<summary>Hint 4: Volume Mounts Use a Projected Volume</summary>

When you mount a ConfigMap as a volume, Kubernetes creates a projected volume.
The kubelet periodically syncs the contents from the API server to the node's
filesystem. This means the files at the mount path are updated automatically,
but there is a sync interval (default: up to 60 seconds).

</details>
