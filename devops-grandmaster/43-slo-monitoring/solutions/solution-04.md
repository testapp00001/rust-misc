# Solution 04: Multi-Window Burn Rate Alerts

## Part A -- The math

**Error budget for 99.9% SLO over 30 days**:

```
43,200 minutes * 0.001 = 43.2 minutes
```

**Budget consumed per scenario**:

| Scenario | Burn Rate | Duration | Budget Consumed | Calculation |
|----------|-----------|----------|----------------|-------------|
| Fast burn | 14.4x | 1 hour | 2% | 14.4 * (60/43200) = 2% |
| Medium burn | 6x | 6 hours | 5% | 6 * (360/43200) = 5% |
| Slow burn | 1x | 3 days | 6.94% | 1 * (4320/43200) = 10% |

**More precise calculations**:

```
Fast burn:  14.4 * (1h / 30d) = 14.4 * (1/720) = 0.02 = 2%
Medium burn: 6 * (6h / 30d) = 6 * (6/720) = 6 * 0.00833 = 0.05 = 5%
Slow burn:  1 * (3d / 30d) = 1 * 0.1 = 10%
```

**Allowed error ratio for 1h fast-burn**:

```
Burn rate * (1 - SLO) = 14.4 * 0.001 = 0.0144 (1.44%)
```

**What this means at 1000 req/s**:

```
1000 req/s * 0.0144 = 14.4 errors/second sustained over 1 hour
```

That is approximately **51,840 errors** in one hour -- a significant outage.

---

## Part B -- Multi-window alert implementation

```yaml
groups:
  - name: slo_multi_window_alerts
    interval: 30s
    rules:
      # ── Recording rules for error ratios at each window size ──

      - record: job:http_errors:ratio5m
        expr: |
          sum(rate(http_requests_total{status=~"5.."}[5m])) by (job)
          /
          sum(rate(http_requests_total[5m])) by (job)
          or vector(0)

      - record: job:http_errors:ratio30m
        expr: |
          sum(rate(http_requests_total{status=~"5.."}[30m])) by (job)
          /
          sum(rate(http_requests_total[30m])) by (job)
          or vector(0)

      - record: job:http_errors:ratio1h
        expr: |
          sum(rate(http_requests_total{status=~"5.."}[1h])) by (job)
          /
          sum(rate(http_requests_total[1h])) by (job)
          or vector(0)

      - record: job:http_errors:ratio6h
        expr: |
          sum(rate(http_requests_total{status=~"5.."}[6h])) by (job)
          /
          sum(rate(http_requests_total[6h])) by (job)
          or vector(0)

      - record: job:http_errors:ratio3d
        expr: |
          sum(rate(http_requests_total{status=~"5.."}[3d])) by (job)
          /
          sum(rate(http_requests_total[3d])) by (job)
          or vector(0)

      - record: job:http_errors:ratio30d
        expr: |
          sum(rate(http_requests_total{status=~"5.."}[30d])) by (job)
          /
          sum(rate(http_requests_total[30d])) by (job)
          or vector(0)

      # ── Alerting rules ──

      # Tier 1: Fast burn -- major outage
      # Burns 2% of budget per hour. Will exhaust budget in ~20 hours.
      - alert: SLOBurnRateFast
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
          burn_rate: "14.4"
          window: "1h/5m"
        annotations:
          summary: "Fast burn on {{ $labels.job }}: budget exhaustion in ~20h"
          description: |
            1h error ratio: {{ with query "job:http_errors:ratio1h{job='{{ $labels.job }}'}" }}{{ . | first | value | humanizePercentage }}{{ end }}
            5m error ratio: {{ with query "job:http_errors:ratio5m{job='{{ $labels.job }}'}" }}{{ . | first | value | humanizePercentage }}{{ end }}
            Threshold: {{ $labels.burn_rate }}x burn rate (1.44% error ratio)
          runbook_url: "https://runbooks.example.com/slo-fast-burn"

      # Tier 2: Medium burn -- partial outage
      # Burns 5% of budget per 6 hours. Will exhaust budget in ~5 days.
      - alert: SLOBurnRateMedium
        expr: |
          (
            job:http_errors:ratio6h > (6 * (1 - 0.999))
          and
            job:http_errors:ratio30m > (6 * (1 - 0.999))
          )
        for: 5m
        labels:
          severity: warning
          slo_target: "99.9"
          burn_rate: "6"
          window: "6h/30m"
        annotations:
          summary: "Medium burn on {{ $labels.job }}: budget exhaustion in ~5d"
          description: |
            6h error ratio: {{ with query "job:http_errors:ratio6h{job='{{ $labels.job }}'}" }}{{ . | first | value | humanizePercentage }}{{ end }}
            30m error ratio: {{ with query "job:http_errors:ratio30m{job='{{ $labels.job }}'}" }}{{ . | first | value | humanizePercentage }}{{ end }}
            Threshold: {{ $labels.burn_rate }}x burn rate (0.6% error ratio)
          runbook_url: "https://runbooks.example.com/slo-medium-burn"

      # Tier 3: Slow burn -- gradual degradation
      # Burns 10% of budget per 3 days. Will exhaust budget in ~27 days.
      - alert: SLOBurnRateSlow
        expr: |
          (
            job:http_errors:ratio3d > (1 * (1 - 0.999))
          and
            job:http_errors:ratio6h > (1 * (1 - 0.999))
          )
        for: 30m
        labels:
          severity: ticket
          slo_target: "99.9"
          burn_rate: "1"
          window: "3d/6h"
        annotations:
          summary: "Slow burn on {{ $labels.job }}: gradual degradation detected"
          description: |
            3d error ratio: {{ with query "job:http_errors:ratio3d{job='{{ $labels.job }}'}" }}{{ . | first | value | humanizePercentage }}{{ end }}
            6h error ratio: {{ with query "job:http_errors:ratio6h{job='{{ $labels.job }}'}" }}{{ . | first | value | humanizePercentage }}{{ end }}
            Threshold: {{ $labels.burn_rate }}x burn rate (0.1% error ratio)
            This will not page, but should be investigated within 1 business day.
          runbook_url: "https://runbooks.example.com/slo-slow-burn"

      # Tier 4: Budget exhausted -- SLO already violated
      - alert: SLOBudgetExhausted
        expr: |
          job:http_errors:ratio30d > (1 * (1 - 0.999))
        for: 1h
        labels:
          severity: critical
          slo_target: "99.9"
        annotations:
          summary: "SLO VIOLATED for {{ $labels.job }}: error budget exhausted"
          description: |
            The 30-day error ratio exceeds the SLO target.
            30d error ratio: {{ with query "job:http_errors:ratio30d{job='{{ $labels.job }}'}" }}{{ . | first | value | humanizePercentage }}{{ end }}
            Allowed: {{ $labels.slo_target }}%
            The error budget policy should be activated.
```

