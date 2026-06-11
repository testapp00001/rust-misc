# Cheatsheet: Change Management

## Change Types

| Type | Risk | Approval | Examples |
|------|------|----------|----------|
| Standard | Low | Pre-approved | Routine updates |
| Normal | Medium | CAB approval | Feature releases |
| Emergency | High | Post-approval | Security patches |

## Change Process
```
1. Request → Submit change request
2. Assess → Risk assessment
3. Approve → CAB or manager approval
4. Schedule → Maintenance window
5. Execute → Perform change
6. Verify → Confirm success
7. Close → Document outcome
```

## Rolling Updates
```yaml
# Kubernetes rolling update
spec:
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0
```

## Maintenance Windows
```
Low traffic: 2-6 AM local time
Notify users: 48 hours in advance
Have rollback plan ready
Monitor during and after
```

## Change Freeze
```
No changes during:
- Major holidays
- Critical business periods
- After incidents (cooling period)
```
