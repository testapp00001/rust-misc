# Cheatsheet: Incident Response

## Severity Levels

| Level | Name | Response Time | Examples |
|-------|------|---------------|----------|
| P1 | Critical | 15 min | Service down, data loss |
| P2 | High | 30 min | Major feature broken |
| P3 | Medium | 2 hours | Minor feature broken |
| P4 | Low | 24 hours | Cosmetic issue |

## Incident Response Steps
```
1. Detect → Alert fires or user reports
2. Triage → Assess severity, assign roles
3. Communicate → Status page, stakeholders
4. Investigate → Check monitoring, logs
5. Mitigate → Rollback, scale, restart
6. Resolve → Permanent fix
7. Post-mortem → Learn and improve
```

## On-Call Rotation
```
Week 1: Engineer A (primary), Engineer B (backup)
Week 2: Engineer B (primary), Engineer C (backup)
Week 3: Engineer C (primary), Engineer A (backup)
```

## Escalation Matrix
```
0 min  → On-call engineer
15 min → Backup on-call
30 min → Engineering manager
60 min → VP Engineering
2 hours → CTO
```

## Communication Templates
```
🔴 INCIDENT: [Description]
Severity: P[X]
Impact: [What's affected]
Status: Investigating
IC: [Name]
```
