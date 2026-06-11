# Lesson 38: Kubernetes Security

**Previous:** [Kubernetes Networking](../37-kubernetes-networking-deep-dive/README.md) | **Next:** [Metrics Collection](../39-metrics-collection/README.md)

---

## The Problem

A Kubernetes cluster is a massive attack surface. Containers run as root by default. Any Pod can access the Kubernetes API. Images pulled from public registries contain known vulnerabilities. A single compromised Pod can pivot to the node, then to the cluster, then to your data. Security is not a feature you add later -- it is a posture you maintain from the start.

---

## The Naive Way

Run everything as root, disable RBAC, pull images from Docker Hub without scanning, and trust that the cluster is "internal" so it is safe.

```yaml
# A Pod running as root with full host access
apiVersion: v1
kind: Pod
metadata:
  name: dangerous-pod
spec:
  hostNetwork: true
  hostPID: true
  containers:
  - name: app
    image: myapp:latest
    securityContext:
      privileged: true
    volumeMounts:
    - name: host-fs
      mountPath: /host
  volumes:
  - name: host-fs
    hostPath:
      path: /
```

This Pod can read every file on the node, kill any process, and escape the container. It is common in tutorials and terrifying in production.

---

## The Right Way

Apply Pod Security Standards, enforce with Pod Security Admission, and scan images before deployment.

### Pod Security Standards (PSS)

Kubernetes defines three levels of security restriction:

**Privileged** -- No restrictions. For system-level workloads (CNI plugins, log collectors, storage drivers).

**Baseline** -- Prevents known privilege escalations. For most workloads.

| Restriction | What it blocks |
|------------|----------------|
| hostNetwork | Pod cannot use the host network namespace |
| hostPID | Pod cannot see host processes |
| hostIPC | Pod cannot use host IPC |
| privileged | Container cannot run in privileged mode |
| hostPath | Pod cannot mount host filesystem |
| allowPrivilegeEscalation | Container cannot gain more privileges than parent |
| capabilities | Blocks dangerous capabilities (SYS_ADMIN, NET_ADMIN, etc.) |
| runAsNonRoot | Container must not run as root |

**Restricted** -- Hardened policy for security-critical workloads. Includes all baseline restrictions plus:
- Must run as non-root
- Must drop ALL capabilities, only add NET_BIND_SERVICE if needed
- Must use seccomp profile RuntimeDefault or Localhost
- Cannot use volume types other than configMap, csi, downwardAPI, emptyDir, ephemeral, persistentVolumeClaim, projected, secret

### Pod Security Admission (PSA)

PSA is a built-in admission controller that enforces PSS at the namespace level.

```yaml
# Label a namespace to enforce baseline policy
apiVersion: v1
kind: Namespace
metadata:
  name: production
  labels:
    pod-security.kubernetes.io/enforce: baseline
    pod-security.kubernetes.io/enforce-version: latest
    pod-security.kubernetes.io/warn: restricted
    pod-security.kubernetes.io/warn-version: latest
    pod-security.kubernetes.io/audit: restricted
    pod-security.kubernetes.io/audit-version: latest
```

Three modes per standard:
- **enforce** -- Reject Pods that violate the policy
- **warn** -- Allow but show a warning to the user
- **audit** -- Allow but log an audit event

Production pattern: enforce baseline, warn on restricted, audit restricted.

```bash
# Apply labels to an existing namespace
kubectl label namespace production \
  pod-security.kubernetes.io/enforce=baseline \
  pod-security.kubernetes.io/enforce-version=latest \
  pod-security.kubernetes.io/warn=restricted \
  pod-security.kubernetes.io/warn-version=latest \
  pod-security.kubernetes.io/audit=restricted \
  pod-security.kubernetes.io/audit-version=latest

# Test: Try to create a privileged Pod in the namespace
kubectl run test --image=nginx --privileged -n production
# Error: pods "test" is forbidden: violates PodSecurity "baseline:latest": privileged
```

### Image Scanning

Never deploy an image without scanning it for known vulnerabilities.

```bash
# Trivy -- scan an image before deployment
trivy image myapp:v1.2.3

# Scan with severity threshold
trivy image --severity HIGH,CRITICAL myapp:v1.2.3

# Fail the build if CRITICAL vulnerabilities are found
trivy image --exit-code 1 --severity CRITICAL myapp:v1.2.3
```

```yaml
# CI pipeline step
- name: Scan image
  run: |
    trivy image --exit-code 1 --severity CRITICAL \
      --ignore-unfixed \
      ${{ env.REGISTRY }}/${{ env.IMAGE }}:${{ env.TAG }}
```

Prevent unscanned images from running in the cluster using admission webhooks or OPA/Gatekeeper.

