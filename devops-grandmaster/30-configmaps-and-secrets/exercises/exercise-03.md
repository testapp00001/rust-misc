# Exercise 03: Secret Rotation Without Restart

**Type:** Independent
**Time:** 30 minutes
**Difficulty:** Intermediate

## Objective

Implement a secret rotation strategy where updated secrets are delivered to
running Pods without restarting them, using volume-mounted Secrets and
understanding the kubelet sync mechanism.

---

## Background

Your team runs a payment processing service. The PCI compliance team requires
that database credentials be rotated every 24 hours. The current deployment
uses Secrets as environment variables:

```yaml
envFrom:
  - secretRef:
      name: db-credentials
```

When the secret is updated with new credentials, the Pods must be manually
restarted (`kubectl rollout restart`) to pick up the new values. This causes
brief downtime during each rotation cycle.

You need to change the deployment so that secret updates are picked up
automatically without Pod restarts.

---

## Tasks

### Task 1: Create the Initial Secret and Deployment

Create a Secret named `db-credentials` with the following keys:

| Key | Value |
|-----|-------|
| DB_USER | app_user |
| DB_PASSWORD | old-password-2024 |

Create a Deployment named `payment-service` with 2 replicas using
`busybox:1.36` that:

1. Mounts `db-credentials` at `/etc/db-credentials` as a read-only volume.
2. Runs a loop that reads and prints `/etc/db-credentials/DB_PASSWORD` every
   10 seconds.
3. Sleeps between iterations.

Verify the Pods are printing the initial password.

<details>
<summary>Hint 1: Why volume mounts, not env vars</summary>

Environment variables are injected at container start and never updated.
Volume mounts are backed by files on the node filesystem. The kubelet
periodically syncs mounted ConfigMap/Secret contents to those files. This
is the only built-in mechanism for automatic updates.

</details>

### Task 2: Rotate the Secret

Update the `db-credentials` Secret with a new password:

| Key | Value |
|-----|-------|
| DB_USER | app_user |
| DB_PASSWORD | new-password-2025 |

Use `kubectl edit`, `kubectl patch`, or re-apply a YAML manifest. After
updating, watch the Pod output to see when the new password appears.

1. Did you need to restart the Pods?
2. How long did it take for the change to appear?
3. Is the update atomic? (Do all files in the mount update at once?)

<details>
<summary>Hint 2: kubectl patch for Secrets</summary>

You can patch a Secret with:

```bash
kubectl patch secret db-credentials -p '{"stringData":{"DB_PASSWORD":"new-password-2025"}}'
```

The `stringData` field lets you provide plain text values; Kubernetes
base64-encodes them automatically.

</details>

### Task 3: Handle the Transition Period

During the sync window (up to 60 seconds), some Pods may have the old
password and others the new one. The database has been updated to accept
both passwords temporarily.

Your application must handle the case where the password file changes
mid-request. Design a strategy:

1. Should the application re-read the password file on every request, cache
   it in memory, or use a file watcher?
2. What happens if the application reads the file while Kubernetes is
   replacing it (atomic update)?
3. Write pseudocode or a description of the ideal credential-reading pattern.

<details>
<summary>Hint 3: Atomic symlink swap</summary>

When the kubelet updates a mounted Secret, it does not modify files in
place. It creates a new directory with the updated files and then
atomically swaps a symlink. This means a reader will see either the old
set of files or the new set -- never a partial mix. However, an
application that opened a file handle before the swap will continue
reading the old data until it reopens the file.

</details>

<details>
<summary>Hint 4: File watching vs polling</summary>

- **Polling:** Re-read the file every N seconds. Simple, but adds latency.
- **inotify/watch:** Get notified when the file changes. More responsive,
  but requires a library.
- **Per-request read:** Read the file on every request. Maximum freshness,
  but adds I/O overhead.

Choose based on how critical credential freshness is versus performance.

</details>

### Task 4: Clean Up

Delete all resources created in this exercise.

---

## Success Criteria

- [ ] The Deployment mounts the Secret as a volume, not as env vars.
- [ ] Rotating the Secret updates the mounted files without Pod restarts.
- [ ] You can describe the kubelet sync interval and the atomic update
      mechanism.
- [ ] You have a strategy for handling credentials that may change at any
      time during application runtime.

---

## Hints

<details>
<summary>Hint 5: Default sync interval</summary>

The kubelet's sync period for ConfigMap/Secret volume mounts defaults to
1 minute (`--sync-frequency` flag). Changes may take up to this interval
to propagate. In Kubernetes 1.27+, you can also set
`configMapAndSecretChangeDetectionStrategy` to `Watch` or `Cache` to
control how changes are detected.

</details>
