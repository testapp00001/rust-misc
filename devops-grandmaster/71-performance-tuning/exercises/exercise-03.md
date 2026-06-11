# Exercise 03: File Descriptor and Connection Limits

**Type:** Independent
**Time:** 30 minutes
**Difficulty:** Medium

## Objective
Understand file descriptor limits, configure them for high-concurrency servers, and demonstrate what happens when limits are exhausted.

## Scenario
Your Node.js server handles 10,000 concurrent WebSocket connections. Each connection uses one file descriptor. The default file descriptor limit is 1,024 (soft) and 4,096 (hard). The server crashes after approximately 1,000 connections with "EMFILE: too many open files."

## Tasks

### Part A: Check Current Limits
Write the commands to check:
1. The current soft and hard file descriptor limits for the shell
2. The system-wide file descriptor maximum
3. The file descriptor limits for a specific process (by PID)

<details>
<summary>Hint</summary>
Use `ulimit -n` for soft limit, `ulimit -Hn` for hard limit, `cat /proc/sys/fs/file-max` for system-wide max, and `cat /proc/<pid>/limits` for a specific process.
</details>

### Part B: Increase File Descriptor Limits
Write the configuration to:
1. Set the system-wide file descriptor maximum to 2,097,152
2. Set per-user soft and hard limits to 1,048,576
3. Set the systemd service limit for your application

<details>
<summary>Hint</summary>
System-wide: set `fs.file-max` and `fs.nr_open` in `/etc/sysctl.d/`. Per-user: set `nofile` in `/etc/security/limits.conf`. Systemd: set `LimitNOFILE` in the service file.
</details>

### Part C: Demonstrate File Descriptor Exhaustion
Write a Python script that:
1. Opens sockets until it hits the file descriptor limit
2. Catches the error and reports how many sockets were opened
3. Increases the limit
4. Opens sockets again to verify the new limit works

<details>
<summary>Hint</summary>
Use `socket.socket()` to create sockets in a loop. Catch `OSError` when the limit is reached. Use `resource.setrlimit()` to increase the limit programmatically.
</details>

### Part D: Configure Container File Descriptor Limits
Write Docker and Kubernetes configurations that:
1. Set file descriptor limits for a container
2. Set `sysctl` parameters inside the container
3. Verify the limits are applied correctly

<details>
<summary>Hint</summary>
Docker: use `--ulimits nofile=65536:65536`. Kubernetes: set `securityContext` and use init containers to set sysctl. Verify with `docker exec <container> sh -c "ulimit -n"`.
</details>

## Success Criteria
- [ ] You can check current file descriptor limits at the shell, system, and process level.
- [ ] You can increase limits via sysctl, limits.conf, and systemd.
- [ ] The exhaustion script correctly demonstrates the failure and recovery.
- [ ] Container limits are configured correctly in Docker and Kubernetes.
- [ ] You can explain the relationship between soft limits, hard limits, and system-wide limits.

## What You Should Understand After This Exercise
File descriptors are a fundamental resource that limits concurrent connections. The default limits (1,024 soft) are far too low for servers. Increasing limits requires changes at three levels: system-wide (sysctl), per-user (limits.conf), and per-service (systemd). Containers inherit limits from the host but can have their own limits configured.
