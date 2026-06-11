# Cheatsheet: Alerting Systems

## AlertManager Configuration
```yaml
route:
  receiver: 'slack-notifications'
  routes:
    - match:
        severity: critical
      receiver: 'pagerduty-critical'

receivers:
  - name: 'slack-notifications'
    slack_configs:
      - channel: '#alerts'

  - name: 'pagerduty-critical'
    pagerduty_configs:
      - service_key: 'your-key'
```

## Alert Rules (Prometheus)
```yaml
groups:
  - name: alerts
    rules:
      - alert: HighCPU
        expr: rate(container_cpu_usage_seconds_total[5m]) > 0.8
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High CPU on {{ $labels.instance }}"
```

## Alert Severity Levels

| Level | Response | Examples |
|-------|----------|----------|
| Critical | Page immediately | Service down, data loss |
| Warning | Notify on-call | High error rate, high latency |
| Info | Log only | Deployment completed |

## Best Practices
- Avoid alert fatigue
- Include runbook links
- Use inhibition rules
- Set appropriate thresholds
- Test alerts regularly
