# Solution 05: Full System Performance Playbook

## Part A: Ansible Playbook

```yaml
# performance-tuning.yml — Ansible playbook for server performance tuning
---
- name: Performance Tuning for High-Throughput Servers
  hosts: all
  become: yes
  vars:
    sysctl_params:
      vm.swappiness: "10"
      vm.vfs_cache_pressure: "50"
      vm.dirty_ratio: "10"
      vm.dirty_background_ratio: "5"
      vm.min_free_kbytes: "65536"
      net.core.somaxconn: "65535"
      net.core.netdev_max_backlog: "65536"
      net.ipv4.tcp_rmem: "4096 87380 16777216"
      net.ipv4.tcp_wmem: "4096 65536 16777216"
      net.core.rmem_max: "16777216"
      net.core.wmem_max: "16777216"
      net.ipv4.tcp_fin_timeout: "15"
      net.ipv4.tcp_tw_reuse: "1"
      net.ipv4.tcp_keepalive_time: "300"
      net.ipv4.tcp_keepalive_intvl: "30"
      net.ipv4.tcp_keepalive_probes: "5"
      net.ipv4.ip_local_port_range: "1024 65535"
      net.ipv4.tcp_syncookies: "1"
      net.ipv4.tcp_max_syn_backlog: "65535"
      net.ipv4.tcp_congestion_control: "bbr"
      net.core.default_qdisc: "fq"
      net.ipv4.tcp_fastopen: "3"
      net.ipv4.tcp_slow_start_after_idle: "0"
      net.ipv4.tcp_window_scaling: "1"
      net.ipv4.tcp_sack: "1"
      fs.file-max: "2097152"
      fs.nr_open: "2097152"

    fd_soft_limit: "1048576"
    fd_hard_limit: "1048576"

  tasks:
    - name: Load BBR kernel module
      modprobe:
        name: tcp_bbr
        state: present

    - name: Ensure BBR module loads on boot
      lineinfile:
        path: /etc/modules-load.d/bbr.conf
        line: tcp_bbr
        create: yes

    - name: Apply sysctl parameters
      sysctl:
        name: "{{ item.key }}"
        value: "{{ item.value }}"
        sysctl_file: /etc/sysctl.d/99-performance.conf
        reload: yes
        state: present
      loop: "{{ sysctl_params | dict2items }}"

    - name: Set file descriptor limits (soft)
      pam_limits:
        domain: '*'
        limit_type: soft
        limit_item: nofile
        value: "{{ fd_soft_limit }}"
        dest: /etc/security/limits.conf

    - name: Set file descriptor limits (hard)
      pam_limits:
        domain: '*'
        limit_type: hard
        limit_item: nofile
        value: "{{ fd_hard_limit }}"
        dest: /etc/security/limits.conf

    - name: Set CPU governor to performance
      shell: |
        for cpu in /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor; do
          echo performance > $cpu 2>/dev/null || true
        done
      when: ansible_os_family == "Debian" or ansible_os_family == "RedHat"

    - name: Install cpupower (Debian)
      apt:
        name: linux-tools-common
        state: present
      when: ansible_os_family == "Debian"

    - name: Run validation
      include_tasks: validate.yml
```

---

## Part B: Validation Script

```bash
#!/bin/bash
# validate-performance.sh — Validate performance tuning settings
set -euo pipefail

PASS=0
FAIL=0

check() {
    local name="$1"
    local expected="$2"
    local actual="$3"

    if [ "$actual" = "$expected" ]; then
        echo "  PASS: $name = $actual"
        PASS=$((PASS + 1))
    else
        echo "  FAIL: $name = $actual (expected $expected)"
        FAIL=$((FAIL + 1))
    fi
}

echo "=== Performance Tuning Validation ==="
echo ""

# sysctl parameters
echo "--- sysctl Parameters ---"
check "vm.swappiness" "10" "$(sysctl -n vm.swappiness)"
check "net.core.somaxconn" "65535" "$(sysctl -n net.core.somaxconn)"
check "net.ipv4.tcp_fin_timeout" "15" "$(sysctl -n net.ipv4.tcp_fin_timeout)"
check "net.ipv4.tcp_tw_reuse" "1" "$(sysctl -n net.ipv4.tcp_tw_reuse)"
check "net.ipv4.tcp_congestion_control" "bbr" "$(sysctl -n net.ipv4.tcp_congestion_control)"
check "net.core.default_qdisc" "fq" "$(sysctl -n net.core.default_qdisc)"
check "net.ipv4.tcp_fastopen" "3" "$(sysctl -n net.ipv4.tcp_fastopen)"
check "fs.file-max" "2097152" "$(sysctl -n fs.file-max)"

echo ""

# File descriptor limits
echo "--- File Descriptor Limits ---"
CURRENT_SOFT=$(ulimit -n)
if [ "$CURRENT_SOFT" -ge 1048576 ]; then
    echo "  PASS: ulimit -n = $CURRENT_SOFT (>= 1048576)"
    PASS=$((PASS + 1))
else
    echo "  FAIL: ulimit -n = $CURRENT_SOFT (expected >= 1048576)"
    FAIL=$((FAIL + 1))
fi

echo ""

# BBR module
echo "--- BBR Module ---"
if lsmod | grep -q tcp_bbr; then
    echo "  PASS: tcp_bbr module is loaded"
    PASS=$((PASS + 1))
else
    echo "  FAIL: tcp_bbr module is not loaded"
    FAIL=$((FAIL + 1))
fi

echo ""

# CPU governor
echo "--- CPU Governor ---"
GOVERNOR=$(cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor 2>/dev/null || echo "unknown")
check "CPU governor" "performance" "$GOVERNOR"

echo ""

# Summary
echo "=== Results ==="
echo "PASS: $PASS"
echo "FAIL: $FAIL"

if [ "$FAIL" -gt 0 ]; then
    echo ""
    echo "VALIDATION FAILED: $FAIL check(s) did not pass"
    exit 1
else
    echo ""
    echo "ALL CHECKS PASSED"
    exit 0
fi
```

