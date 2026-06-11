# Cheatsheet: Disaster Recovery

## Key Terms

| Term | Definition | Example |
|------|-----------|---------|
| RPO | How much data can you lose | 1 hour of data |
| RTO | How long can you be down | 30 minutes |

## DR Strategies

| Strategy | RPO | RTO | Cost |
|----------|-----|-----|------|
| Backup & Restore | Hours | Hours | Low |
| Pilot Light | Minutes | 30 min | Medium |
| Warm Standby | Seconds | Minutes | Medium-High |
| Multi-Site Active | Zero | Zero | High |

## DR Plan Checklist
- [ ] Define RPO and RTO
- [ ] Identify critical systems
- [ ] Document recovery procedures
- [ ] Automate backup verification
- [ ] Test failover regularly
- [ ] Document communication plan
- [ ] Train team on procedures

## Failover Procedure
```bash
# 1. Detect failure
# 2. Assess impact
# 3. Declare disaster
# 4. Activate DR site
# 5. Update DNS
# 6. Verify services
# 7. Communicate status
# 8. Monitor recovery
```

## DR Testing
```bash
# Quarterly DR drill
# 1. Simulate primary failure
# 2. Execute failover
# 3. Verify data integrity
# 4. Measure RTO/RPO
# 5. Document findings
# 6. Improve procedures
```