---

## Part C -- Short window vs. long window explanation

### Why both windows are needed

Each alert uses a **long window** for stability and a **short window** for
precision. This is the key insight from the Google SRE book.

**What the long window provides (stability)**:

The long window confirms that the burn rate is sustained, not a brief spike.
Without it, a 30-second spike that exceeds the threshold would fire the
alert, even though the service recovered immediately.

**What the short window provides (precision)**:

The short window ensures the alert fires while the problem is happening,
not 6 hours later when the long window finally catches up. Without it, a
fast outage that starts and ends within the long window might not trigger
the alert at all.

### Scenario analysis

**Scenario 1: 2-minute spike to 5% error rate, then recovery**

| Alert | Long window | Short window | Fires? |
|-------|-------------|--------------|--------|
| Without short window | 1h avg: ~0.08% (diluted) | N/A | No |
| Without long window | N/A | 5m avg: 5% | Yes (false positive) |
| With both windows | 1h avg: ~0.08% | 5m avg: 5% | No (correct) |

**Scenario 2: Sustained 2% error rate for 3 hours**

| Alert | Long window | Short window | Fires? |
|-------|-------------|--------------|--------|
| Without short window | 1h avg: 2% (exceeds 1.44%) | N/A | Yes, but delayed |
| Without long window | N/A | 5m avg: 2% | Yes (correct) |
| With both windows | 1h avg: 2% | 5m avg: 2% | Yes (correct) |

**Scenario 3: Gradual increase from 0.05% to 0.15% over 6 hours**

| Alert | Long window | Short window | Fires? |
|-------|-------------|--------------|--------|
| Without short window | 6h avg: ~0.1% (equals threshold) | N/A | Maybe (borderline) |
| Without long window | N/A | 30m avg: 0.15% | Yes (correct, but noisy) |
| With both windows | 6h avg: ~0.1% | 30m avg: 0.15% | No (30m below 0.6%) |

**Key insight**: The short window acts as a "confirming" signal. The long
window detects the trend; the short window confirms it is still happening.

---

## Part D -- Tuning sensitivity

### Strategy 1: Increase the short window threshold

**Change**: Instead of requiring the short window to exceed the same burn
rate, require it to exceed a higher burn rate.

```yaml
# Before: both windows use the same threshold
job:http_errors:ratio1h > (14.4 * 0.001)
and
job:http_errors:ratio5m > (14.4 * 0.001)

# After: short window requires a higher threshold
job:http_errors:ratio1h > (14.4 * 0.001)
and
job:http_errors:ratio5m > (20 * 0.001)
```

**Tradeoff**:
- **Gain**: Reduces false positives from brief spikes that just cross the
  threshold. The higher short-window threshold acts as a buffer.
- **Lose**: Slightly delays detection. If the error rate is between 14.4x
  and 20x, the alert will not fire until the long window catches up.

**When to use**: When your traffic has regular brief spikes (e.g., cache
misses, deployment rollouts) that cross the threshold but are not incidents.

### Strategy 2: Add a minimum request rate filter

**Change**: Only evaluate the burn rate when there are enough requests to
make the error ratio statistically meaningful.

