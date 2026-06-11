# 71 - Performance Tuning

> **Previous:** [70 - Caching Strategies](../70-caching-strategies/README.md)
> **Next:** [72 - Multi-Host Orchestration](../72-multi-host-orchestration/README.md)

## The Problem

Your application is architecturally sound — load balanced, cached, containerized — but performance is still disappointing. Requests take longer than they should. The server has plenty of CPU and memory, yet throughput is low. Network connections drop under load. File descriptors run out. The Linux kernel defaults are designed for desktop use, not high-throughput server workloads.

The operating system and its kernel are the foundation everything runs on. Default kernel parameters are conservative — optimized for compatibility, not performance. Tuning these parameters can yield 2-10x improvements in throughput, latency, and connection capacity without changing a single line of application code.

---

## The Naive Way

Deploy with default OS settings and throw hardware at the problem.

```bash
# "Just add more servers" — the default sysctl settings work fine, right?
# Default vm.swappiness = 60 (swaps aggressively)
# Default net.core.somaxconn = 128 (tiny listen backlog)
# Default fs.file-max = 65535 (runs out under load)
# Default net.ipv4.tcp_fin_timeout = 60 (holds connections too long)
```

**Why this fails:**
- The kernel swaps out application memory even when plenty of RAM is available (vm.swappiness=60).
- The TCP listen backlog is only 128 — connections get dropped under burst traffic.
- File descriptor limits are hit with a few thousand concurrent connections.
- TCP connection recycling is slow, leaving sockets in TIME_WAIT too long.
- You pay for more hardware to compensate for software misconfiguration.
- Each new server deployed inherits the same bad defaults.

---

## The Right Way

Tune kernel parameters, network stack, and resource limits systematically.

### Linux Kernel Tuning (sysctl)

The `sysctl` interface lets you tune kernel parameters at runtime. Changes persist across reboots when added to `/etc/sysctl.conf` or `/etc/sysctl.d/`.

**Memory Management:**

```bash
# /etc/sysctl.d/99-performance.conf

# Reduce swappiness — prefer keeping application data in RAM
# 0 = never swap if memory is available (aggressive)
# 10 = minimal swapping (recommended for servers)
# 60 = default (desktop-oriented)
vm.swappiness = 10

# Increase the tendency to reclaim inode and dentry cache
vm.vfs_cache_pressure = 50

# Set dirty page ratio — how much dirty memory before writeback starts
vm.dirty_ratio = 10
vm.dirty_background_ratio = 5

# Overcommit memory
vm.overcommit_memory = 0

# Minimum free memory (in KB) — kernel starts reclaiming below this
vm.min_free_kbytes = 65536
```

**Network Stack:**

```bash
# TCP connection backlog
net.core.somaxconn = 65535

# Maximum number of packets queued on the INPUT side
net.core.netdev_max_backlog = 65536

# TCP buffer sizes (min, default, max in bytes)
net.core.rmem_max = 16777216
net.core.wmem_max = 16777216
net.core.rmem_default = 262144
net.core.wmem_default = 262144
net.ipv4.tcp_rmem = 4096 87380 16777216
net.ipv4.tcp_wmem = 4096 65536 16777216

# TCP connection recycling
net.ipv4.tcp_fin_timeout = 15
net.ipv4.tcp_tw_reuse = 1
net.ipv4.tcp_max_tw_buckets = 65536

# TCP keepalive — detect dead connections faster
net.ipv4.tcp_keepalive_time = 300
net.ipv4.tcp_keepalive_intvl = 30
net.ipv4.tcp_keepalive_probes = 5

# Increase local port range for outbound connections
net.ipv4.ip_local_port_range = 1024 65535

# SYN flood protection
net.ipv4.tcp_syncookies = 1
net.ipv4.tcp_max_syn_backlog = 65535

# Enable TCP Fast Open
net.ipv4.tcp_fastopen = 3

# Congestion control algorithm
net.ipv4.tcp_congestion_control = bbr
net.core.default_qdisc = fq

# Disable slow start after idle
net.ipv4.tcp_slow_start_after_idle = 0

# Enable window scaling and SACK
net.ipv4.tcp_window_scaling = 1
net.ipv4.tcp_sack = 1
```

**Apply changes:**

