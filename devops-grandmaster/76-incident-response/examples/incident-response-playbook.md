# Incident Response Playbook

## Severity Levels

| Level | Name | Description | Response Time | Examples |
|-------|------|-------------|---------------|----------|
| P1 | Critical | Service completely down | 15 min | Site unreachable, data loss |
| P2 | High | Major feature broken | 30 min | Payment failing, auth broken |
| P3 | Medium | Minor feature broken | 2 hours | Slow search, UI glitch |
| P4 | Low | Cosmetic issue | 24 hours | Typo, color wrong |

## Incident Response Procedure

### 1. Detect (0-5 minutes)
```
□ Alert fires or user reports issue
□ Acknowledge the alert in PagerDuty
□ Open incident channel in Slack (#incident-YYYY-MM-DD-HHMM)
□ Notify incident commander
```

### 2. Triage (5-15 minutes)
```
□ Assess severity (P1-P4)
□ Identify affected services/users
□ Estimate impact scope
□ Assign roles:
  - Incident Commander (IC) — coordinates response
  - Technical Lead — investigates and fixes
  - Communications — updates stakeholders
```

### 3. Communicate (15-30 minutes)
```
□ Update status page
□ Notify affected customers (if P1/P2)
□ Send internal update to stakeholders
□ Post in incident channel every 15 minutes
```

### 4. Investigate (30-60 minutes)
```
□ Check monitoring dashboards
□ Review recent deployments
□ Check infrastructure (CPU, memory, disk, network)
□ Review logs for errors
□ Identify root cause
```

### 5. Mitigate (as fast as possible)
```
□ Apply immediate fix:
  - Rollback recent deployment
  - Scale up resources
  - Restart affected services
  - Enable maintenance mode
  - Failover to backup
□ Verify mitigation worked
□ Monitor for recurrence
```

### 6. Resolve (after mitigation)
```
□ Apply permanent fix
□ Verify fix in production
□ Monitor for 30 minutes
□ Close incident
```

### 7. Post-Incident (within 48 hours)
```
□ Schedule post-mortem meeting
□ Write post-mortem document
□ Identify action items
□ Assign action items with due dates
□ Share learnings with team
```

## Common Scenarios

### Server Down
```bash
# 1. Check if server is reachable
ping -c 3 server-ip

# 2. Check SSH access
ssh user@server-ip

# 3. Check system resources
top
df -h
free -m

# 4. Check running services
systemctl status my-app
docker ps

# 5. Check logs
journalctl -u my-app --since "10 minutes ago"
docker logs --tail 100 my-app

# 6. Restart if needed
systemctl restart my-app
# or
docker restart my-app
```

### High CPU Usage
```bash
# 1. Find the process
top -b -n 1 | head -20

# 2. Check container resource usage
docker stats --no-stream

# 3. Check for runaway processes
ps aux --sort=-%cpu | head -10

# 4. Check application logs
docker logs --tail 1000 my-app | grep ERROR

# 5. Scale if needed
docker compose up -d --scale api=3
```

### Disk Full
```bash
# 1. Check disk usage
df -h

# 2. Find large files
du -sh /* | sort -rh | head -10

# 3. Clean up Docker
docker system prune -a --volumes

# 4. Clean up logs
find /var/log -name "*.log" -mtime +7 -delete

# 5. Clean up old backups
find /backups -mtime +30 -delete
```

### Database Connection Issues
```bash
# 1. Check if database is running
docker ps | grep postgres

# 2. Check connections
docker exec postgres psql -U postgres -c "SELECT count(*) FROM pg_stat_activity;"

# 3. Check for locks
docker exec postgres psql -U postgres -c "SELECT * FROM pg_locks WHERE NOT granted;"

# 4. Kill long-running queries
docker exec postgres psql -U postgres -c "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE state = 'active' AND query_start < now() - interval '5 minutes';"

# 5. Restart if needed
docker restart postgres
```

## Escalation Matrix

| Time Elapsed | Action |
|-------------|--------|
| 0 min | On-call engineer responds |
| 15 min | Page backup on-call |
| 30 min | Page engineering manager |
| 60 min | Page VP Engineering |
| 2 hours | Page CTO |

## Communication Templates

### Initial Notification
```
🔴 INCIDENT: [Brief Description]
Severity: P[X]
Impact: [What's affected]
Status: Investigating
IC: [Name]
Updates: Every 15 minutes in #incident-YYYY-MM-DD
```

### Status Update
```
🟡 UPDATE: [Brief Description]
Time: [Current time]
Status: [Investigating/Mitigating/Resolved]
Findings: [What we know]
Next Steps: [What we're doing next]
ETA: [When we expect resolution]
```

### Resolution
```
🟢 RESOLVED: [Brief Description]
Duration: [How long it lasted]
Root Cause: [What caused it]
Fix: [What we did]
Follow-up: [Post-mortem scheduled for...]
```
