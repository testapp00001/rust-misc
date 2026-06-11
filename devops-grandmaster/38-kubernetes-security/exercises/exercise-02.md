# Exercise 02: Harden a Deployment with Security Contexts and PSA

**Type:** Guided
**Estimated time:** 30 minutes

## Objective

Take an insecure deployment manifest and harden it so that it passes the
**restricted** Pod Security Standard. You will configure security contexts
at both the pod and container level and enforce PSA on the namespace.

## Prerequisites

- A Kubernetes cluster with `kubectl` access
- A namespace you can create and label

## Instructions

### Step 1 -- Start with the Insecure Deployment

Save the following manifest as `insecure-deploy.yaml`:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: webapp
  namespace: default
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
      containers:
        - name: webapp
          image: nginx:1.25
          ports:
            - containerPort: 80
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

Deploy it and verify it runs:

```bash
kubectl create namespace hardening
kubectl apply -f insecure-deploy.yaml
kubectl get pods -n default -l app=webapp
```

### Step 2 -- Create the Restricted Namespace

Label the `hardening` namespace so that the restricted standard is enforced:

```bash
kubectl label namespace hardening \
  pod-security.kubernetes.io/enforce=restricted
```

### Step 3 -- Move the Deployment to the Hardened Namespace

Update the manifest (save as `hardened-deploy.yaml`) so that:

1. The deployment targets the `hardening` namespace.
2. The pod-level security context sets `runAsNonRoot: true` and
   `seccompProfile` to `RuntimeDefault`.
3. The container-level security context sets:
   - `allowPrivilegeEscalation: false`
   - `runAsUser: 101` (nginx user in the official image)
   - `readOnlyRootFilesystem: true`
   - `capabilities` with `drop: ["ALL"]`

Remember: the container already mounts emptyDir volumes at `/tmp` and
`/var/cache/nginx` to handle the read-only root filesystem.

### Step 4 -- Verify

Apply your hardened manifest and confirm:

```bash
kubectl apply -f hardened-deploy.yaml
kubectl get pods -n hardening -l app=webapp
```

Then verify the security settings are applied:

```bash
kubectl get pod -n hardening -l app=webapp -o jsonpath='{.items[0].spec.securityContext}'
kubectl get pod -n hardening -l app=webapp -o jsonpath='{.items[0].spec.containers[0].securityContext}'
```

## Success Criteria

- [ ] The deployment runs successfully in the `hardening` namespace.
- [ ] The namespace enforces the restricted PSA standard.
- [ ] The pod runs as a non-root user (UID 101).
- [ ] The root filesystem is read-only.
- [ ] All capabilities are dropped.
- [ ] `allowPrivilegeEscalation` is `false`.
- [ ] A seccomp profile of type `RuntimeDefault` is set.
- [ ] No PSA violation warnings appear when applying the manifest.

## Hints

<details>
<summary>Hint 1 -- Where to set seccompProfile</summary>
The seccomp profile is set at the **pod** level under `spec.securityContext`,
not at the container level. The field is `seccompProfile.type`.
</details>

<details>
<summary>Hint 2 -- nginx user UID</summary>
The official nginx Docker image uses UID 101 for the nginx user. Setting
`runAsUser: 101` ensures the container does not run as root.
</details>

<details>
<summary>Hint 3 -- Read-only rootfs workarounds</summary>
When `readOnlyRootFilesystem: true`, any directory the process needs to
write to must be backed by a volume mount. The manifest already provides
emptyDir mounts for `/tmp` and `/var/cache/nginx`.
</details>
