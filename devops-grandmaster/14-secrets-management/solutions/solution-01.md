# Solution 01: Why Environment Variables Fail for Secrets

## Part A: The Attack Surface

Four distinct methods for reading environment variables from a running container:

| Method | Access Level Required | Command / API Call |
|--------|----------------------|---------------------|
| **Docker inspect** | Docker socket access (root or `docker` group) | `docker inspect <container_id>` -- the `Config.Env` field lists all environment variables in plaintext. |
| **Process filesystem** | Root access inside the container or on the host | `cat /proc/<pid>/environ` -- the kernel exposes the process environment as a null-delimited file readable by the process owner or root. |
| **Orchestrator API** | Kubernetes API access (RBAC to read Pods) | `kubectl get pod <name> -o yaml` -- environment variables injected via `env:` appear in the pod spec. Secrets referenced via `secretKeyRef` show the reference, but inline values appear in plaintext. |
| **CI/CD logs** | Access to CI/CD platform (often all developers) | Many CI/CD systems log the environment in debug mode. A failed build step that runs `env` or `printenv` dumps every variable, including secrets, into a log file accessible to anyone with pipeline access. |

### Why this matters

These are not theoretical risks. The most common real-world secret leak is a developer running `docker inspect` on a production container to debug an issue and seeing database passwords in the output. The second most common is a CI/CD pipeline logging environment variables on a test failure, which then persist in build logs for months.

### Common mistakes

- Assuming that "only the ops team has Docker access" is sufficient protection. In practice, Docker socket access is often granted broadly for debugging purposes.
- Forgetting that `kubectl describe pod` shows environment variables (except those sourced from Secrets via `secretKeyRef`, which show the reference but not the value).
- Not realizing that `/proc/<pid>/environ` is readable by anyone who can exec into the container as root.

---

## Part B: Why "Just Use env vars" Breaks the Security Chain

**Response to the senior developer:**

Treating secrets the same as config creates three specific security problems. First, **visibility**: environment variables are exposed through `docker inspect`, `/proc/<pid>/environ`, and orchestrator APIs, meaning anyone with container or cluster access can read secrets as easily as they read `LOG_LEVEL`. Second, **persistence**: environment variables survive in CI/CD logs, Docker image layers (if set with `ENV`), and container metadata -- a secret leaked in a build log stays in the log forever, even after the password is rotated. Third, **auditability**: environment variables have no access control or audit trail. You cannot determine who read `DB_PASSWORD` from a running container, when they read it, or whether the access was authorized. A dedicated secrets manager solves all three: it encrypts values, restricts access by policy, and logs every read operation with a timestamp and identity.

### Common mistakes

- Arguing that "we trust our team" as a reason to skip secret management. The problem is not malicious insiders -- it is accidental exposure through logs, screenshots, and shared debugging sessions.
- Thinking that Kubernetes Secrets solve the problem. By default, they are base64-encoded, not encrypted, and are still injected as environment variables or files accessible to anyone with pod access.
- Confusing simplicity with security. Using env vars for everything is simpler, but simplicity that sacrifices security is technical debt that compounds over time.

---

## Part C: Classify the Exposure Risk

