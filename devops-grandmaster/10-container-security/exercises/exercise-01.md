# Exercise 01: Why Running as Root in Containers Is Dangerous

**Type:** Conceptual
**Time:** 15 minutes
**Difficulty:** Easy

## Objective

Understand why the default container configuration (running as root) is a security risk, and explain the specific attack vectors it exposes. You will examine real container behavior to see these risks firsthand.

## Background

Docker containers run as root by default. Many developers assume that container isolation makes this safe -- that root inside a container is somehow "contained." This assumption is wrong. Containers share the host kernel, and a root process inside a container has significant power over that shared kernel.

## Tasks

### Part A: Observe Default Root Behavior

Run the following commands and record the output:

```bash
# Check who the default container user is
docker run --rm alpine whoami

# Check the effective capabilities of a root container process
docker run --rm alpine sh -c 'cat /proc/1/status | grep -i cap'
```

Answer these questions:

1. What user does the container process run as?
2. What does `CapEff` represent, and what does a non-zero value tell you?

<details>
<summary>Hint</summary>

`CapEff` stands for "Effective Capabilities." It is a bitmask that represents which Linux capabilities the process currently has. A non-zero value means the process has real privileges -- it is not fully restricted.

</details>

### Part B: Compare Root vs Non-Root Capabilities

Run these commands and compare the output:

```bash
# Root container capabilities
docker run --rm alpine sh -c 'cat /proc/1/status | grep CapEff'

# Non-root container capabilities
docker run --rm --user 1001:1001 alpine sh -c 'cat /proc/1/status | grep CapEff'
```

1. How do the capability values differ?
2. Why does changing the user ID reduce capabilities?

<details>
<summary>Hint</summary>

Docker's default seccomp profile drops many capabilities for non-root users. The kernel treats UID 0 (root) specially -- it grants capabilities that non-root UIDs do not receive, even inside a container. The user namespace mapping means UID 1001 inside the container is a regular unprivileged user from the kernel's perspective.

</details>

### Part C: Demonstrate a Container Escape Risk

Consider this scenario: a developer runs a web application container with the Docker socket mounted:

```bash
docker run --rm -v /var/run/docker.sock:/var/run/docker.sock alpine sh -c \
  'apk add -q curl && curl -s --unix-socket /var/run/docker.sock http://localhost/v1.24/containers/json | head -c 200'
```

1. What does this command do?
2. Why is mounting the Docker socket into a container equivalent to giving the container root access on the host?
3. How does running as root inside the container make this worse?

<details>
<summary>Hint</summary>

The Docker socket is the API endpoint for the Docker daemon, which runs as root on the host. Any process that can access the socket can create new containers, mount host filesystems, and effectively do anything root can do on the host. If the container is already running as root, there are fewer barriers to exploiting this access.

</details>

### Part D: Identify the Risks

For each of the following default container behaviors, explain in one sentence why it is a security risk:

1. Running as root (UID 0)
2. Writable filesystem
3. Default capabilities enabled
4. No seccomp profile (or the default one)
5. No resource limits

<details>
<summary>Hint</summary>

Think about what an attacker could do if they gained code execution inside the container. Each default behavior gives the attacker an additional tool: root lets them install software and modify system files, a writable filesystem lets them persist malware, capabilities let them interact with the kernel, and no resource limits let them consume all host resources (denial of service).

</details>

---

## Success Criteria

- [ ] You can explain why root inside a container is not equivalent to root on the host, but is still dangerous.
- [ ] You can describe at least three specific attack vectors enabled by running as root.
- [ ] You can explain the relationship between Linux capabilities and the root user.
- [ ] You can explain why mounting the Docker socket is a critical security risk.
- [ ] You understand that container security requires multiple layers (defense in depth).

## What You Should Understand After This Exercise

Running as root inside a container is not safe just because it is inside a container. The container shares the host kernel, and root access inside the container gives an attacker significant leverage to exploit kernel vulnerabilities, modify the container environment, and potentially escape to the host. Every container should run as a non-root user unless there is a compelling technical reason not to -- and even then, compensating controls must be in place.
