# Solution 03: Secret Rotation Without Restart

---

## Task 1: Create the Initial Secret and Deployment

### Secret

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: db-credentials
type: Opaque
stringData:
  DB_USER: app_user
  DB_PASSWORD: old-password-2024
```

### Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: payment-service
spec:
  replicas: 2
  selector:
    matchLabels:
      app: payment-service
  template:
    metadata:
      labels:
        app: payment-service
    spec:
      containers:
        - name: app
          image: busybox:1.36
          command:
            - sh
            - -c
            - |
              while true; do
                echo "$(date): DB_USER=$(cat /etc/db-credentials/DB_USER) DB_PASSWORD=$(cat /etc/db-credentials/DB_PASSWORD)"
                sleep 10
              done
          volumeMounts:
            - name: db-creds
              mountPath: /etc/db-credentials
              readOnly: true
      volumes:
        - name: db-creds
          secret:
            secretName: db-credentials
```

Verify:

```bash
kubectl apply -f secret.yaml
kubectl apply -f deployment.yaml
kubectl logs -l app=payment-service --tail=5
```

Expected output (repeated every 10 seconds):

```
Wed Jun 11 12:00:00 UTC 2026: DB_USER=app_user DB_PASSWORD=old-password-2024
```

---

## Task 2: Rotate the Secret

Update the secret:

```bash
kubectl patch secret db-credentials -p '{"stringData":{"DB_PASSWORD":"new-password-2025"}}'
```

Or re-apply an updated manifest:

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: db-credentials
type: Opaque
stringData:
  DB_USER: app_user
  DB_PASSWORD: new-password-2025
```

### Observations

**1. Did you need to restart the Pods?**

No. Because the Secret is mounted as a volume (not used as env vars), the
kubelet automatically syncs the updated contents to the node filesystem.
The Pods' `cat` command reads the file on each iteration, so they pick up
the new password without any restart.

**2. How long did it take for the change to appear?**

Up to 60 seconds. The kubelet's default `--sync-frequency` is 1 minute.
In practice, changes often appear within 30-60 seconds. You can observe
this by watching the Pod logs:

```bash
kubectl logs -l app=payment-service -f
```

**3. Is the update atomic?**

Yes. When the kubelet syncs a Secret volume, it does not modify files in
place. Instead, it:

1. Creates a new directory with the updated files.
2. Atomically swaps the symlink pointing to the data directory.

This means a reader sees either the complete old set of files or the
complete new set. There is no state where `DB_USER` is new but
`DB_PASSWORD` is old (or vice versa).

---

## Task 3: Handle the Transition Period

### Recommended Strategy: Per-Request Read with Short Cache

```
class CredentialProvider:
    def __init__(self, path="/etc/db-credentials"):
        self.path = path
        self._cache = {}
        self._cache_time = 0
        self._cache_ttl = 30  # seconds

    def get_password(self):
        now = time.time()
        if now - self._cache_time > self._cache_ttl:
            self._cache = self._read_from_disk()
            self._cache_time = now
        return self._cache["DB_PASSWORD"]

    def _read_from_disk(self):
        creds = {}
        for key in ["DB_USER", "DB_PASSWORD"]:
            with open(f"{self.path}/{key}") as f:
                creds[key] = f.read().strip()
        return creds
```

### Analysis of Each Approach

**1. Per-request read:**
- Pros: Always fresh. No stale credentials.
- Cons: Disk I/O on every request. Adds latency (microseconds, usually
  negligible).
- Best for: High-security environments where credential freshness is
  critical.

**2. Cache in memory with TTL:**
- Pros: Low I/O overhead. Fresh enough for most use cases.
- Cons: May serve stale credentials for up to TTL seconds.
- Best for: Most applications. A 30-second TTL balances freshness and
  performance.

**3. File watcher (inotify):**
- Pros: Near-instant notification. No polling overhead.
- Cons: Requires a library (e.g., `inotify` in Python, `fsnotify` in Go).
  More complex error handling.
- Best for: Long-lived services that must react within seconds.

### Atomic Update Behavior

When Kubernetes swaps the Secret volume, it uses a symlink-based atomic
update:

```
/var/lib/kubelet/pods/<uid>/volumes/kubernetes.io~secret/db-creds/
  ..data -> ..2026_06_11_12_00_00.1234567890/
  ..2026_06_11_11_59_00.0000000000/   # old data directory
  ..2026_06_11_12_00_00.1234567890/   # new data directory
  DB_USER
  DB_PASSWORD
```

The `..data` symlink is atomically pointed to the new directory. Any process
that opens a file after the swap sees the new data. A process with an
already-open file handle continues reading the old data until it closes and
reopens.

---

## Task 4: Clean Up

```bash
kubectl delete deployment payment-service
kubectl delete secret db-credentials
```

---

## Common Mistakes

1. **Using env vars for rotating secrets.** Environment variables are set
   once at container start. They cannot be updated without restarting the
   container. Always use volume mounts for secrets that rotate.

2. **Expecting instant updates.** The kubelet sync interval introduces a
   delay of up to 60 seconds. Design your rotation schedule with this
   buffer in mind. If old credentials expire before the sync completes,
   Pods will fail during the gap.

3. **Not handling the overlap window.** During rotation, both old and new
   credentials must be valid. If the database rejects the old password
   before all Pods have synced the new one, some Pods will fail. Use
   overlapping validity windows (e.g., old password valid for 2 hours
   after rotation).

4. **Reading the file once at startup.** If the application reads the
   password file once during initialization and caches it forever, it will
   never see rotated credentials. The application must re-read the file
   periodically or on each connection attempt.

5. **Ignoring file permissions.** Secret volumes are mounted with
   permission `0644` by default. Set `defaultMode` to restrict access:

   ```yaml
   volumes:
     - name: db-creds
       secret:
         secretName: db-credentials
         defaultMode: 0400
   ```

   This ensures only the file owner can read the credentials.
