# Solution 03: File Descriptor and Connection Limits

## Part A: Check Current Limits

```bash
# Soft limit (current shell)
ulimit -n
# Typical output: 1024

# Hard limit (current shell)
ulimit -Hn
# Typical output: 4096

# System-wide maximum
cat /proc/sys/fs/file-max
# Typical output: 65535

# Per-process limits (by PID)
cat /proc/<pid>/limits | grep "open files"
# Output: Max open files  1024  4096  files

# Summary for current shell
ulimit -a | grep "open files"
```

**Why these matter**: The soft limit is the default for new processes. The hard limit is the maximum the soft limit can be increased to (without root). The system-wide limit is the absolute maximum across all processes.

---

## Part B: Increase File Descriptor Limits

### System-wide (sysctl)

```bash
# /etc/sysctl.d/99-file-limits.conf
fs.file-max = 2097152
fs.nr_open = 2097152
```

```bash
# Apply
sudo sysctl --system
```

### Per-user (limits.conf)

```bash
# /etc/security/limits.conf
# Set for all users
*           soft    nofile    1048576
*           hard    nofile    1048576

# Set for root specifically
root        soft    nofile    1048576
root        hard    nofile    1048576
```

### Systemd service

```bash
# /etc/systemd/system/myapp.service.d/limits.conf
[Service]
LimitNOFILE=1048576
LimitNPROC=1048576
```

```bash
# Reload systemd and restart service
sudo systemctl daemon-reload
sudo systemctl restart myapp
```

**Why all three levels**: System-wide (`fs.file-max`) sets the absolute maximum. Per-user (`limits.conf`) sets the default for user sessions. Systemd (`LimitNOFILE`) overrides for specific services. All three must be consistent.

---

## Part C: File Descriptor Exhaustion Demo

```python
#!/usr/bin/env python3
"""fd_exhaustion.py — Demonstrate file descriptor limits."""
import socket
import resource
import os

def check_limits():
    """Check current file descriptor limits."""
    soft, hard = resource.getrlimit(resource.RLIMIT_NOFILE)
    print(f"Current FD limits: soft={soft}, hard={hard}")
    print(f"System-wide max: {open('/proc/sys/fs/file-max').read().strip()}")
    return soft, hard

def exhaust_fds():
    """Open sockets until we hit the limit."""
    sockets = []
    try:
        while True:
            s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            sockets.append(s)
    except OSError as e:
        print(f"Hit FD limit after {len(sockets)} sockets: {e}")
    finally:
        for s in sockets:
            s.close()
    return len(sockets)

def increase_limit(new_soft, new_hard):
    """Increase file descriptor limit."""
    print(f"\nIncreasing limit to: soft={new_soft}, hard={new_hard}")
    resource.setrlimit(resource.RLIMIT_NOFILE, (new_soft, new_hard))
    soft, hard = resource.getrlimit(resource.RLIMIT_NOFILE)
    print(f"New FD limits: soft={soft}, hard={hard}")

# Run the demo
print("=== File Descriptor Exhaustion Demo ===\n")

print("Step 1: Check current limits")
soft, hard = check_limits()

print("\nStep 2: Exhaust file descriptors")
count = exhaust_fds()

print("\nStep 3: Increase limits")
increase_limit(65536, 65536)

print("\nStep 4: Verify new limits work")
count2 = exhaust_fds()
print(f"Now can open {count2} sockets (vs {count} before)")

print(f"\nImprovement: {count2 / count:.1f}x more connections possible")
```

**Why this works**: `socket.socket()` creates a socket, which consumes one file descriptor. The loop continues until `OSError` is raised (EMFILE). `resource.setrlimit()` programmatically increases the limit.

---

## Part D: Container File Descriptor Limits

### Docker

```bash
# Run container with file descriptor limits
docker run -d --name api-server \
  --ulimits nofile=65536:65536 \
  --sysctl net.core.somaxconn=65535 \
  --sysctl net.ipv4.tcp_tw_reuse=1 \
  nginx:alpine

# Verify limits inside container
docker exec api-server sh -c "ulimit -n"
# Expected output: 65536

# Check sysctl inside container
docker exec api-server sh -c "sysctl net.core.somaxconn"
# Expected output: net.core.somaxconn = 65535
```

### Kubernetes

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: api-server
spec:
  template:
    spec:
      containers:
        - name: api
          image: nginx:alpine
          resources:
            limits:
              cpu: "4"
              memory: "8Gi"
          # Note: ulimits are set via securityContext or init containers
          securityContext:
            capabilities:
              add:
                - NET_BIND_SERVICE
      # Use init container to set sysctl (requires privileged)
      initContainers:
        - name: sysctl-init
          image: busybox
          command:
            - sh
            - -c
            - |
              sysctl -w net.core.somaxconn=65535
              sysctl -w net.ipv4.tcp_tw_reuse=1
          securityContext:
            privileged: true
```

**Why containers need explicit configuration**: Containers do not inherit all host sysctl settings. Some sysctl parameters (like `net.core.somaxconn`) can be set per-container. Others (like `vm.swappiness`) are host-wide and cannot be changed per-container.

---

## Common Mistakes
1. **Only changing limits.conf without restarting**: Changes to `limits.conf` only apply to new login sessions. Existing processes keep their old limits. Restart the service or re-login.
2. **Setting soft limit higher than hard limit**: The soft limit cannot exceed the hard limit. Always set hard >= soft.
3. **Not increasing fs.file-max**: If `fs.file-max` is lower than your per-user limit, the system-wide limit caps your connections.
4. **Forgetting systemd limits**: Systemd services ignore `limits.conf`. You must set `LimitNOFILE` in the service file.
5. **Setting ulimits in Dockerfile instead of docker run**: `ulimit` in a Dockerfile only affects the build process. Use `--ulimits` in `docker run` or the `ulimits` key in docker-compose.

## Relevant README Sections
- [File Descriptor Limits](../README.md#file-descriptor-limits)
- [Container Resource Tuning](../README.md#container-resource-tuning)
