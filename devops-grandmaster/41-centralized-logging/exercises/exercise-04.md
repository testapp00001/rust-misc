# Exercise 04: Build Log-Based Alerting for Error Patterns

**Type:** Challenge
**Time:** 60 minutes
**Difficulty:** Medium-Hard

## Objective

Configure Grafana alert rules that fire when log-based conditions are met
(error rate thresholds, new error patterns, absent logs), route those alerts
through a contact point, and implement recording rules to pre-compute expensive
log queries for alerting performance.

## Background

Dashboards are useful when someone is looking at them. Alerting is what happens
when nobody is looking. Log-based alerting lets you detect problems from the
content and volume of log lines -- a spike in errors, a new error message that
has never appeared before, or the absence of expected log lines. This exercise
builds on the Loki and Grafana deployment from Exercises 02 and 03.

---

## Tasks

### Part A: Error Rate Alert

Create a Grafana alert rule that fires when the ERROR rate exceeds a threshold.

1. Write a LogQL metric query that calculates the error rate as a ratio of
   ERROR log lines to total log lines over a 5-minute window.
2. Create a Grafana alert rule using this query with the following conditions:
   - **Alert if:** error rate > 1% for 2 minutes
   - **Labels:** `severity=warning`, `service=order-api`
   - **Annotations:** A summary that includes the current error rate value
3. Verify the alert rule evaluates correctly by checking the Grafana alert
   rules UI.
4. Describe what happens when the error rate crosses the threshold and then
   recovers.

<details>
<summary>Hint</summary>

- In Grafana, go to Alerting > Alert rules > New alert rule.
- Use Loki as the data source.
- The query: `sum(rate({app="log-generator"} | json | level="ERROR" [5m]))
  / sum(rate({app="log-generator"} | json [5m]))`
- Set the "Condition" to `IS ABOVE 0.01` (1%).
- Set "Evaluate every" to 1m and "For" to 2m.
- In annotations, use `{{ $value }}` to include the current value.

</details>

### Part B: New Error Pattern Detection

Create an alert that fires when a never-before-seen error message appears.

1. Write a LogQL query that identifies ERROR messages that have appeared in the
   last 10 minutes but did NOT appear in the previous 24 hours.
2. Create a Grafana alert rule that fires when such new error messages are
   detected.
3. What are the limitations of this approach? What false positives might you
   expect?

<details>
<summary>Hint</summary>

- This is harder than a simple threshold. You need to compare two time ranges.
- One approach: use `count_over_time` for two different ranges and compare.
- Another approach: use Loki's `absent_over_time` or compare counts.
- A simpler approach for a lab: alert when a specific rare error keyword
  appears (e.g., "FATAL" or "OutOfMemory").
- In production, consider using Loki recording rules to pre-compute baseline
  error counts and compare against them.

</details>

### Part C: Absent Logs Alert

Create an alert that fires when a service stops producing logs entirely.

1. Write a LogQL query that detects when a service has produced zero log lines
   in the last 5 minutes.
2. Create a Grafana alert rule that fires under this condition.
3. Explain why "no logs" is a critical signal that many teams overlook. What
   types of failures does it catch that error-rate alerts miss?

<details>
<summary>Hint</summary>

- Use `absent_over_time({app="log-generator"} [5m])` -- this returns 1 if
  there are no log lines in the time range, and nothing if there are.
- Alternatively, use `count_over_time({app="log-generator"} [5m]) == 0` and
  alert when the result is present.
- "No logs" catches: container crashes, OOMKills, network partitions between
  node and logging backend, log shipper failures, and silent hangs.

</details>

### Part D: Recording Rules for Alerting

Create Loki recording rules to pre-compute expensive log queries.

1. Identify which of your alerting queries are expensive (scan large time
   windows, use complex parsing).
2. Create a Loki recording rule group that computes the following metrics
   every 1 minute:
   - `log:errors:rate5m` -- ERROR rate over 5 minutes
   - `log:volume:rate5m` -- total log volume rate over 5 minutes
3. Rewrite your alert rules from Part A to use the recording rule metrics
   instead of raw LogQL queries.
4. Explain the benefit of recording rules for alerting.

Write the recording rule configuration in YAML format as it would appear in
a Loki ruler configuration or Grafana's alert rule provisioning.

<details>
<summary>Hint</summary>

- Recording rules are defined in rule groups, similar to Prometheus recording
  rules.
- In Loki, they go under `ruler.storage` configuration or in Grafana's
  alerting UI under "Recording rules."
- The recording rule evaluates the LogQL expression and writes the result as
  a new metric that Prometheus-compatible tools can query.
- Benefit: the expensive query runs once per minute instead of every time an
  alert rule evaluates.

</details>

### Part E: Alert Routing

Design an alert routing configuration for the following scenario:

- **Critical alerts** (service down, data loss) should page the on-call
  engineer via PagerDuty.
- **Warning alerts** (error rate above threshold, slow queries) should send
  a message to a Slack channel.
- **Info alerts** (new error patterns, log volume anomalies) should create
  a Jira ticket.

Write the Grafana contact points and notification policy configuration in
YAML or describe the configuration you would use in the Grafana UI.

<details>
<summary>Hint</summary>

- In Grafana, contact points define where alerts are sent (PagerDuty, Slack,
  email, webhook, etc.).
- Notification policies route alerts to contact points based on label matchers.
- Use `severity=critical` labels on critical alerts and match them to the
  PagerDuty contact point.
- The default notification policy catches anything not matched by a more
  specific policy.

</details>

---

## Success Criteria

- [ ] You have a working Grafana alert rule that fires when the ERROR rate
      exceeds 1% for 2 minutes, with appropriate labels and annotations
- [ ] You have a new-error-pattern detection rule that identifies ERROR
      messages not seen in the previous 24 hours
- [ ] You have an absent-logs alert that fires when a service produces zero
      log lines for 5 minutes
- [ ] You have recording rules that pre-compute error rate and volume metrics
      from LogQL queries
- [ ] You can describe how to route alerts to different channels based on
      severity labels

## What You Should Understand After This Exercise

Log-based alerting extends your observability beyond metrics. While Prometheus
alerts on numeric metrics (CPU, latency, request count), log-based alerts
detect problems visible only in log content: new error types, specific failure
messages, and the complete absence of logs. The key challenge is avoiding alert
fatigue -- use recording rules to make queries efficient, set appropriate
thresholds and evaluation windows to reduce false positives, and route alerts
by severity so the right person gets woken up for the right reason.