```bash
# Apply all sysctl changes
sudo sysctl --system

# Or apply a specific file
sudo sysctl -p /etc/sysctl.d/99-performance.conf

# Verify a specific parameter
sysctl net.core.somaxconn
cat /proc/sys/net/core/somaxconn
```

### File Descriptor Limits

Every open file, socket, and pipe consumes a file descriptor. Default limits are too low for servers.

```bash
# Check current limits
ulimit -n          # soft limit for current shell
ullimit -Hn         # hard limit for current shell
cat /proc/sys/fs/file-max  # system-wide maximum

# Temporary increase (current session)
ulimit -n 1048576

# Permanent increase — /etc/security/limits.conf
*           soft    nofile    1048576
*           hard    nofile    1048576
root        soft    nofile    1048576
root        hard    nofile    1048576

# System-wide limit — /etc/sysctl.d/99-file-limits.conf
fs.file-max = 2097152
fs.nr_open = 2097152
```

```bash
# For systemd services
# /etc/systemd/system/myapp.service.d/limits.conf
[Service]
LimitNOFILE=1048576
LimitNPROC=1048576
```

### CPU Scheduling

```bash
# Set CPU governor to performance mode
sudo cpupower frequency-set -g performance

# For all CPUs
for cpu in /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor; do
    echo performance | sudo tee $cpu
done

# NUMA awareness — pin processes to specific CPU nodes
numactl --cpunodebind=0 --membind=0 ./my_server

# Check NUMA topology
numactl --hardware
lscpu | grep NUMA
```

### Container Resource Tuning

Docker and container runtimes have their own resource constraints that interact with kernel settings.

```yaml
# docker-compose.yml — production resource limits
services:
  api:
    image: myapp:latest
    deploy:
      resources:
        limits:
          cpus: '4.0'
          memory: 8G
        reservations:
          cpus: '2.0'
          memory: 4G
    ulimits:
      nofile:
        soft: 65536
        hard: 65536
    sysctls:
      net.core.somaxconn: 65535
      net.ipv4.tcp_tw_reuse: 1
    read_only: true
    tmpfs:
      - /tmp:size=256M
```

**Kubernetes resource tuning:**

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
          image: myapp:latest
          resources:
            requests:
              cpu: "2"
              memory: "4Gi"
            limits:
              cpu: "4"
              memory: "8Gi"
          env:
            - name: GOMAXPROCS
              valueFrom:
                resourceFieldRef:
                  resource: limits.cpu
          securityContext:
            capabilities:
              add:
                - NET_BIND_SERVICE
```

### Disk I/O Tuning

```bash
# Check current I/O scheduler
cat /sys/block/sda/queue/scheduler

# Set I/O scheduler
# For SSDs: none (noop) or mq-deadline
# For HDDs: bfq or mq-deadline
echo "none" | sudo tee /sys/block/sda/queue/scheduler

# Increase read-ahead buffer for sequential workloads
sudo blockdev --setra 4096 /dev/sda

# Mount options for performance (/etc/fstab)
/dev/sda1 / ext4 defaults,noatime,nodiratime,discard 0 1

# For XFS (better for large files and high throughput)
/dev/sda1 /data xfs defaults,noatime,logbufs=8,logbsize=256k 0 0
```

### Monitoring Performance

```bash
# System-wide performance overview
vmstat 1 10          # CPU, memory, I/O, system activity
iostat -xz 1 10      # Detailed disk I/O statistics
sar -n DEV 1 10      # Network interface statistics
mpstat -P ALL 1 10   # Per-CPU statistics

# Detailed analysis tools
perf stat -d ./my_application   # Hardware performance counters
perf top                         # Real-time CPU profiling
strace -c -p <pid>              # System call profiling

# Network diagnostics
ss -s                    # Socket statistics summary
ss -tuln                 # Listening sockets
ethtool eth0             # NIC settings and capabilities

