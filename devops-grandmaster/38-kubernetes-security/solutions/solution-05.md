# Solution 05: Design a Complete Kubernetes Security Policy

## Part A -- Namespace and PSA Configuration

```yaml
# Platform namespace -- no PSA restrictions (system-level workloads)
apiVersion: v1
kind: Namespace
metadata:
  name: platform
---
# QA namespace -- baseline enforcement
apiVersion: v1
kind: Namespace
metadata:
  name: qa
  labels:
    pod-security.kubernetes.io/enforce: baseline
    pod-security.kubernetes.io/warn: restricted
    pod-security.kubernetes.io/audit: restricted
---
# Production namespaces -- restricted enforcement
apiVersion: v1
kind: Namespace
metadata:
  name: prod-api
  labels:
    environment: production
    pod-security.kubernetes.io/enforce: restricted
    pod-security.kubernetes.io/warn: restricted
    pod-security.kubernetes.io/audit: restricted
---
apiVersion: v1
kind: Namespace
metadata:
  name: prod-worker
  labels:
    environment: production
    pod-security.kubernetes.io/enforce: restricted
    pod-security.kubernetes.io/warn: restricted
    pod-security.kubernetes.io/audit: restricted
---
# Staging namespaces -- baseline enforcement, warn on restricted
apiVersion: v1
kind: Namespace
metadata:
  name: staging-api
  labels:
    environment: staging
    pod-security.kubernetes.io/enforce: baseline
    pod-security.kubernetes.io/warn: restricted
    pod-security.kubernetes.io/audit: restricted
---
apiVersion: v1
kind: Namespace
metadata:
  name: staging-worker
  labels:
    environment: staging
    pod-security.kubernetes.io/enforce: baseline
    pod-security.kubernetes.io/warn: restricted
    pod-security.kubernetes.io/audit: restricted
---
# Development namespaces -- baseline enforcement, warn only
apiVersion: v1
kind: Namespace
metadata:
  name: dev-api
  labels:
    environment: development
    pod-security.kubernetes.io/enforce: baseline
    pod-security.kubernetes.io/warn: baseline
    pod-security.kubernetes.io/audit: baseline
---
apiVersion: v1
kind: Namespace
metadata:
  name: dev-worker
  labels:
    environment: development
    pod-security.kubernetes.io/enforce: baseline
    pod-security.kubernetes.io/warn: baseline
    pod-security.kubernetes.io/audit: baseline
```

### Why platform and kube-system are excluded

The `platform` namespace hosts infrastructure components (CNI, CSI, ingress
controllers) that often require privileged access to the host. Similarly,
`kube-system` runs core Kubernetes components. Enforcing restricted PSA on
these namespaces would break cluster functionality. They should be left
unlabeled or labeled as `privileged`.

## Part B -- RBAC Design

### Platform Team

```yaml
apiVersion: v1
kind: ServiceAccount
metadata:
  name: platform-sa
  namespace: platform
---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRoleBinding
metadata:
  name: platform-admin-binding
subjects:
  - kind: ServiceAccount
    name: platform-sa
    namespace: platform
roleRef:
  kind: ClusterRole
  name: cluster-admin
  apiGroup: rbac.authorization.k8s.io
```

### Backend Team (per environment)

This template applies to each environment. Create one set per namespace
(`prod-api`, `prod-worker`, `staging-api`, etc.).

