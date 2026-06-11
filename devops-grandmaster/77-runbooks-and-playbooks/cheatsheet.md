# Cheatsheet: Runbooks & Playbooks

## Runbook Template
```markdown
# Runbook: [Scenario]

## Trigger
When [condition] occurs.

## Impact
[What is affected]

## Steps
1. [Step 1]
2. [Step 2]
3. [Step 3]

## Verification
- [ ] [Check 1]
- [ ] [Check 2]

## Escalation
If unable to resolve in [X] minutes, escalate to [team].
```

## Common Runbooks

### Server Down
```bash
# 1. Check connectivity
ping -c 3 server-ip

# 2. Check SSH
ssh user@server-ip

# 3. Check resources
top, df -h, free -m

# 4. Check services
systemctl status my-app
docker ps

# 5. Check logs
journalctl -u my-app --since "10 minutes ago"

# 6. Restart if needed
systemctl restart my-app
```

### High CPU
```bash
# 1. Find process
top -b -n 1 | head -20

# 2. Check containers
docker stats --no-stream

# 3. Check logs
docker logs --tail 1000 my-app | grep ERROR

# 4. Scale if needed
docker compose up -d --scale api=3
```

### Disk Full
```bash
# 1. Check usage
df -h

# 2. Find large files
du -sh /* | sort -rh | head -10

# 3. Clean Docker
docker system prune -a --volumes

# 4. Clean logs
find /var/log -name "*.log" -mtime +7 -delete
```
