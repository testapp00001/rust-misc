# Exercise 05: Design a Complete Kubernetes Security Policy

**Type:** Integration
**Estimated time:** 60 minutes

## Objective

Design and implement a comprehensive Kubernetes security policy that covers
all the topics from this module: Pod Security Standards, RBAC, image
scanning, seccomp profiles, AppArmor annotations, and runtime security.
This is a capstone exercise that brings everything together.

## Scenario

You are the platform security engineer for a company called **Acme Corp**.
The company runs a multi-tenant Kubernetes cluster with three environments:

| Environment | Namespace Pattern | Security Level |
|-------------|-------------------|----------------|
| Production | `prod-*` | Maximum hardening |
| Staging | `staging-*` | Moderate hardening |
| Development | `dev-*` | Relaxed, but still safe |

The company has the following teams:

- **Platform team** (`platform-sa` in `platform` namespace): Full cluster
  admin.
- **Backend team** (`backend-sa` in each environment namespace): Deploy and
  manage application workloads.
- **QA team** (`qa-sa` in `qa` namespace): Read-only access to staging and
  production, exec into pods in staging for debugging.

## Instructions

### Part A -- Namespace and PSA Configuration

Create a document or set of manifests that:

1. Creates the namespace structure:
   - `prod-api`, `prod-worker`
   - `staging-api`, `staging-worker`
   - `dev-api`, `dev-worker`
   - `platform`, `qa`

2. Labels each namespace with the appropriate PSA level:
   - Production: enforce **restricted**, warn and audit on restricted.
   - Staging: enforce **baseline**, warn on restricted, audit on restricted.
   - Development: enforce **baseline**, warn on baseline.

3. Excludes `platform` and `kube-system` from strict PSA (explain why).

### Part B -- RBAC Design

Design RBAC for each team. Provide complete YAML manifests.

**Platform team:**
- Cluster-wide admin using the built-in `cluster-admin` ClusterRole.

**Backend team (per environment):**
- Full CRUD on Deployments, ReplicaSets, Pods, ConfigMaps, Services,
  Ingresses.
- Read-only on Secrets.
- Pod logs and exec.
- Cannot modify RBAC resources or namespaces.

**QA team:**
- Read-only on all resources in `staging-*` and `prod-*` namespaces.
- Exec into pods in `staging-*` namespaces only.
- No access to `dev-*` namespaces.

### Part C -- Image Security Policy

Write a `ConstraintTemplate` and `Constraint` for Gatekeeper (or describe
the policy in plain language if you do not have Gatekeeper installed) that:

1. Rejects images from untrusted registries. Only these registries are
   allowed:
   - `docker.io/library/` (official images only)
   - `ghcr.io/acme-corp/`
   - `123456789.dkr.ecr.us-east-1.amazonaws.com/`

2. Rejects images using the `latest` tag.

3. Requires images to have a trivy scan annotation
   (`acme.io/trivy-scan: "passed"`).

If you cannot use Gatekeeper, write the policy as a documented standard with
a validation script.

### Part D -- Seccomp and AppArmor Profiles

1. Write a custom seccomp profile that:
   - Allows common application syscalls (read, write, open, close, mmap,
     brk, etc.).
   - Denies dangerous syscalls (mount, reboot, kexec_load, ptrace,
     userfaultfd).
   - Uses `SCMP_ACT_ERRNO` for denied syscalls.

2. Document how to apply this profile to a Deployment.

3. Write an AppArmor profile for the nginx container that:
   - Allows reading from `/etc/nginx/`.
   - Allows writing to `/tmp/` and `/var/cache/nginx/`.
   - Denies writing to `/etc/`, `/proc/`, `/sys/`.
   - Denies network access except TCP on port 8080.

4. Write the pod annotation to apply the AppArmor profile.

### Part E -- Runtime Security (Network Policies)

Write NetworkPolicies for the `prod-api` namespace that:

1. **Default deny** all ingress and egress traffic.
2. Allow ingress from:
   - The ingress controller namespace (port 8080).
   - The `prod-worker` namespace (port 8080).
3. Allow egress to:
   - DNS (port 53 to kube-dns).
   - The `prod-worker` namespace (any port).
   - External HTTPS (port 443) for API calls to third-party services.
4. Deny all other traffic.

### Part F -- Validation Checklist

Create a checklist of `kubectl` commands that verify every policy is in
place. Group them by category (PSA, RBAC, image, network).

## Success Criteria

- [ ] All namespaces exist with correct PSA labels.
- [ ] RBAC manifests pass `kubectl auth can-i` checks for all three teams.
- [ ] The image policy correctly rejects `latest` tags and untrusted
      registries.
- [ ] The seccomp profile blocks dangerous syscalls while allowing normal
      application operation.
- [ ] The AppArmor profile restricts filesystem and network access as
      specified.
- [ ] The NetworkPolicy blocks unexpected traffic while allowing required
      flows.
- [ ] The validation checklist covers every policy area.
- [ ] All manifests are valid YAML and can be applied to a real cluster.

## Hints

<details>
<summary>Hint 1 -- Gatekeeper constraint</summary>
If you have Gatekeeper installed, the K8sAllowedRepos template is a good
starting point. Otherwise, write an OPA Rego policy or a validating webhook
in any language.
</details>

<details>
<summary>Hint 2 -- Seccomp syscalls</summary>
Start with the RuntimeDefault profile as a baseline. Run
`strace -c -f -p <PID>` on a running nginx process to see which syscalls
it actually uses, then build your custom profile from that list.
</details>

<details>
<summary>Hint 3 -- AppArmor syntax</summary>
AppArmor profiles use the `profile` keyword and `deny` rules. Example:
```
profile nginx-profile flags=(attach_disconnected) {
  #include <abstractions/base>
  /etc/nginx/** r,
  /tmp/** rw,
  deny /etc/** w,
  deny /proc/** w,
  deny /sys/** w,
  network inet stream,
}
```
</details>

<details>
<summary>Hint 4 -- NetworkPolicy selectors</summary>
Use `namespaceSelector` with `matchLabels` to allow traffic from specific
namespaces. Use `podSelector: {}` to select all pods in the current
namespace. Combine them with `from` and `to` fields.
</details>