```yaml
apiVersion: v1
kind: ServiceAccount
metadata:
  name: backend-sa
  namespace: NAMESPACE_PLACEHOLDER
---
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: backend-role
  namespace: NAMESPACE_PLACEHOLDER
rules:
  # Application workloads
  - apiGroups: ["apps"]
    resources: ["deployments", "replicasets", "statefulsets"]
    verbs: ["get", "list", "watch", "create", "update", "patch", "delete"]
  - apiGroups: [""]
    resources: ["pods", "services", "configmaps", "endpoints"]
    verbs: ["get", "list", "watch", "create", "update", "patch", "delete"]
  - apiGroups: [""]
    resources: ["pods/log"]
    verbs: ["get"]
  - apiGroups: [""]
    resources: ["pods/exec"]
    verbs: ["create"]
  - apiGroups: ["networking.k8s.io"]
    resources: ["ingresses"]
    verbs: ["get", "list", "watch", "create", "update", "patch", "delete"]
  # Secrets -- read only
  - apiGroups: [""]
    resources: ["secrets"]
    verbs: ["get", "list", "watch"]
  # Events -- read only (for debugging)
  - apiGroups: [""]
    resources: ["events"]
    verbs: ["get", "list", "watch"]
---
apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: backend-rolebinding
  namespace: NAMESPACE_PLACEHOLDER
subjects:
  - kind: ServiceAccount
    name: backend-sa
    namespace: NAMESPACE_PLACEHOLDER
roleRef:
  kind: Role
  name: backend-role
  apiGroup: rbac.authorization.k8s.io
```

### QA Team

```yaml
apiVersion: v1
kind: ServiceAccount
metadata:
  name: qa-sa
  namespace: qa
---
# Read-only ClusterRole for staging and production
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: qa-readonly
rules:
  - apiGroups: ["*"]
    resources: ["*"]
    verbs: ["get", "list", "watch"]
---
# Bind to staging namespaces
apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: qa-staging-api-binding
  namespace: staging-api
subjects:
  - kind: ServiceAccount
    name: qa-sa
    namespace: qa
roleRef:
  kind: ClusterRole
  name: qa-readonly
  apiGroup: rbac.authorization.k8s.io
---
apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: qa-staging-worker-binding
  namespace: staging-worker
subjects:
  - kind: ServiceAccount
    name: qa-sa
    namespace: qa
roleRef:
  kind: ClusterRole
  name: qa-readonly
  apiGroup: rbac.authorization.k8s.io
---
# Bind to production namespaces
apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: qa-prod-api-binding
  namespace: prod-api
subjects:
  - kind: ServiceAccount
    name: qa-sa
    namespace: qa
roleRef:
  kind: ClusterRole
  name: qa-readonly
  apiGroup: rbac.authorization.k8s.io
---
apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: qa-prod-worker-binding
  namespace: prod-worker
subjects:
  - kind: ServiceAccount
    name: qa-sa
    namespace: qa
roleRef:
  kind: ClusterRole
  name: qa-readonly
  apiGroup: rbac.authorization.k8s.io
---
# Exec permission -- staging only
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: qa-staging-exec
  namespace: staging-api
rules:
  - apiGroups: [""]
    resources: ["pods/exec"]
    verbs: ["create"]
---
apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: qa-staging-exec-binding
  namespace: staging-api
subjects:
  - kind: ServiceAccount
    name: qa-sa
    namespace: qa
roleRef:
  kind: Role
  name: qa-staging-exec
  apiGroup: rbac.authorization.k8s.io
---
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: qa-staging-worker-exec
  namespace: staging-worker
rules:
  - apiGroups: [""]
    resources: ["pods/exec"]
    verbs: ["create"]
---
apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: qa-staging-worker-exec-binding
  namespace: staging-worker
subjects:
  - kind: ServiceAccount
    name: qa-sa
    namespace: qa
roleRef:
  kind: Role
  name: qa-staging-worker-exec
  apiGroup: rbac.authorization.k8s.io
```

## Part C -- Image Security Policy

### Gatekeeper ConstraintTemplate