```yaml
# Before
job:http_errors:ratio5m > (14.4 * 0.001)

# After: require minimum traffic
(
  job:http_errors:ratio5m > (14.4 * 0.001)
  and
  job:http_requests:rate5m > 10
)
```

**Tradeoff**:
- **Gain**: Eliminates false positives during low-traffic periods (e.g.,
  3am) when a single error can produce a high error ratio.
- **Lose**: You will not detect incidents during low-traffic periods. If
  your service has 5 requests per minute and all fail, the alert will not
  fire.

**When to use**: For services with highly variable traffic patterns where
low-traffic periods produce noisy ratios.

### Bonus Strategy 3: Use a longer `for` duration

**Change**: Increase the `for` clause from 2m to 5m.

**Tradeoff**:
- **Gain**: Requires the condition to persist for 5 minutes, filtering out
  very brief spikes.
- **Lose**: Adds 3 minutes to detection time.

**When to use**: When false positives are frequent but incidents typically
last more than 5 minutes.

---

## Part E -- Recording rules for efficiency

```yaml
groups:
  - name: slo_recording_rules
    interval: 30s
    rules:
      # ── Error ratios at each required window size ──

      # Short windows (for precision)
      - record: job:http_errors:ratio5m
        expr: |
          sum(rate(http_requests_total{status=~"5.."}[5m])) by (job)
          /
          sum(rate(http_requests_total[5m])) by (job)
          or vector(0)

      - record: job:http_errors:ratio30m
        expr: |
          sum(rate(http_requests_total{status=~"5.."}[30m])) by (job)
          /
          sum(rate(http_requests_total[30m])) by (job)
          or vector(0)

      - record: job:http_errors:ratio6h
        expr: |
          sum(rate(http_requests_total{status=~"5.."}[6h])) by (job)
          /
          sum(rate(http_requests_total[6h])) by (job)
          or vector(0)

      # Long windows (for stability)
      - record: job:http_errors:ratio1h
        expr: |
          sum(rate(http_requests_total{status=~"5.."}[1h])) by (job)
          /
          sum(rate(http_requests_total[1h])) by (job)
          or vector(0)

      - record: job:http_errors:ratio3d
        expr: |
          sum(rate(http_requests_total{status=~"5.."}[3d])) by (job)
          /
          sum(rate(http_requests_total[3d])) by (job)
          or vector(0)

      - record: job:http_errors:ratio30d
        expr: |
          sum(rate(http_requests_total{status=~"5.."}[30d])) by (job)
          /
          sum(rate(http_requests_total[30d])) by (job)
          or vector(0)

      # ── Burn rates (pre-computed for dashboards) ──

      - record: job:http_errors:burn_rate_1h
        expr: |
          job:http_errors:ratio1h / (1 - 0.999)

      - record: job:http_errors:burn_rate_6h
        expr: |
          job:http_errors:ratio6h / (1 - 0.999)

      - record: job:http_errors:burn_rate_3d
        expr: |
          job:http_errors:ratio3d / (1 - 0.999)

  - name: slo_alerts
    interval: 30s
    rules:
      # Alerts now reference recording rules -- no inline computation
      - alert: SLOBurnRateFast
        expr: |
          (
            job:http_errors:ratio1h > (14.4 * (1 - 0.999))
          and
            job:http_errors:ratio5m > (14.4 * (1 - 0.999))
          )
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "Fast burn on {{ $labels.job }}"

      - alert: SLOBurnRateMedium
        expr: |
          (
            job:http_errors:ratio6h > (6 * (1 - 0.999))
          and
            job:http_errors:ratio30m > (6 * (1 - 0.999))
          )
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Medium burn on {{ $labels.job }}"

      - alert: SLOBurnRateSlow
        expr: |
          (
            job:http_errors:ratio3d > (1 * (1 - 0.999))
          and
            job:http_errors:ratio6h > (1 * (1 - 0.999))
          )
        for: 30m
        labels:
          severity: ticket
        annotations:
          summary: "Slow burn on {{ $labels.job }}"

      - alert: SLOBudgetExhausted
        expr: |
          job:http_errors:ratio30d > (1 * (1 - 0.999))
        for: 1h
        labels:
          severity: critical
        annotations:
          summary: "SLO VIOLATED for {{ $labels.job }}"
```

**Why recording rules improve efficiency**:

- Prometheus evaluates recording rules once per `interval` (30s) and stores
  the result. Alerts then query the pre-computed values instead of
  re-calculating `rate()` for every evaluation cycle.
- Without recording rules, each alert with a 30d window would compute
  `rate(http_requests_total[30d])` -- a query that scans 30 days of data
  on every evaluation. With recording rules, this scan happens once.
- Recording rules also enable dashboards and other consumers to use the same
  pre-computed values, ensuring consistency.

**Common mistakes**:
- Creating too many recording rules (each one consumes memory and disk)
- Not using `by (job)` which forces re-aggregation later
- Setting the recording rule interval too high (stale values) or too low
  (unnecessary compute)
- Forgetting `or vector(0)` in the ratio computation
