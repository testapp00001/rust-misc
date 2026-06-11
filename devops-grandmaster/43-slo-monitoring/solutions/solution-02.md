# Solution 02: Burn Rate Alerts in Prometheus

## Part A -- Recording rules for SLO components

```yaml
groups:
  - name: slo_recording_rules
    interval: 30s
    rules:
      # Total request rate (5-minute window)
      - record: job:http_requests:rate5m
        expr: |
          sum(rate(http_requests_total[5m])) by (job)

      # Error request rate (5-minute window)
      - record: job:http_errors:rate5m
        expr: |
          sum(rate(http_requests_total{status=~"5.."}[5m])) by (job)

      # Error ratio (5-minute window)
      - record: job:http_errors:ratio5m
        expr: |
          sum(rate(http_requests_total{status=~"5.."}[5m])) by (job)
          /
          sum(rate(http_requests_total[5m])) by (job)
          or vector(0)

      # Error budget remaining as percentage
      # For 99.9% SLO: allowed error rate = 0.001
      - record: job:http_errors:budget_remaining_pct
        expr: |
          clamp_min(
            (
              1 - (
                avg_over_time(job:http_errors:ratio5m[30d]) / (1 - 0.999)
              )
            ) * 100,
            0
          )
```

**Why this works**:

- `rate()` over a 5-minute window smooths out per-second noise while
  remaining responsive to changes.
- `by (job)` allows the rules to work for multiple services simultaneously.
- `or vector(0)` prevents division-by-zero when there are no requests.
- `clamp_min(..., 0)` ensures the budget percentage does not go negative
  (it can go negative if the SLO is already violated).
- Recording rules are evaluated at the `interval` (30s), so they are
  pre-computed and fast to query in alerts and dashboards.

**Common mistakes**:
- Forgetting `or vector(0)` leading to `NaN` results during zero-traffic
  periods
- Not using `by (job)` which causes all services to be aggregated together
- Using `irate()` instead of `rate()` -- `irate` is too sensitive for SLO
  monitoring because it only looks at the last two data points

---

## Part B -- Burn rate alert (14.4x over 1 hour)

```yaml
groups:
  - name: slo_alerts
    rules:
      - alert: SLOBurnRateFast
        expr: |
          job:http_errors:ratio1h > (14.4 * (1 - 0.999))
        for: 2m
        labels:
          severity: critical
          slo_target: "99.9"
          burn_rate: "14.4"
          window: "1h"
        annotations:
          summary: "Fast burn detected on {{ $labels.job }}"
          description: |
            Error ratio {{ $value | humanizePercentage }} over 1h exceeds
            the 14.4x burn rate threshold for a 99.9% SLO.
            At this rate, the 30-day error budget will be exhausted in ~20 hours.
          runbook_url: "https://runbooks.example.com/slo-fast-burn"
```

**Additional recording rule needed for 1-hour window**:

```yaml
      - record: job:http_errors:ratio1h
        expr: |
          sum(rate(http_requests_total{status=~"5.."}[1h])) by (job)
          /
          sum(rate(http_requests_total[1h])) by (job)
          or vector(0)
```

**Why this works**:

- The threshold `14.4 * 0.001 = 0.0144` (1.44% error rate)
- At 14.4x burn rate, you consume 14.4 * (1/30) = 48% of the daily budget
  per hour, meaning you will exhaust the full budget in ~20.8 hours
- `for: 2m` reduces false positives from brief spikes while still detecting
  the incident quickly

**Common mistakes**:
- Computing burn rate as a ratio of error counts instead of error ratios
- Using the raw error count instead of the error rate
- Setting `for:` too high (e.g., 10m) which delays alerting during real outages

---

## Part C -- Multi-burn-rate alert

```yaml
      - alert: SLOBurnRateFastConfirmed
        expr: |
          (
            job:http_errors:ratio1h > (14.4 * (1 - 0.999))
          and
            job:http_errors:ratio5m > (14.4 * (1 - 0.999))
          )
        for: 2m
        labels:
          severity: critical
          slo_target: "99.9"
        annotations:
          summary: "Confirmed fast burn on {{ $labels.job }}"
          description: |
            Both 1h and 5m error ratios exceed the 14.4x burn rate.
            This is a sustained fast burn, not a brief spike.
            Budget exhaustion projected in ~20 hours.
```

**Why this works**:

- The `and` operator requires both conditions to be true for the same
  time series (matched by labels).
- The 5m window catches the current state; the 1h window confirms it is
  sustained.
- If a brief spike causes the 5m ratio to exceed the threshold but the 1h
  ratio does not, the alert will not fire. This eliminates false positives
  from short blips.

**Common mistakes**:
- Using `or` instead of `and` (which makes the alert MORE sensitive, not less)
- Mismatching label selectors between the two sides of `and`
- Forgetting that `and` performs an inner join on label sets

---

## Part D -- Error budget consumption alert

```yaml
      - alert: SLOBudgetExhausted
        expr: |
          (
            1 - (
              avg_over_time(job:http_errors:ratio5m[30d]) / (1 - 0.999)
            )
          ) < 0.5
        for: 1h
        labels:
          severity: warning
          slo_target: "99.9"
        annotations:
          summary: "Error budget 50% consumed for {{ $labels.job }}"
          description: |
            More than 50% of the 30-day error budget has been consumed.
            Remaining budget: approximately {{ with $value }}{{ (1 - $value) * 43.2 | printf "%.1f" }}{{ end }} minutes.
```

**Why this works**:

- `avg_over_time(...[30d])` computes the average error ratio over the full
  30-day window.
- Dividing by `(1 - 0.999)` normalizes it as a fraction of the total budget.
- `1 - ...` gives the remaining fraction; `< 0.5` means more than 50% consumed.
- `for: 1h` prevents the alert from firing during brief periods where the
  30d average fluctuates.

**Common mistakes**:
- Not using `avg_over_time` (using `rate` alone does not cover the full window)
- Computing budget consumed as a simple percentage of time rather than
  weighted by error ratio
- Using `for: 5m` which is too short for a 30-day signal (causes flapping)