### Hardened Pod Example

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: secure-pod
spec:
  automountServiceAccountToken: false
  securityContext:
    runAsNonRoot: true
    runAsUser: 1000
    runAsGroup: 1000
    fsGroup: 1000
    seccompProfile:
      type: RuntimeDefault
  containers:
  - name: app
    image: myapp:v1.2.3@sha256:abc123...  # Pin by digest
    securityContext:
      allowPrivilegeEscalation: false
      readOnlyRootFilesystem: true
      capabilities:
        drop:
        - ALL
    resources:
      limits:
        cpu: "500m"
        memory: "256Mi"
      requests:
        cpu: "100m"
        memory: "128Mi"
    volumeMounts:
    - name: tmp
      mountPath: /tmp
  volumes:
  - name: tmp
    emptyDir: {}
```

---

## The Production Way

### OPA/Gatekeeper for Custom Policies

Pod Security Admission is limited to the three predefined standards. OPA (Open Policy Agent) Gatekeeper lets you write custom policies using Rego.

```bash
# Install Gatekeeper
kubectl apply -f https://raw.githubusercontent.com/open-policy-agent/gatekeeper/v3.15.0/deploy/gatekeeper.yaml
```

```yaml
# Require all images come from a trusted registry
apiVersion: templates.gatekeeper.sh/v1
kind: ConstraintTemplate
metadata:
  name: k8sallowedregistries
spec:
  crd:
    spec:
      names:
        kind: K8sAllowedRegistries
      validation:
        openAPIV3Schema:
          type: object
          properties:
            registries:
              type: array
              items:
                type: string
  targets:
  - target: admission.k8s.gatekeeper.sh
    rego: |
      package k8sallowedregistries
      violation[{"msg": msg}] {
        container := input.review.object.spec.containers[_]
        not startswith(container.image, input.parameters.registries[_])
        msg := sprintf("Container <%v> uses image <%v> not from allowed registries: %v", [container.name, container.image, input.parameters.registries])
      }
      violation[{"msg": msg}] {
        container := input.review.object.spec.initContainers[_]
        not startswith(container.image, input.parameters.registries[_])
        msg := sprintf("Init container <%v> uses image <%v> not from allowed registries: %v", [container.name, container.image, input.parameters.registries])
      }

---
apiVersion: constraints.gatekeeper.sh/v1beta1
kind: K8sAllowedRegistries
metadata:
  name: require-trusted-registry
spec:
  match:
    kinds:
    - apiGroups: [""]
      kinds: ["Pod"]
    namespaces:
    - production
    - staging
  parameters:
    registries:
    - "gcr.io/myorg/"
    - "ghcr.io/myorg/"
    - "123456789.dkr.ecr.us-east-1.amazonaws.com/"
```

```yaml
# Require labels on all resources
apiVersion: templates.gatekeeper.sh/v1
kind: ConstraintTemplate
metadata:
  name: k8srequiredlabels
spec:
  crd:
    spec:
      names:
        kind: K8sRequiredLabels
      validation:
        openAPIV3Schema:
          type: object
          properties:
            labels:
              type: array
              items:
                type: string
  targets:
  - target: admission.k8s.gatekeeper.sh
    rego: |
      package k8srequiredlabels
      violation[{"msg": msg}] {
        provided := {label | input.review.object.metadata.labels[label]}
        required := {label | label := input.parameters.labels[_]}
        missing := required - provided
        count(missing) > 0
        msg := sprintf("Missing required labels: %v", [missing])
      }

---
apiVersion: constraints.gatekeeper.sh/v1beta1
kind: K8sRequiredLabels
metadata:
  name: require-team-and-env
spec:
  match:
    kinds:
    - apiGroups: [""]
      kinds: ["Namespace"]
  parameters:
    labels:
    - "team"
    - "environment"
```

### Network Policies for Security

Combine with lessons from the previous topic -- network segmentation is a security control.

```yaml
# Prevent Pods from reaching the Kubernetes API directly
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: restrict-api-access
  namespace: production
spec:
  podSelector: {}
  policyTypes:
  - Egress
  egress:
  - to:
    - ipBlock:
        cidr: 10.96.0.1/32  # ClusterIP of kubernetes service
    ports:
    - protocol: TCP
      port: 443
  # Deny all other egress to the API server range
```

### Audit Logging

Enable Kubernetes audit logging to track who did what.

```yaml
# Audit policy -- save as /etc/kubernetes/audit-policy.yaml
apiVersion: audit.k8s.io/v1
kind: Policy
rules:
# Log all requests at Metadata level
- level: Metadata
  resources:
  - group: ""
    resources: ["secrets", "configmaps"]
# Log request body for changes to Pods
- level: RequestResponse
  resources:
  - group: ""
    resources: ["pods"]
    verbs: ["create", "update", "delete"]
# Don't log read-only requests to endpoints
- level: None
  resources:
  - group: ""
    resources: ["endpoints"]
    verbs: ["get", "list", "watch"]
