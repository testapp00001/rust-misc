# Solution 01: Why Running as Root in Containers Is Dangerous

## Part A: Observe Default Root Behavior

**Expected output:**

```bash
$ docker run --rm alpine whoami
root

$ docker run --rm alpine sh -c 'cat /proc/1/status | grep -i cap'
CapBnd: 00000000a80425fb
CapEff: 00000000a80425fb
```

**Answers:**

1. The container process runs as `root` (UID 0).
2. `CapEff` is the "Effective Capabilities" bitmask. A non-zero value means the process has real Linux capabilities -- it can perform privileged operations like binding to low ports, changing file ownership, and interacting with network interfaces. The value `a80425fb` means many specific capabilities are enabled.

**Why this matters:** A process running as root with effective capabilities has significant power. If an attacker gains code execution inside this container (through an application vulnerability), they inherit all these capabilities and can use them to escalate their attack.

---

## Part B: Compare Root vs Non-Root Capabilities

**Expected output:**

```bash
$ docker run --rm alpine sh -c 'cat /proc/1/status | grep CapEff'
CapEff: 00000000a80425fb

$ docker run --rm --user 1001:1001 alpine sh -c 'cat /proc/1/status | grep CapEff'
CapEff: 0000000000000000
```

**Answers:**

1. The root container has `CapEff: 00000000a80425fb` (many capabilities). The non-root container has `CapEff: 0000000000000000` (no capabilities).
2. The Linux kernel treats UID 0 specially. When a process runs as root, the kernel grants it capabilities based on the capability bounding set. For non-root users, the kernel does not grant these capabilities by default. Docker's default seccomp profile further restricts what non-root processes can do. This is why changing the user ID is the single most impactful security improvement -- it removes capabilities at the kernel level.

**Why this matters:** Capabilities are the mechanism by which the kernel grants fine-grained privileges. A root process has many capabilities; a non-root process has none (by default). This is not just a Docker convention -- it is how the Linux kernel enforces privilege separation.

---

## Part C: Demonstrate a Container Escape Risk

**Expected output:**

```bash
$ docker run --rm -v /var/run/docker.sock:/var/run/docker.sock alpine sh -c \
  'apk add -q curl && curl -s --unix-socket /var/run/docker.sock http://localhost/v1.24/containers/json | head -c 200'
[{"Id":"abc123...","Names":["/test-container"],...}]
```

**Answers:**

1. This command mounts the Docker socket into the container and uses `curl` to query the Docker API from inside the container. It lists all running containers on the host.
2. The Docker socket is the Unix domain socket that the Docker CLI uses to communicate with the Docker daemon. The daemon runs as root on the host. Any process that can access this socket can issue Docker API commands: create containers, delete containers, pull images, and critically, create a container that mounts the host's root filesystem. This is equivalent to root access on the host.
3. If the container is running as root, there are fewer barriers to exploiting the socket access. A root process can install tools (like `curl` or the Docker CLI), modify the container's filesystem to persist an attack, and use the socket to create a new privileged container that mounts the host filesystem. A non-root process would have a harder time installing tools and would face additional permission barriers.

**Why this matters:** Mounting the Docker socket is the single most dangerous misconfiguration in Docker. It completely undermines container isolation. This is why the security checklist includes "Docker socket is NOT mounted into containers" as a critical item.

---

## Part D: Identify the Risks

1. **Running as root (UID 0):** A root process inside the container can install packages, modify system files, and exploit kernel vulnerabilities with full privileges, making any container compromise significantly more dangerous.
2. **Writable filesystem:** An attacker who gains code execution can write malware to disk, modify application binaries, install persistent backdoors, and tamper with configuration files.
3. **Default capabilities enabled:** Capabilities like `SYS_ADMIN`, `NET_RAW`, and `DAC_OVERRIDE` allow the process to interact with the kernel in ways that can be exploited for container escape or privilege escalation.
4. **No seccomp profile (or default):** Without a restrictive seccomp profile, the process can make any of the 300+ system calls, including ones that are only needed for container management (like `mount` or `reboot`), increasing the kernel attack surface.
5. **No resource limits:** Without memory and CPU limits, a compromised container can consume all host resources (fork bombs, memory exhaustion), causing denial of service for every other container and service on the host.

**Why this matters:** Each of these is a layer of defense. No single layer is sufficient, but together they create meaningful barriers. An attacker who breaches one layer still faces the others. This is the principle of defense in depth.

---

## Common Mistakes

- **"Containers are like VMs."** They are not. Containers share the host kernel. A kernel vulnerability affects all containers on the host.
- **"Root in a container is fine because it is isolated."** The isolation is a thin boundary (namespaces and cgroups), not a security boundary. Kernel exploits can escape containers.
- **"I will add security later."** Security must be built in from the start. Retrofitting is harder and more error-prone than designing for security from the beginning.
