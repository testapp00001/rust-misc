# Cheatsheet: High Availability

## Availability Levels

| Uptime | Downtime/Year | Downtime/Month |
|--------|---------------|----------------|
| 99.9% | 8.76 hours | 43.83 minutes |
| 99.99% | 52.60 minutes | 4.38 minutes |
| 99.999% | 5.26 minutes | 26.30 seconds |

## HA Patterns

### Active-Passive
```
[Active] ←→ [Passive]
  ↓            ↓
[Shared Storage]
```

### Active-Active
```
[Active 1] ←→ [Active 2]
     ↓              ↓
  [Load Balancer]
```

## PostgreSQL HA with Patroni
```yaml
# patroni.yml
scope: postgres-cluster
name: node1

restapi:
  listen: 0.0.0.0:8008

postgresql:
  listen: 0.0.0.0:5432
  data_dir: /var/lib/postgresql/data

etcd:
  host: etcd:2379
```

## Split-Brain Prevention
```
- Use quorum (majority agreement)
- Use fencing (STONITH)
- Use witness node
- Use consensus algorithm (Raft)
```