```

```bash
# Enable audit logging in kube-apiserver
# Add to /etc/kubernetes/manifests/kube-apiserver.yaml:
# --audit-policy-file=/etc/kubernetes/audit-policy.yaml
# --audit-log-path=/var/log/kubernetes/audit.log
# --audit-log-maxage=30
# --audit-log-maxbackup=10
# --audit-log-maxsize=100
```

### Service Account Hardening

```yaml
# Disable automatic token mounting for all ServiceAccounts in a namespace
apiVersion: v1
kind: ServiceAccount
metadata:
  name: app-sa
  namespace: production
automountServiceAccountToken: false

---
# If the app needs the API, create a scoped token
apiVersion: v1
kind: Secret
metadata:
  name: app-sa-token
  namespace: production
  annotations:
    kubernetes.io/service-account.name: app-sa
type: kubernetes.io/service-account-token
```

---

## Hands-On Lab

### Lab: Harden a Namespace End-to-End

**Step 1:** Create a namespace and deploy a vulnerable application.

```bash
kubectl create namespace insecure-app
kubectl run vulnerable --image=nginx:latest -n insecure-app \
  --env="SECRET_KEY=supersecret" \
  --labels=app=web
```

**Step 2:** Verify the Pod runs as root with no restrictions.

```bash
kubectl exec -n insecure-app vulnerable -- whoami
# root

kubectl exec -n insecure-app vulnerable -- cat /etc/shadow
# Should show shadow file (scary!)
```

**Step 3:** Apply Pod Security Admission labels.

```bash
kubectl label namespace insecure-app \
  pod-security.kubernetes.io/enforce=baseline \
  pod-security.kubernetes.io/warn=restricted \
  pod-security.kubernetes.io/audit=restricted
```

**Step 4:** Try to create a privileged Pod -- it should be rejected.

```bash
kubectl run privileged-test --image=nginx --privileged -n insecure-app
# Error: violates PodSecurity "baseline:latest"
```

**Step 5:** Install Trivy and scan the nginx image.

```bash
# Install Trivy (if not installed)
# curl -sfL https://raw.githubusercontent.com/aquasecurity/trivy/main/contrib/install.sh | sh -s -- -b /usr/local/bin

trivy image --severity HIGH,CRITICAL nginx:latest
# Expect many CVEs for the latest tag
```

**Step 6:** Deploy a hardened version of the application.

```yaml
# secure-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: secure-web
  namespace: insecure-app
spec:
  replicas: 1
  selector:
    matchLabels:
      app: secure-web
  template:
    metadata:
      labels:
        app: secure-web
    spec:
      automountServiceAccountToken: false
      securityContext:
        runAsNonRoot: true
        runAsUser: 101  # nginx user
        runAsGroup: 101
        fsGroup: 101
        seccompProfile:
          type: RuntimeDefault
      containers:
      - name: web
        image: nginx:1.25-alpine@sha256:replace-with-digest
        securityContext:
          allowPrivilegeEscalation: false
          readOnlyRootFilesystem: true
          capabilities:
            drop:
            - ALL
        resources:
          limits:
            cpu: "200m"
            memory: "128Mi"
          requests:
            cpu: "50m"
            memory: "64Mi"
        volumeMounts:
        - name: tmp
          mountPath: /tmp
        - name: nginx-cache
          mountPath: /var/cache/nginx
        - name: nginx-run
          mountPath: /var/run
      volumes:
      - name: tmp
        emptyDir: {}
      - name: nginx-cache
        emptyDir: {}
      - name: nginx-run
        emptyDir: {}
```

```bash
kubectl apply -f secure-deployment.yaml
```

**Step 7:** Verify the hardened Pod cannot escalate privileges.

```bash
kubectl exec -n insecure-app deploy/secure-web -- whoami
# nginx (not root)

kubectl exec -n insecure-app deploy/secure-web -- cat /etc/shadow
# Permission denied
```

**Step 8:** Clean up.

```bash
kubectl delete namespace insecure-app
```

---

## Limitation

Kubernetes security hardening protects the cluster from misconfigured workloads and unauthorized access, but it cannot see what is happening inside running containers. Pod Security Standards prevent bad configurations. OPA/Gatekeeper enforces policies at admission time. Image scanning catches known vulnerabilities before deployment. But none of these tools detect runtime anomalies -- a container that starts behaving suspiciously after deployment, a process that opens unexpected network connections, or data being exfiltrated through allowed channels.

Runtime security requires tools like Falco, Sysdig, or Cilium Tetragon that monitor system calls and container behavior in real time. Without runtime visibility, you are securing the gates but not watching the halls.

**Next:** [Metrics Collection](../39-metrics-collection/README.md) -- Prometheus, Grafana, and the tools that let you see what is actually happening inside your cluster.