```yaml
apiVersion: templates.gatekeeper.sh/v1
kind: ConstraintTemplate
metadata:
  name: k8sallowedrepos
spec:
  crd:
    spec:
      names:
        kind: K8sAllowedRepos
      validation:
        openAPIV3Schema:
          type: object
          properties:
            repos:
              type: array
              items:
                type: string
            requireDigest:
              type: boolean
            requireScanAnnotation:
              type: boolean
  targets:
    - target: admission.k8s.gatekeeper.sh
      rego: |
        package k8sallowedrepos

        violation[{"msg": msg}] {
          container := input.review.object.spec.containers[_]
          not startswith_allowed_repo(container.image)
          msg := sprintf("Container %v has an image from an untrusted registry: %v", [container.name, container.image])
        }

        violation[{"msg": msg}] {
          container := input.review.object.spec.containers[_]
          input.parameters.requireDigest
          not contains(container.image, "@sha256:")
          endswith(container.image, ":latest")
          msg := sprintf("Container %v uses the 'latest' tag which is not allowed", [container.name])
        }

        violation[{"msg": msg}] {
          container := input.review.object.spec.containers[_]
          input.parameters.requireScanAnnotation
          not input.review.object.metadata.annotations["acme.io/trivy-scan"] == "passed"
          msg := "Pod must have annotation acme.io/trivy-scan: passed"
        }

        startswith_allowed_repo(image) {
          some i
          startswith(image, input.parameters.repos[i])
        }
```

### Gatekeeper Constraint

```yaml
apiVersion: constraints.gatekeeper.sh/v1beta1
kind: K8sAllowedRepos
metadata:
  name: require-trusted-repos
spec:
  match:
    kinds:
      - apiGroups: [""]
        kinds: ["Pod"]
    namespaces:
      - "prod-api"
      - "prod-worker"
      - "staging-api"
      - "staging-worker"
  parameters:
    repos:
      - "docker.io/library/"
      - "ghcr.io/acme-corp/"
      - "123456789.dkr.ecr.us-east-1.amazonaws.com/"
    requireDigest: false
    requireScanAnnotation: true
```

### Plain-language alternative (if Gatekeeper is not installed)

**Image Policy Standard:**

1. All production images MUST come from one of the three approved registries.
2. The `latest` tag is prohibited in staging and production. Use semantic
   version tags or SHA digests.
3. All images must have the annotation `acme.io/trivy-scan: "passed"` in
   their Deployment metadata.
4. CI pipelines must run `trivy image --exit-code 1 --severity CRITICAL`
   and gate on the result.

**Validation script:**

```bash
#!/bin/bash
# validate-images.sh -- checks all deployments in a namespace

NAMESPACE=${1:?Usage: $0 <namespace>}

ALLOWED_REGISTRIES=(
  "docker.io/library/"
  "ghcr.io/acme-corp/"
  "123456789.dkr.ecr.us-east-1.amazonaws.com/"
)

kubectl get deployments -n "$NAMESPACE" -o json | jq -r '
  .items[] |
  .metadata.name as $deploy |
  .spec.template.metadata.annotations["acme.io/trivy-scan"] as $scan |
  .spec.template.spec.containers[] |
  "\($deploy)|\(.image)|\($scan // "missing")"
' | while IFS='|' read -r deploy image scan; do
  # Check registry
  allowed=false
  for reg in "${ALLOWED_REGISTRIES[@]}"; do
    if [[ "$image" == "$reg"* ]]; then
      allowed=true
      break
    fi
  done
  if [ "$allowed" = false ]; then
    echo "FAIL: $deploy uses untrusted image: $image"
  fi

  # Check latest tag
  if [[ "$image" == *":latest" ]] || [[ "$image" != *":"* && "$image" != *"@sha256:"* ]]; then
    echo "FAIL: $deploy uses 'latest' tag: $image"
  fi

  # Check scan annotation
  if [ "$scan" != "passed" ]; then
    echo "FAIL: $deploy missing trivy scan annotation (got: $scan)"
  fi
done
```

## Part D -- Seccomp and AppArmor Profiles

### Custom Seccomp Profile

