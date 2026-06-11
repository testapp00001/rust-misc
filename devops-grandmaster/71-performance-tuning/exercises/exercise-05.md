# Exercise 05: Full System Performance Playbook

**Type:** Integration
**Time:** 45 minutes
**Difficulty:** Hard

## Objective
Create a complete performance tuning playbook that automates kernel tuning, file descriptor limits, network optimization, and monitoring for a production server fleet.

## Scenario
You are the platform engineer responsible for 100 application servers. Each server runs a high-throughput API service. You need to create a standardized performance tuning playbook that:
1. Tunes kernel parameters
2. Configures file descriptor limits
3. Optimizes the network stack
4. Sets up performance monitoring
5. Validates the changes
6. Can be applied to all servers automatically

## Tasks

### Part A: Create the Ansible Playbook
Write an Ansible playbook that:
1. Applies sysctl parameters from a configuration file
2. Sets file descriptor limits in limits.conf
3. Configures systemd service limits
4. Enables BBR congestion control
5. Sets the CPU governor to performance mode
6. Validates all changes

<details>
<summary>Hint</summary>
Use the `sysctl` module for kernel parameters. Use the `pam_limits` module for limits.conf. Use the `template` module for systemd service overrides. Use the `command` module for CPU governor. Use the `assert` module for validation.
</details>

### Part B: Create the Validation Script
Write a bash script that:
1. Checks all sysctl parameters are set correctly
2. Checks file descriptor limits are set correctly
3. Checks BBR congestion control is active
4. Checks CPU governor is set to performance
5. Reports PASS/FAIL for each check
6. Exits non-zero if any check fails

<details>
<summary>Hint</summary>
Use `sysctl -n <parameter>` to read current values. Compare against expected values. Use `grep` to check limits.conf. Use `cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor` to check CPU governor.
</details>

### Part C: Create the Monitoring Configuration
Write Prometheus alerting rules that detect:
1. File descriptor usage above 80%
2. TCP connection count above 50,000
3. Network retransmissions above 1%
4. CPU steal time above 5% (noisy neighbor)
5. Memory swap usage above 0 (any swapping)

<details>
<summary>Hint</summary>
Use `node_filefd_allocated / node_filefd_maximum` for FD usage. Use `node_netstat_Tcp_CurrEstab` for connections. Use `rate(node_netstat_Tcp_RetransSegs[5m]) / rate(node_netstat_Tcp_OutSegs[5m])` for retransmission rate. Use `rate(node_cpu_seconds_total{mode="steal"}[5m])` for CPU steal.
</details>

### Part D: Create the Rollback Plan
Write a rollback script that:
1. Restores all sysctl parameters to Linux defaults
2. Restores file descriptor limits to defaults
3. Restores congestion control to cubic
4. Restores CPU governor to powersave
5. Logs all changes made

<details>
<summary>Hint</summary>
Keep a backup of original values before applying changes. The rollback script reads the backup and applies the original values. Log each change to `/var/log/performance-rollback.log`.
</details>

## Success Criteria
- [ ] The Ansible playbook applies all performance tuning changes idempotently.
- [ ] The validation script checks all parameters and reports PASS/FAIL.
- [ ] The monitoring configuration covers all critical performance metrics.
- [ ] The rollback script can restore all defaults safely.
- [ ] The playbook can be applied to multiple servers in parallel.

## What You Should Understand After This Exercise
Performance tuning is not a one-time task -- it must be automated, validated, and repeatable. An Ansible playbook ensures consistent tuning across all servers. Validation scripts catch drift. Monitoring detects degradation. Rollback plans provide safety. This is how you operationalize performance tuning at scale.
