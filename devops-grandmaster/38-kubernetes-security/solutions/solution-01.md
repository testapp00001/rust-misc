# Solution 01: Pod Security Standards Explained

## Part A -- Definitions

### 1. Purpose of Pod Security Standards

Pod Security Standards define a clear, opinionated security policy for pod
hardening. They give cluster operators a simple way to restrict what pods can
do inside a namespace without writing custom admission webhooks. The standards
cover privilege escalation, host namespaces, volume types, and other sensitive
pod fields.

### 2. The Three Levels

| Standard | Summary |
|----------|---------|
| **privileged** | Unrestricted. Allows everything. Intended for system-level workloads like CNI plugins, CSI drivers, and kube-system pods. |
| **baseline** | Minimally restrictive. Prevents known privilege escalations such as hostPID, hostNetwork, hostPorts, privileged containers, and dangerous capabilities. Suitable for most application workloads. |
| **restricted** | Heavily restricted. Enforces pod hardening best practices: non-root runtime, read-only root filesystem, drop all capabilities, seccomp profile required. Suitable for security-critical workloads. |

### 3. Enforce vs Warn vs Audit

| Mode | Behaviour |
|------|-----------|
| **enforce** | Rejects pods that violate the standard. The pod is never created. |
| **warn** | Allows the pod but prints a warning to the user who submitted it. Useful for catching violations without blocking. |
| **audit** | Allows the pod but writes an audit event to the cluster audit log. Useful for detecting violations in existing workloads. |

All three modes can be set independently and simultaneously on the same
namespace, which is useful for gradually rolling out stricter policies.

### 4. Why PodSecurityPolicy Was Replaced

PodSecurityPolicy (PSP) was a cluster-wide resource with a complex
interaction model. It was difficult to reason about which policy applied to
a pod because it depended on the creating user, the service account, and
multiple policy objects. It also required enabling an admission plugin. PSA
replaced it because:

- PSA is namespace-scoped and uses simple labels, making it easy to understand.
- No custom resources are needed -- it is built into the admission controller.
- The three fixed profiles are well-defined and cover most use cases.
- PSA supports warn and audit modes for gradual adoption, which PSP lacked.

## Part B -- Classification

| # | Scenario | Minimum Standard | Reasoning |
|---|----------|-----------------|-----------|
| 1 | Root + hostNetwork | **privileged** | `hostNetwork` is denied by baseline. Running as root is denied by restricted. Only privileged allows both. |
| 2 | Non-root, drop all caps | **restricted** | This configuration meets the restricted standard's requirements for non-root execution and capability dropping. |
| 3 | hostPID, non-root | **privileged** | `hostPID: true` is denied by baseline, so only privileged permits it regardless of the user ID. |
| 4 | hostPort binding | **privileged** | hostPorts are denied by the baseline standard. |
| 5 | RuntimeDefault seccomp, UID 1000 | **restricted** | The restricted standard requires a seccomp profile and non-root UID. This pod satisfies both. |

## Part C -- Namespace Labels

```bash
kubectl label namespace production \
  pod-security.kubernetes.io/enforce=restricted \
  pod-security.kubernetes.io/warn=restricted \
  pod-security.kubernetes.io/audit=restricted
```

## Part D -- Quick Validation

```bash
kubectl create namespace test-psa

kubectl label namespace test-psa \
  pod-security.kubernetes.io/enforce=restricted \
  pod-security.kubernetes.io/warn=restricted \
  pod-security.kubernetes.io/audit=restricted

kubectl run nginx-test --image=nginx:latest --restart=Never -n test-psa
```

Expected output (error rejecting the pod):

```
Error from server (Forbidden): pods "nginx-test" is forbidden: violates
PodSecurity "restricted:latest": allowPrivilegeEscalation != false,
runAsNonRoot != true, seccompProfile
```

The pod is rejected because the default nginx image runs as root, does not
set `allowPrivilegeEscalation: false`, and does not set a seccomp profile.

## Common Mistakes

- **Confusing enforce with warn.** Setting only `warn` does not block
  non-compliant pods -- it only prints a message.
- **Applying restricted to kube-system.** The kube-system namespace contains
  system pods that require privileged access. Label it as privileged or leave
  it unlabeled.
- **Forgetting the `pod-security.kubernetes.io/` prefix.** The labels must
  use the full prefix; short forms like `enforce=restricted` alone do nothing.
- **Not checking all three modes independently.** You can enforce baseline
  while warning on restricted to prepare for a stricter rollout.