```json
{
  "defaultAction": "SCMP_ACT_ERRNO",
  "defaultErrnoRet": 1,
  "architectures": ["SCMP_ARCH_X86_64"],
  "syscalls": [
    {
      "names": [
        "accept4", "access", "arch_prctl", "bind", "brk",
        "clock_gettime", "clone", "close", "connect",
        "dup", "dup2", "epoll_create1", "epoll_ctl",
        "epoll_wait", "execve", "exit", "exit_group",
        "faccessat", "fadvise64", "fallocate", "fchmod",
        "fchown", "fcntl", "fdatasync", "flock", "fstat",
        "futex", "getcwd", "getdents64", "getegid",
        "geteuid", "getgid", "getpeername", "getpid",
        "getppid", "getrandom", "getsockname", "getsockopt",
        "gettid", "getuid", "ioctl", "listen", "lseek",
        "madvise", "mmap", "mprotect", "munmap", "nanosleep",
        "newfstatat", "openat", "pipe2", "poll", "prctl",
        "pread64", "prlimit64", "pwrite64", "read",
        "readlink", "recvfrom", "recvmsg", "rename",
        "restart_syscall", "rt_sigaction", "rt_sigprocmask",
        "rt_sigreturn", "sendmsg", "sendto", "set_robust_list",
        "set_tid_address", "setsockopt", "shutdown", "sigaltstack",
        "socket", "stat", "statfs", "tgkill", "umask",
        "uname", "unlink", "wait4", "write", "writev"
      ],
      "action": "SCMP_ACT_ALLOW"
    }
  ]
}
```

Save this as `seccomp-nginx.json` and load it as a seccomp profile.

### Applying the Seccomp Profile

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: webapp
  namespace: prod-api
spec:
  template:
    spec:
      securityContext:
        seccompProfile:
          type: Localhost
          localhostProfile: profiles/seccomp-nginx.json
      containers:
        - name: webapp
          image: ghcr.io/acme-corp/webapp:1.0.0@sha256:abc123
```

The profile file must be present on each node at
`/var/lib/kubelet/seccomp/profiles/seccomp-nginx.json`.

### AppArmor Profile

```
#include <tunables/global>