| Scenario | Risk Level | Reasoning |
|----------|------------|-----------|
| 1. `DB_PASSWORD` in a `docker-compose.yml` committed to a private GitHub repo | **High** | The secret is in version control history permanently. Even if removed in a future commit, it remains in the git history. Every person with repo access (and every CI system connected to the repo) can read it. Private repos are not immune -- all collaborators, service accounts, and GitHub integrations have access. |
| 2. CI/CD pipeline passes `API_KEY` as an env var, logged on failure | **Medium** | The secret is in the CI/CD logs, which are typically accessible to the development team. Risk depends on log retention policy and who has access. If logs are retained for 90 days, the exposure window is 90 days. The secret is not in version control, so it does not propagate to every clone. |
| 3. Production container with `JWT_SECRET` as env var, ops team SSH only | **Low** | The secret is only in the running container's memory. It is accessible only to those with SSH access to the host and Docker socket access. The exposure window is limited to the container's lifetime. Risk increases if the ops team is large or if SSH access is shared. |
| 4. Docker image with `ENV DB_PASSWORD=secret123`, pushed to private registry | **High** | The secret is baked into the image layer. Anyone who can pull the image can extract it with `docker history`. The secret persists even if the Dockerfile is later fixed -- the old image layers remain in the registry. Every environment that pulls this image gets the secret. |
| 5. Kubernetes pod injects `SMTP_PASSWORD` from a Secret object as env var | **Medium** | Better than hardcoding, but the secret is still visible via `kubectl describe pod` and `/proc/<pid>/environ` inside the container. The Kubernetes Secret itself is base64-encoded (not encrypted) by default. Risk depends on RBAC controls around pod access. |
| 6. Developer runs `docker inspect` on a shared dev container with prod creds | **High** | Production credentials are on a developer's machine, visible in the inspect output, and potentially saved in terminal history, screenshots, or notes. The developer may not even realize they are looking at production credentials. Shared development environments multiply the exposure. |

### Key insight

Risk is a function of three variables: **persistence** (how long the secret is exposed), **breadth** (how many people can access it), and **sensitivity** (what damage exposure causes). A secret in a git repo is high-risk because it persists forever and is accessible to everyone with repo access. A secret in a running container is lower-risk because it exists only while the container runs and is accessible only to those with host access.

---

## Part D: Design a Better Pattern

The file-based secret injection pattern:

```
Secret Store                Container                     Application
(Docker/K8s/Vault)          (runtime)                     (process)

+------------------+        +--------------------+        +---------------+
| Encrypted secret |------->| /run/secrets/      |------->| Read file at  |
| stored in Swarm  |  mount |   db_password      |  read  | startup       |
| Raft log or K8s  |  (tmpfs|   api_key          |  (fs)  |               |
| etcd)            |        +--------------------+        +---------------+
+------------------+              ^
                                  |
                          tmpfs filesystem
                          (never touches disk)
```

**1. Where the secret is stored:**

The secret lives in an encrypted secret store -- Docker Swarm's Raft log (encrypted with AES-256-GCM), Kubernetes etcd (encrypted at rest with `EncryptionConfiguration`), or an external manager like Vault. It is never stored in the image, in environment variables, or on a persistent filesystem.

**2. How it enters the container:**

The orchestrator mounts the secret as a file into the container at `/run/secrets/`. In Docker Swarm, this uses a tmpfs mount -- the secret exists only in memory and is never written to the container's writable layer. In Kubernetes, secrets can be mounted as a `secret` volume, which is also backed by tmpfs.

**3. How the application reads it:**

The application reads the secret from the file at startup:

```rust
fn read_secret(path: &str) -> Result<String, std::io::Error> {
    let content = std::fs::read_to_string(path)?;
    Ok(content.trim().to_string())
}

let db_password = read_secret("/run/secrets/db_password")?;
```

The application never calls `std::env::var("DB_PASSWORD")` -- the secret is not in the process environment.

**4. Why this is more secure:**

- `docker inspect` shows no secrets -- only the fact that secrets are mounted.
- `/proc/<pid>/environ` contains no secrets -- they are not environment variables.
- The orchestrator API shows secret metadata (name, creation date) but not the value.
- The secret exists in tmpfs, so it is not persisted to disk if the container stops.
- Access can be restricted per-service -- not every container in the stack needs every secret.

### Common mistakes

- Assuming tmpfs means the secret is completely secure. A process inside the container can still read the file. The protection is against external observation (inspect, proc, API), not against the container itself.
- Forgetting to set file permissions on the mounted secret. Docker secrets are mounted as `0444` (world-readable) by default. In production, your application should set `chmod 0400` or run as a non-root user that owns the secret file.
- Not cleaning up secret files on shutdown. If the application writes secrets to a persistent volume, the tmpfs protection is negated.
