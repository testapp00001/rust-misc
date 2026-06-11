# Solution 02: Harden a Deployment with Security Contexts and PSA

## Hardened Manifest

```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: hardening
  labels:
    pod-security.kubernetes.io/enforce: restricted
    pod-security.kubernetes.io/warn: restricted
    pod-security.kubernetes.io/audit: restricted
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: webapp
  namespace: hardening
  labels:
    app: webapp
spec:
  replicas: 2
  selector:
    matchLabels:
      app: webapp
  template:
    metadata:
      labels:
        app: webapp
    spec:
      securityContext:
        runAsNonRoot: true
        seccompProfile:
          type: RuntimeDefault
      containers:
        - name: webapp
          image: nginx:1.25
          ports:
            - containerPort: 80
          securityContext:
            allowPrivilegeEscalation: false
            runAsUser: 101
            readOnlyRootFilesystem: true
            capabilities:
              drop:
                - ALL
          volumeMounts:
            - name: tmp
              mountPath: /tmp
            - name: cache
              mountPath: /var/cache/nginx
      volumes:
        - name: tmp
          emptyDir: {}
        - name: cache
          emptyDir: {}
```

## Why It Works

### Pod-level security context

```yaml
securityContext:
  runAsNonRoot: true
  seccompProfile:
    type: RuntimeDefault
```

- `runAsNonRoot: true` ensures the kubelet rejects any container that
  attempts to start as UID 0. This is a defence-in-depth measure -- even if
  the image Dockerfile has `USER root`, the pod will not start.
- `seccompProfile.type: RuntimeDefault` applies the container runtime's
  default seccomp profile, which blocks roughly 44 dangerous syscalls
  including `reboot`, `mount`, and `kexec_load`. The restricted standard
  requires this field.

### Container-level security context

```yaml
securityContext:
  allowPrivilegeEscalation: false
  runAsUser: 101
  readOnlyRootFilesystem: true
  capabilities:
    drop:
      - ALL
```

- `allowPrivilegeEscalation: false` sets the `no_new_privs` bit on the
  process, preventing it from gaining more privileges than its parent
  (e.g. via setuid binaries).
- `runAsUser: 101` runs the process as the nginx user. Combined with
  `runAsNonRoot: true`, this guarantees non-root execution.
- `readOnlyRootFilesystem: true` makes the container's root filesystem
  immutable. Any writes must go to mounted volumes, which limits the blast
  radius of a compromise.
- `capabilities.drop: ["ALL"]` removes all Linux capabilities from the
  container. The nginx process does not need any.

### Volume mounts

The two `emptyDir` volumes at `/tmp` and `/var/cache/nginx` are required
because nginx needs to write temporary files and cache data. Without these
mounts, nginx would fail to start with a read-only root filesystem.

## Verification Commands

```bash
# Confirm the namespace has the PSA label
kubectl get namespace hardening --show-labels

# Check pod security context
kubectl get pod -n hardening -l app=webapp \
  -o jsonpath='{.items[0].spec.securityContext}'

# Check container security context
kubectl get pod -n hardening -l app=webapp \
  -o jsonpath='{.items[0].spec.containers[0].securityContext}'

# Verify the process is not running as root
kubectl exec -n hardening deploy/webapp -- id
# Expected: uid=101(nginx) gid=101(nginx)
```

## Common Mistakes

- **Setting seccompProfile at the container level.** The restricted standard
  requires seccomp at the pod level (`spec.securityContext.seccompProfile`).
  Setting it only at the container level will pass the check, but pod-level
  is the conventional and recommended placement.
- **Forgetting emptyDir volumes.** Setting `readOnlyRootFilesystem: true`
  without adding volume mounts for writable paths causes the container to
  crash on startup.
- **Using `runAsUser: 0` or omitting it.** If `runAsUser` is not set and the
  image defaults to root, `runAsNonRoot: true` will reject the pod at the
  kubelet level with a confusing error.
- **Dropping capabilities without `allowPrivilegeEscalation: false`.**
  Dropping `ALL` capabilities is ineffective if privilege escalation is
  allowed, because the process could regain capabilities through setuid
  binaries.
- **Applying to the wrong namespace.** Applying the deployment to `default`
  instead of `hardening` means the restricted PSA label is not enforced.
