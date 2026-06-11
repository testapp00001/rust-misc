# Solution 04: Build Log-Based Alerting for Error Patterns

## Part A: Error Rate Alert

### LogQL Query

```logql
sum(rate({app="log-generator"} | json | level="ERROR" [5m]))
/
sum(rate({app="log-generator"} | json [5m]))
```

This returns a value between 0 and 1 representing the fraction of log lines
that are ERROR-level. A value of 0.01 means 1% error rate.

### Grafana Alert Rule Configuration

In Grafana, navigate to Alerting > New alert rule:

- **Name:** High Error Rate - log-generator
- **Data source:** Loki
- **Query:** (the LogQL query above)
- **Condition:** `IS ABOVE 0.01` (1%)
- **Evaluate every:** 1 minute
- **For:** 2 minutes (the condition must be true for 2 consecutive evaluations
  before the alert fires)
- **Labels:**
  - `severity: warning`
  - `service: log-generator`
  - `team: platform`
- **Annotations:**
  - `summary: Error rate on {{ $labels.service }} is {{ $value | humanizePercentage }}`
  - `description: The error rate for {{ $labels.service }} has exceeded 1% for more than 2 minutes. Current value: {{ $value | humanizePercentage }}.`

### What happens on threshold crossing and recovery

When the error rate crosses 1% and stays above for 2 minutes, the alert
transitions from `Normal` to `Pending` (after the first evaluation above
threshold) to `Alerting` (after 2 consecutive evaluations). Grafana sends
a notification to the configured contact point.

When the error rate drops below 1%, the alert transitions from `Alerting` to
`Normal` after one evaluation below threshold (the "For" period does not apply
to recovery). Grafana sends a recovery notification.

**Why this works:** The `For` duration prevents flapping. Without it, a
single-minute spike would trigger and resolve the alert repeatedly. The 2-minute
`For` window ensures the condition is sustained before paging someone.

## Part B: New Error Pattern Detection

### Approach 1: Compare time ranges (advanced)

This is conceptually difficult in pure LogQL because Loki does not natively
support comparing two arbitrary time ranges in a single query. The workaround
uses recording rules:

```yaml
# Recording rule: count unique error messages in the last 24h
- record: log:error_messages:count24h
  expr: |
    count(
      sum by (message) (
        count_over_time({app="log-generator"} | json | level="ERROR" [24h])
      )
    )
```

Then a separate query for the current hour, and an alert when new messages
appear. In practice, this is complex and has limitations.

### Approach 2: Alert on specific rare keywords (practical)

A more practical approach for a lab:

```logql
sum(count_over_time({app="log-generator"} | json | level="ERROR"
  |~ "FATAL|OutOfMemory|SIGKILL|Segmentation fault|panic" [10m]))
```

Create an alert that fires when this count is greater than 0:

- **Name:** Critical Error Pattern Detected
- **Condition:** IS ABOVE 0
- **Evaluate every:** 1 minute
- **For:** 0 minutes (immediate)
- **Labels:** `severity: critical`

### Approach 3: Alert on error count spike

Compare the current hour's error count to the previous 24h average:

```logql
# Current 1h error count
sum(count_over_time({app="log-generator"} | json | level="ERROR" [1h]))
```

Set the alert threshold at 3x the expected baseline. If you expect ~100 errors
per hour, alert at 300:

- **Condition:** IS ABOVE 300
- **For:** 5 minutes

### Limitations

- **False positives from deployments:** A new code deployment might introduce
  a new error message that is actually expected (e.g., a deprecation warning).
  The alert fires, but it is not a real problem.
- **False negatives:** If a new error message matches an existing pattern
  (e.g., "payment failed" vs "payment_failure"), it may not trigger the
  keyword-based alert.
- **Baseline drift:** The 3x threshold assumes a stable baseline. During
  traffic spikes, error counts naturally increase, causing false positives.
