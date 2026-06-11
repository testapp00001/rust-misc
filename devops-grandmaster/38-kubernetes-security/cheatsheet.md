# Cheatsheet: Kubernetes Security

## Pod Security Standards

| Level | Description | Use Case |
|-------|-------------|----------|
| `privileged` | Unrestricted | System workloads |
| `baseline` | Known privilege escalations blocked | Most workloads |
| `restricted` | Hardened, best practice | Security-critical |

## Pod Security Admission (PSA)

```yaml
# Namespace labels
apiVersion: v1
kind: Namespace
metadata:
  name: production
  labels:
    pod-security.kubernetes.io/enforce: restricted
    pod-security.kubernetes.io/warn: restricted
    pod-security.kubernetes.io/audit: restricted
```

## Security Context

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: secure-pod
spec:
  securityContext:
    runAsNonRoot: true
    runAsUser: 1000
    runAsGroup: 1000
    fsGroup: 1000
    seccompProfile:
      type: RuntimeDefault
  containers:
  - name: app
    image: myapp:1.0
    securityContext:
      allowPrivilegeEscalation: false
      readOnlyRootFilesystem: true
      capabilities:
        drop: ["ALL"]
    volumeMounts:
    - name: tmp
      mountPath: /tmp
  volumes:
  - name: tmp
    emptyDir: {}
```

## RBAC Quick Reference

```yaml
# Role (namespace-scoped)
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: pod-reader
rules:
- apiGroups: [""]
  resources: ["pods"]
  verbs: ["get", "list", "watch"]

# RoleBinding
apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: read-pods
subjects:
- kind: User
  name: jane
roleRef:
  kind: Role
  name: pod-reader
  apiGroup: rbac.authorization.k8s.io
```

## Image Security

```bash
# Scan with Trivy
trivy image myapp:1.0

# Scan with Grype
grype myapp:1.0

# Verify image signature (Cosign)
cosign verify --key cosign.pub myregistry.io/myapp:1.0

# Sign image
cosign sign --key cosign.key myregistry.io/myapp:1.0
```

## Network Policies

```yaml
# Default deny all ingress
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: default-deny-ingress
spec:
  podSelector: {}
  policyTypes:
  - Ingress

# Allow only from specific namespace
ingress:
- from:
  - namespaceSelector:
      matchLabels:
        name: frontend
```

## Secrets Management

```bash
# Create secret
kubectl create secret generic db-pass --from-literal=password=mysecret

# Use in Pod
env:
- name: DB_PASSWORD
  valueFrom:
    secretKeyRef:
      name: db-pass
      key: password

# External Secrets Operator (recommended)
# Syncs from Vault, AWS SM, GCP SM, etc.
```

## Security Checklist

- [ ] Pod Security Standards enforced on namespaces
- [ ] RBAC with least privilege
- [ ] NetworkPolicies restricting pod-to-pod traffic
- [ ] Images scanned for vulnerabilities
- [ ] Non-root containers
- [ ] Read-only root filesystem
- [ ] No privilege escalation
- [ ] Seccomp profile applied
- [ ] Secrets externalized (not in etcd plaintext)
- [ ] API server audit logging enabled
