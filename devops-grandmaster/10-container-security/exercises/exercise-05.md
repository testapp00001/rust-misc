# Exercise 05: Design a Container Security Policy for a Production Environment

**Type:** Integration
**Time:** 45-60 minutes
**Difficulty:** Hard

## Objective

Design a comprehensive container security policy document and implement it as enforceable configuration. This exercise combines everything from the module: non-root users, read-only filesystems, capability management, security profiles, image scanning, secrets management, and network isolation. You will produce a policy document, a set of enforceable templates, and an audit script.

## Scenario

You are the platform engineer at a mid-size company. The engineering team runs 15 microservices in Docker Compose (with plans to migrate to Kubernetes). The security team has asked you to create a container security policy that:

1. Defines mandatory security requirements for all containers.
2. Provides templates that teams can copy and adapt.
3. Includes an automated audit tool that checks compliance.
4. Covers the full lifecycle: build, ship, run.

## Tasks

### Part A: Write the Policy Document

Create a file called `container-security-policy.md` that defines the company's container security requirements. Organize it into these sections:

**1. Image Requirements**
- Base image restrictions (which images are allowed, versioning policy).
- Vulnerability scanning requirements (which tool, severity thresholds).
- Image signing requirements.
- Dockerfile best practices.

**2. Runtime Requirements**
- User requirements (non-root, specific UID ranges).
- Filesystem requirements (read-only, allowed writable paths).
- Capability policy (default: drop all, allowed exceptions).
- Security profiles (seccomp, AppArmor).
- Resource limits (memory, CPU).

**3. Network Requirements**
- Network segmentation rules.
- Port exposure policy.
- DNS policy.

**4. Secrets Management**
- How secrets must be stored and delivered.
- What is forbidden (ENV vars, baked into images).
- Rotation requirements.

**5. Monitoring and Audit**
- Logging requirements.
- Compliance audit frequency.
- Incident response.

For each requirement, specify:
- The requirement itself (what must be done).
- The rationale (why).
- How to verify compliance.

<details>
<summary>Hint</summary>

Use the Security Checklist from the module README as a starting point. Expand each checklist item into a full requirement with rationale and verification method. Think about what a new developer would need to understand to comply with each rule.

</details>

### Part B: Create the Reference Template

Create a file called `docker-compose.template.yml` that serves as the "golden template" -- the starting point for any new service. It should include every security measure from the policy, with comments explaining each setting.

The template should cover:
- A web application service with full security hardening.
- A database service with appropriate (different) hardening.
- Shared infrastructure (networks, secrets, volumes).
- Inline comments referencing the policy section for each setting.

<details>
<summary>Hint</summary>

Start from the production docker-compose.yml in the module README. Add comments that map each setting to a policy requirement. Use placeholder values (like `YOUR_IMAGE:VERSION`) for things teams must customize.

</details>

### Part C: Create the Reference Dockerfile

Create a file called `Dockerfile.template` that serves as the golden Dockerfile template. It should demonstrate every Dockerfile-level security best practice:

- Minimal base image with pinned version.
- Non-root user creation.
- Proper COPY ordering for layer caching.
- Health check.
- No secrets in build args or environment.
- `.dockerignore` guidance in comments.

<details>
<summary>Hint</summary>

Use multi-stage builds if the application needs build tools. The final stage should contain only the runtime dependencies. Add comments explaining each security decision.

</details>

### Part D: Build the Compliance Audit Script

Create a script called `audit-containers.sh` that checks all running containers against the policy. The script should:

1. Iterate over all running containers.
2. For each container, check:
   - Running user (not root).
   - Capabilities (should be minimal).
   - Filesystem (read-only where possible).
   - Secrets in environment variables.
   - Docker socket mounts.
   - Resource limits.
   - Network mode.
   - Privileged mode.
3. Output a compliance report in a structured format.
4. Exit with code 0 if all containers are compliant, code 1 otherwise.

The output should look like:

```
=== Container Security Compliance Report ===
Date: 2024-01-15T10:30:00Z

Container: web-app
  [PASS] Non-root user
  [PASS] Capabilities dropped
  [PASS] Read-only filesystem
  [FAIL] Secrets in environment variables
  [PASS] No Docker socket mount
  [PASS] Resource limits set
  [PASS] Isolated network
  [PASS] Not privileged
  Compliance: 7/8 (87%)

Container: database
  [PASS] Non-root user
  [FAIL] Capabilities not fully dropped
  [PASS] Read-only filesystem
  [PASS] No secrets in ENV
  [PASS] No Docker socket mount
  [PASS] Resource limits set
  [PASS] Isolated network
  [PASS] Not privileged
  Compliance: 7/8 (87%)

=== Overall: 14/16 checks passed (87%) ===
```

<details>
<summary>Hint 1: Container iteration</summary>

Use `docker ps --format '{{.Names}}'` to get container names, then iterate with a for loop. For each container, use `docker inspect` with Go templates to extract the relevant fields.

</details>

<details>
<summary>Hint 2: Checking secrets in ENV</summary>

Use `docker inspect --format '{{.Config.Env}}'` and grep for patterns like `PASSWORD`, `SECRET`, `KEY`, `TOKEN`. This is a heuristic -- it may produce false positives for non-secret values like `NODE_ENV=production`, but it is a reasonable starting point.

</details>

<details>
<summary>Hint 3: Checking capabilities</summary>

Use `docker exec <container> cat /proc/1/status | grep CapEff` to get the effective capabilities bitmask. A value of `0000000000000000` means all capabilities are dropped.

</details>

---

## Success Criteria

- [ ] The policy document covers all five sections (image, runtime, network, secrets, monitoring) with requirements, rationale, and verification methods.
- [ ] The docker-compose template includes every security measure and has comments mapping each setting to a policy requirement.
- [ ] The Dockerfile template follows all best practices with explanatory comments.
- [ ] The audit script correctly identifies security issues in running containers.
- [ ] The audit script produces a clear, structured compliance report and exits with the correct exit code.

## What You Should Understand After This Exercise

A container security policy is not a document that sits on a shelf. It is a living system: a policy document that defines the rules, templates that make compliance easy, and automation that enforces compliance continuously. The best security policy is one where doing the right thing is easier than doing the wrong thing -- templates and automation make that possible.