- **LogQL limitations:** Comparing two time ranges in a single query is not
  straightforward. Recording rules or external tools (e.g., a script that
  queries Loki's API) may be needed for robust new-pattern detection.

## Part C: Absent Logs Alert

### LogQL Query

```logql
absent_over_time({app="log-generator"} [5m])
```

`absent_over_time` returns `1` if there are zero log lines matching the
selector in the given time range. If there are any log lines, it returns
nothing (empty result). This is the Loki equivalent of Prometheus's
`absent()` function.

### Alert Rule Configuration

- **Name:** Service Down - No Logs from log-generator
- **Data source:** Loki
- **Query:** `absent_over_time({app="log-generator"} [5m])`
- **Condition:** `IS ABOVE 0` (the query returns 1 when absent, nothing
  when present)
- **Evaluate every:** 1 minute
- **For:** 0 minutes (immediate -- absence is already a sustained condition
  since it requires a 5-minute window of nothing)
- **Labels:**
  - `severity: critical`
  - `service: log-generator`
- **Annotations:**
  - `summary: No logs received from {{ $labels.service }} for 5 minutes`
  - `description: The logging pipeline has received zero log lines from {{ $labels.service }} in the last 5 minutes. This may indicate a container crash, OOMKill, network partition, or log shipper failure.`

### Why "no logs" is a critical signal

"No logs" catches failures that error-rate alerts miss:

- **Container crash / OOMKill:** The container is dead. It produces no logs
  at all -- not even errors. Error-rate alerts only fire when errors are
  being produced.
- **Network partition:** The node cannot reach Loki. Logs are buffering
  locally but not being shipped. The application may be running fine, but
  you have lost visibility.
- **Log shipper failure:** Promtail or Fluentd crashed or is
  misconfigured. The application is logging, but the logs are not reaching
  Loki.
- **Silent hang:** The application is alive (passing health checks) but
  deadlocked. It produces no new log lines because no requests are being
  processed.
- **Deployment with broken logging:** A new deployment accidentally disabled
  structured logging or redirected output to a file. The application works,
  but you have no visibility.

Error-rate alerts only detect errors you can see. Absent-logs alerts detect
the absence of the signal itself, which is a meta-failure that affects your
ability to detect all other failures.

## Part D: Recording Rules

### Expensive queries to pre-compute

The error rate query scans all log lines, parses JSON, filters by level, and
computes rates over 5-minute windows. If evaluated every 30 seconds by multiple
alert rules, this is expensive. Pre-computing it once per minute reduces load.

### Recording rule configuration

```yaml
groups:
  - name: log_metrics
    interval: 1m
    rules:
      - record: log:errors:rate5m
        expr: |
          sum(rate({app="log-generator"} | json | level="ERROR" [5m]))

      - record: log:volume:rate5m
        expr: |
          sum(rate({app="log-generator"} | json [5m]))

      - record: log:error_rate:ratio5m
        expr: |
          log:errors:rate5m / log:volume:rate5m
```

### Rewritten alert rule

Instead of the expensive inline query, the alert rule now references the
pre-computed metric:

- **Query:** `log:error_rate:ratio5m`
- **Condition:** IS ABOVE 0.01
- **For:** 2 minutes

### Benefits of recording rules

- **Reduced Loki load:** The expensive LogQL query runs once per minute by
  the ruler, not every 30 seconds by each alert rule.
- **Faster alert evaluation:** Alert rules query a pre-computed instant vector
  instead of scanning log chunks.
- **Reusable metrics:** Multiple alert rules can reference the same recording
  rule without duplicating the expensive query.
- **Queryable in Grafana:** Recording rule metrics appear as regular metrics
  in Grafana, so you can graph `log:error_rate:ratio5m` alongside Prometheus
  metrics.

## Part E: Alert Routing

### Contact Points Configuration

```yaml
# Grafana contact points (provisioned or configured in UI)
contactPoints:
  - name: pagerduty-critical
    type: pagerduty
    settings:
      integrationKey: "<pagerduty-integration-key>"
      severity: critical

  - name: slack-warnings
    type: slack
    settings:
      recipient: "#platform-alerts"
      token: "<slack-bot-token>"
      title: '{{ template "default.title" . }}'
      text: '{{ template "default.message" . }}'

  - name: jira-info
    type: webhook
    settings:
      url: "https://your-org.atlassian.net/rest/api/3/issue"
      method: POST
      httpHeaders:
        Authorization: "Basic <base64-encoded-credentials>"
        Content-Type: "application/json"
```

### Notification Policy Configuration

```yaml
notificationPolicies:
  # Critical alerts go to PagerDuty
  - matchers:
      - severity = critical
    receiver: pagerduty-critical
    group_wait: 30s
    group_interval: 5m
    repeat_interval: 4h

  # Warning alerts go to Slack
  - matchers:
      - severity = warning
    receiver: slack-warnings
    group_wait: 1m
    group_interval: 5m
    repeat_interval: 12h

  # Info alerts go to Jira
  - matchers:
      - severity = info
    receiver: jira-info
    group_wait: 5m
    group_interval: 30m
    repeat_interval: 24h

  # Default catch-all (fallback)
  - receiver: slack-warnings
```

### How this works

1. An alert fires with labels `severity=warning`, `service=log-generator`.
2. Grafana's notification policy engine matches the `severity = warning`
   policy.
3. The alert is sent to the `slack-warnings` contact point.
4. `group_wait: 1m` batches multiple alerts firing within 1 minute into a
   single Slack message.
5. `repeat_interval: 12h` re-sends the alert every 12 hours if it is still
   firing.

Critical alerts (`severity=critical`) route to PagerDuty, which pages the
on-call engineer. Info alerts (`severity=info`) create Jira tickets for
triage during business hours.

## Common Mistakes

- **Alerting on raw log counts instead of rates.** Raw counts depend on
  traffic volume. At 2 AM when traffic is low, you get zero errors and no
  alerts. At peak traffic, you get hundreds of errors that are normal for
  that volume. Use rates (errors/total) to normalize for traffic.
- **Setting `For` to 0 for rate-based alerts.** A single evaluation spike
  (caused by a burst of requests or a brief deployment blip) triggers the
  alert immediately. A 2-5 minute `For` window filters out transient spikes
  and only alerts on sustained problems.
- **Using too many recording rules.** Each recording rule consumes Loki ruler
  resources. Only pre-compute metrics that are referenced by multiple alert
  rules or that cause performance issues when queried directly.
- **Not testing alert routing.** Configure a test contact point (e.g., a
  personal email) and verify that alerts actually arrive before depending on
  PagerDuty routing for critical production alerts.
- **Forgetting to set `group_wait` and `repeat_interval`.** Without these,
  every alert fires a separate notification immediately and repeats
  indefinitely. This causes alert fatigue within hours.