---

## Part C: Monitoring Configuration

```yaml
# prometheus-alerts.yml — Performance tuning alerts
groups:
  - name: performance-tuning
    rules:
      - alert: FileDescriptorUsageHigh
        expr: node_filefd_allocated / node_filefd_maximum > 0.8
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "File descriptor usage above 80%"
          description: "{{ $labels.instance }} has {{ $value | humanizePercentage }} FD usage"

      - alert: TCPConnectionCountHigh
        expr: node_netstat_Tcp_CurrEstab > 50000
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "TCP connections above 50,000"

      - alert: TCPRetransmissionRateHigh
        expr: rate(node_netstat_Tcp_RetransSegs[5m]) / rate(node_netstat_Tcp_OutSegs[5m]) > 0.01
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "TCP retransmission rate above 1%"

      - alert: CPUStealTimeHigh
        expr: rate(node_cpu_seconds_total{mode="steal"}[5m]) > 0.05
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "CPU steal time above 5% (noisy neighbor)"

      - alert: MemorySwapUsage
        expr: node_memory_SwapTotal_bytes - node_memory_SwapFree_bytes > 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "System is swapping (any swap usage detected)"
```

---

## Part D: Rollback Script

```bash
#!/bin/bash
# rollback-performance.sh — Restore default kernel settings
set -euo pipefail

LOGFILE="/var/log/performance-rollback.log"
BACKUP_DIR="/etc/sysctl.d/backup"

log() {
    echo "$(date -u +%Y-%m-%dT%H:%M:%SZ) $1" | tee -a "$LOGFILE"
}

log "=== Performance Tuning Rollback Started ==="

# Restore sysctl defaults
log "Restoring sysctl defaults..."
cat > /etc/sysctl.d/99-performance.conf << 'EOF'
# Restored to Linux defaults
vm.swappiness = 60
vm.vfs_cache_pressure = 100
vm.dirty_ratio = 20
vm.dirty_background_ratio = 10
net.core.somaxconn = 128
net.core.netdev_max_backlog = 1000
net.ipv4.tcp_rmem = 4096 131072 6291456
net.ipv4.tcp_wmem = 4096 16384 4194304
net.core.rmem_max = 212992
net.core.wmem_max = 212992
net.ipv4.tcp_fin_timeout = 60
net.ipv4.tcp_tw_reuse = 0
net.ipv4.tcp_keepalive_time = 7200
net.ipv4.tcp_keepalive_intvl = 75
net.ipv4.tcp_keepalive_probes = 9
net.ipv4.ip_local_port_range = 32768 60999
net.ipv4.tcp_congestion_control = cubic
net.core.default_qdisc = pfifo_fast
net.ipv4.tcp_fastopen = 0
net.ipv4.tcp_slow_start_after_idle = 1
net.ipv4.tcp_window_scaling = 1
net.ipv4.tcp_sack = 1
fs.file-max = 65535
EOF

sudo sysctl --system
log "sysctl defaults restored"

# Restore file descriptor limits
log "Restoring file descriptor limits..."
cat > /etc/security/limits.conf << 'EOF'
# Restored to defaults
*           soft    nofile    1024
*           hard    nofile    4096
root        soft    nofile    1024
root        hard    nofile    4096
EOF
log "File descriptor limits restored"

# Restore CPU governor
log "Restoring CPU governor to powersave..."
for cpu in /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor; do
    echo powersave > "$cpu" 2>/dev/null || true
done
log "CPU governor restored to powersave"

log "=== Performance Tuning Rollback Complete ==="
log "NOTE: Changes take effect for new sessions. Re-login or restart services."
```

---

## Common Mistakes
1. **Not making the playbook idempotent**: Running the playbook twice should not change anything. Use `state: present` instead of `state: latest` for sysctl.
2. **Not backing up original values**: Always save the original configuration before applying changes. The rollback script needs the original values.
3. **Not validating after applying**: Always run the validation script after applying changes. Silent failures are common (e.g., sysctl parameter name typo).
4. **Forgetting to restart services**: sysctl changes apply immediately, but file descriptor limits only apply to new sessions. Restart services after changing limits.
5. **Not testing rollback**: The rollback script should be tested in staging before deploying to production. A broken rollback can leave servers in an unknown state.

## Relevant README Sections
- [Performance Tuning Playbook](../README.md#performance-tuning-playbook)
- [Profiling and Benchmarking](../README.md#profiling-and-benchmarking)
- [Hands-On Lab](../README.md#hands-on-lab)