profile nginx-profile flags=(attach_disconnected) {
  #include <abstractions/base>
  #include <abstractions/nameservice>

  # Allow reading nginx configuration
  /etc/nginx/** r,
  /etc/nginx/ r,

  # Allow reading TLS certificates
  /etc/ssl/** r,

  # Allow writing to tmp and cache
  /tmp/** rw,
  /var/cache/nginx/** rw,

  # Allow reading application code
  /app/** r,

  # Deny writes to sensitive paths
  deny /etc/** w,
  deny /proc/** w,
  deny /sys/** w,

  # Network -- allow TCP on port 8080 only
  network inet stream,
  network inet6 stream,

  # Allow common syscalls
  /proc/self/fd/** r,
  /proc/sys/net/core/somaxconn r,
  /dev/null rw,
  /dev/urandom r,

  # Signal handling
  signal (receive) set=(term, int, hup),
  signal (send) set=(term, int, hup),

  # Capability
  capability net_bind_service,
}
```

Save as `apparmor-nginx` and load on each node:

```bash
sudo apparmor_parser -r apparmor-nginx
```

### Pod Annotation for AppArmor

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: webapp
  namespace: prod-api
  annotations:
    container.apparmor.security.beta.kubernetes.io/webapp: localhost/nginx-profile
spec:
  containers:
    - name: webapp
      image: ghcr.io/acme-corp/webapp:1.0.0
```

## Part E -- Network Policies

```yaml
# Default deny all traffic in prod-api
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: default-deny-all
  namespace: prod-api
spec:
  podSelector: {}
  policyTypes:
    - Ingress
    - Egress
---
# Allow ingress from ingress controller
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-ingress-controller
  namespace: prod-api
spec:
  podSelector: {}
  policyTypes:
    - Ingress
  ingress:
    - from:
        - namespaceSelector:
            matchLabels:
              kubernetes.io/metadata.name: ingress-nginx
      ports:
        - protocol: TCP
          port: 8080
---
# Allow ingress from prod-worker
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-worker-ingress
  namespace: prod-api
spec:
  podSelector: {}
  policyTypes:
    - Ingress
  ingress:
    - from:
        - namespaceSelector:
            matchLabels:
              kubernetes.io/metadata.name: prod-worker
      ports:
        - protocol: TCP
          port: 8080
---
# Allow egress to DNS
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-dns-egress
  namespace: prod-api
spec:
  podSelector: {}
  policyTypes:
    - Egress
  egress:
    - to:
        - namespaceSelector:
            matchLabels:
              kubernetes.io/metadata.name: kube-system
      ports:
        - protocol: UDP
          port: 53
        - protocol: TCP
          port: 53
---
# Allow egress to prod-worker
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-worker-egress
  namespace: prod-api
spec:
  podSelector: {}
  policyTypes:
    - Egress
  egress:
    - to:
        - namespaceSelector:
            matchLabels:
              kubernetes.io/metadata.name: prod-worker
---
# Allow egress to external HTTPS
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-external-https
  namespace: prod-api
spec:
  podSelector: {}
  policyTypes:
    - Egress
  egress:
    - to:
        - ipBlock:
            cidr: 0.0.0.0/0
            except:
              - 10.0.0.0/8
              - 172.16.0.0/12
              - 192.168.0.0/16
      ports:
        - protocol: TCP
          port: 43
```

## Part F -- Validation Checklist

```bash
# === PSA Validation ===
# Check all namespace labels
kubectl get namespaces -L pod-security.kubernetes.io/enforce \
  pod-security.kubernetes.io/warn,pod-security.kubernetes.io/audit

# Verify production rejects privileged pods
kubectl run test-privileged --image=nginx --restart=Never \
  -n prod-api --overrides='{"spec":{"containers":[{"name":"test","image":"nginx","securityContext":{"privileged":true}}]}}'
# Expected: rejected

# === RBAC Validation ===
# Platform team
kubectl auth can-i '*' '*' --as=system:serviceaccount:platform:platform-sa
# Expected: yes

# Backend team -- can deploy
kubectl auth can-i create deployments \
  --as=system:serviceaccount:prod-api:backend-sa -n prod-api
# Expected: yes

# Backend team -- cannot modify RBAC
kubectl auth can-i create roles \
  --as=system:serviceaccount:prod-api:backend-sa -n prod-api
# Expected: no

# QA team -- can read production
kubectl auth can-i get pods \
  --as=system:serviceaccount:qa:qa-sa -n prod-api
# Expected: yes

# QA team -- cannot exec in production
kubectl auth can-i create pods/exec \
  --as=system:serviceaccount:qa:qa-sa -n prod-api
# Expected: no

# QA team -- can exec in staging
kubectl auth can-i create pods/exec \
  --as=system:serviceaccount:qa:qa-sa -n staging-api
# Expected: yes

# QA team -- no access to dev
kubectl auth can-i get pods \
  --as=system:serviceaccount:qa:qa-sa -n dev-api
# Expected: no

# === Image Validation ===
# Run the validation script
./validate-images.sh prod-api
./validate-images.sh staging-api

# === Network Policy Validation ===
# Check policies exist
kubectl get networkpolicies -n prod-api

# Test from a pod in prod-api (should fail to reach internet on port 80)
kubectl run test-net --image=busybox --restart=Never -n prod-api \
  --rm -it -- wget -T 5 http://example.com
# Expected: timeout/fail (only port 443 allowed outbound)

# Test DNS resolution (should work)
kubectl run test-dns --image=busybox --restart=Never -n prod-api \
  --rm -it -- nslookup kubernetes.default
# Expected: success
```

## Common Mistakes

- **Using ClusterRoleBinding for the backend team.** This would give them
  access to all namespaces, violating the multi-tenant requirement.
- **Forgetting that NetworkPolicies are additive.** A default-deny policy
  must exist first; additional policies only add permissions, never remove
  them.
- **Not labeling the ingress-nginx namespace.** The NetworkPolicy uses
  `kubernetes.io/metadata.name: ingress-nginx` as a selector. If the
  namespace has a different name, the policy silently allows nothing.
- **Applying restricted PSA to kube-system.** This breaks core cluster
  components that require privileged access.
- **Using `0.0.0.0/0` without excluding private ranges.** This would allow
  egress to internal cluster networks, defeating the purpose of network
  segmentation.
- **Not testing RBAC with `kubectl auth can-i`.** Always verify. RBAC
  misconfigurations are silent until a user hits the forbidden error.
- **Forgetting the AppArmor annotation format.** The annotation key must be
  `container.apparmor.security.beta.kubernetes.io/<container-name>` and
  the value must be `localhost/<profile-name>`.