# Memory diagnostics
free -h                  # Memory usage summary
cat /proc/meminfo        # Detailed memory information
slabtop                  # Kernel slab allocator statistics
```

---

## The Production Way

### Performance Tuning Playbook

Automate performance tuning with a configuration management tool:

```yaml
# Ansible playbook for performance tuning
- name: Performance tuning
  hosts: all
  become: yes
  tasks:
    - name: Set sysctl parameters
      sysctl:
        name: "{{ item.key }}"
        value: "{{ item.value }}"
        sysctl_file: /etc/sysctl.d/99-performance.conf
        reload: yes
      loop:
        - { key: "vm.swappiness", value: "10" }
        - { key: "net.core.somaxconn", value: "65535" }
        - { key: "net.ipv4.tcp_fin_timeout", value: "15" }
        - { key: "net.ipv4.tcp_tw_reuse", value: "1" }
        - { key: "net.ipv4.tcp_fastopen", value: "3" }
        - { key: "fs.file-max", value: "2097152" }

    - name: Set file descriptor limits
      pam_limits:
        domain: '*'
        limit_type: "{{ item.type }}"
        limit_item: nofile
        value: "1048576"
      loop:
        - { type: "soft" }
        - { type: "hard" }

    - name: Enable BBR congestion control
      sysctl:
        name: "{{ item.key }}"
        value: "{{ item.value }}"
      loop:
        - { key: "net.core.default_qdisc", value: "fq" }
        - { key: "net.ipv4.tcp_congestion_control", value: "bbr" }
```

### Profiling and Benchmarking

```bash
# Benchmark network throughput
iperf3 -s &                    # Start server
iperf3 -c <server_ip> -t 30   # 30-second TCP test

# Benchmark HTTP endpoints
wrk -t12 -c400 -d30s http://localhost:8080/api/users
hey -n 10000 -c 200 http://localhost:8080/api/users

# Benchmark disk I/O
fio --name=randwrite --ioengine=libaio --direct=1 --bs=4k \
    --size=1G --numjobs=4 --runtime=60 --rw=randwrite \
    --filename=/data/testfile

# Benchmark Redis
redis-benchmark -h localhost -p 6379 -c 50 -n 100000 -q
```

---

## Hands-On Lab

### Exercise 1: Measure Impact of sysctl Tuning

```bash
# Start a simple HTTP server
python3 -m http.server 8080 &

# Benchmark with default settings
wrk -t4 -c100 -d10s http://localhost:8080/

# Tune somaxconn
sudo sysctl -w net.core.somaxconn=65535
sudo sysctl -w net.ipv4.tcp_fin_timeout=15
sudo sysctl -w net.ipv4.tcp_tw_reuse=1

# Benchmark again
wrk -t4 -c100 -d10s http://localhost:8080/
```

### Exercise 2: File Descriptor Exhaustion

```python
# fd_exhaustion.py — demonstrates what happens when FDs run out
import socket
import resource

soft, hard = resource.getrlimit(resource.RLIMIT_NOFILE)
print(f"Current FD limits: soft={soft}, hard={hard}")

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

resource.setrlimit(resource.RLIMIT_NOFILE, (65536, 65536))
soft, hard = resource.getrlimit(resource.RLIMIT_NOFILE)
print(f"New FD limits: soft={soft}, hard={hard}")
```

### Exercise 3: TCP Congestion Control Comparison

```bash
# Check available congestion control algorithms
sysctl net.ipv4.tcp_available_congestion_control

# Test with cubic (default)
sudo sysctl -w net.ipv4.tcp_congestion_control=cubic
iperf3 -c <server> -t 10 -P 4

# Test with BBR
sudo sysctl -w net.ipv4.tcp_congestion_control=bbr
iperf3 -c <server> -t 10 -P 4
```

### Exercise 4: Container Resource Limits

```bash
# Run a container with tight resource limits
docker run -d --name stress-test \
  --cpus="1.0" \
  --memory="256m" \
  --ulimits nofile=65536:65536 \
  --sysctl net.core.somaxconn=65535 \
  nginx:alpine

# Monitor container resource usage
docker stats stress-test

# Test file descriptor limits inside the container
docker exec stress-test sh -c "ulimit -n"

# Stress test within limits
docker exec stress-test ab -n 10000 -c 500 http://localhost/
```

---

## Limitation

All performance tuning in this lesson targets a single host. When your application outgrows one machine — you need horizontal scaling, distributed databases, or multi-region deployments — single-host tuning is not enough. You need to coordinate services across multiple hosts, manage overlay networks, and handle the complexity of distributed state.

---

## Next Topic

[72 - Multi-Host Orchestration](../72-multi-host-orchestration/README.md) — Orchestrate containers across multiple hosts using Docker Swarm and Nomad, and understand how they compare to Kubernetes.
