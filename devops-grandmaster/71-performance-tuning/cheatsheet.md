# Cheatsheet: Performance Tuning

## Linux Kernel Tuning
```bash
# /etc/sysctl.conf

# Network
net.core.somaxconn = 65535
net.ipv4.tcp_max_syn_backlog = 65535
net.ipv4.ip_local_port_range = 1024 65535
net.ipv4.tcp_tw_reuse = 1
net.core.netdev_max_backlog = 65535

# File system
fs.file-max = 2097152
fs.inotify.max_user_watches = 524288

# Memory
vm.swappiness = 10
vm.overcommit_memory = 1

# Apply
sysctl -p
```

## File Descriptor Limits
```bash
# /etc/security/limits.conf
* soft nofile 1048576
* hard nofile 1048576
* soft nproc 1048576
* hard nproc 1048576
```

## Container Resource Tuning
```yaml
resources:
  requests:
    cpu: 100m
    memory: 128Mi
  limits:
    cpu: 500m
    memory: 256Mi
```

## Monitoring Performance
```bash
# CPU
top
htop
mpstat -P ALL 1

# Memory
free -m
vmstat 1

# I/O
iostat -x 1
iotop

# Network
iftop
nethogs

# Container
docker stats
kubectl top pods
```
