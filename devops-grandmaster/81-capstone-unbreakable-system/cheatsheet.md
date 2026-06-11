# Cheatsheet: Capstone — The Unbreakable System

## Architecture Overview
```
                    ┌─────────────┐
                    │   CDN/WAF   │
                    └──────┬──────┘
                           │
                    ┌──────▼──────┐
                    │ Load Balancer│
                    └──────┬──────┘
                           │
              ┌────────────┼────────────┐
              │            │            │
        ┌─────▼─────┐ ┌───▼────┐ ┌────▼─────┐
        │  Frontend │ │  API   │ │  API     │
        │  (nginx)  │ │  (v1)  │ │  (v2)    │
        └───────────┘ └───┬────┘ └────┬─────┘
                          │            │
                    ┌─────▼────────────▼─────┐
                    │      Message Queue      │
                    └────────────┬────────────┘
                                 │
                    ┌────────────▼────────────┐
                    │       Workers           │
                    └────────────┬────────────┘
                                 │
              ┌──────────────────┼──────────────────┐
              │                  │                  │
        ┌─────▼─────┐    ┌──────▼──────┐    ┌─────▼─────┐
        │ PostgreSQL│    │    Redis    │    │   S3/GCS  │
        │ (Primary) │    │   (Cache)   │    │  (Storage)│
        │ (Replica) │    │             │    │           │
        └───────────┘    └─────────────┘    └───────────┘
```

## Checklist
- [ ] Containerized microservices
- [ ] Health checks on all services
- [ ] Structured logging to stdout
- [ ] Prometheus + Grafana monitoring
- [ ] Centralized log aggregation
- [ ] Alerting with PagerDuty
- [ ] CI/CD pipeline with tests
- [ ] Blue-green or canary deployment
- [ ] Auto-scaling (HPA)
- [ ] Load balancing
- [ ] Database replication
- [ ] Automated backups
- [ ] Disaster recovery plan
- [ ] Rate limiting
- [ ] WAF rules
- [ ] Secrets management
- [ ] Network policies
- [ ] Incident response runbooks
- [ ] Post-mortem process
- [ ] Cost optimization
