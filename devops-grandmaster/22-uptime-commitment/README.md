# Module 22: Uptime Commitment

> **Previous module (21):** Session management — stateless design and JWT tokens.
> **Limitation:** Your app is scalable and stateless, but how do you measure and commit to reliability? What does "99.9% uptime" actually mean?
> **This module:** SLI, SLO, SLA, error budgets, and how to define and measure reliability.

---

## 1. The Problem

Your manager says: "We need 99.99% uptime."

What does that mean? How do you measure it? What happens when you miss the target? Is 99.99% even achievable for your team? What are you trading off to get there?

Without a framework for thinking about reliability, teams either:
- Over-engineer (spending 10x to go from 99.9% to 99.99%)
- Under-invest (no targets, no measurements, surprised by outages)
- Set arbitrary goals ("four nines!") without understanding the cost

You need precise vocabulary and clear math.

---

## 2. The Naive Way — "Just Keep It Running"

"We'll do our best to keep the service up. If it goes down, we fix it fast."

**Why it fails:**

1. **No measurement.** If you do not measure uptime, you do not know if you are improving or getting worse.

2. **No prioritization.** Without a target, every reliability improvement feels equally urgent. You cannot say "this is good enough, focus on features."

3. **No accountability.** When an outage happens, there is no framework for understanding the impact. Was 30 minutes of downtime acceptable or catastrophic?

4. **No trade-off understanding.** Going from 99.9% to 99.99% might cost 10x more. Is it worth it? Without a framework, you cannot answer this.

5. **No error budget.** If you promise 99.99%, you are allowed 52 minutes of downtime per year. Once you have used that budget, you should focus on reliability, not features. Without this concept, teams ship features during outages.

---

## 3. The Right Way — SLI, SLO, SLA

Three distinct concepts that build on each other.

### SLI — Service Level Indicator

**What you measure.** A quantitative metric of your service's behavior.

```
Common SLIs:

Availability:    % of successful requests (non-5xx responses)
                 SLI = successful requests / total requests

Latency:         % of requests faster than a threshold
                 SLI = requests under 200ms / total requests

Throughput:      Requests per second the system handles

Error Rate:      % of requests that fail
                 SLI = failed requests / total requests

Correctness:     % of requests returning correct results
                 (e.g., search relevance, data accuracy)
```

**Example SLI for a web API:**
```python
# Availability SLI
total_requests = 10000
successful_requests = 9985
availability_sli = successful_requests / total_requests  # 0.9985 = 99.85%

# Latency SLI (p99 — 99th percentile)
requests_under_200ms = 9900
latency_sli = requests_under_200ms / total_requests  # 0.99 = 99%
```

### SLO — Service Level Objective

**What you promise yourself.** The target value for an SLI. Internal goal.

```
SLO Examples:

Availability SLO:  99.9% of requests succeed
Latency SLO:       99% of requests complete in under 200ms
Error Rate SLO:    Less than 0.1% of requests return errors

SLO = SLI target + time window
Example: "99.9% availability measured over a rolling 30-day window"
```

### SLA — Service Level Agreement

**What you promise customers.** A contractual commitment with consequences.

```
SLA = SLO + consequences

Example:
  "We guarantee 99.9% availability.
   If we fall below this:
   - 99.0% - 99.9%: 10% service credit
   - 95.0% - 99.0%: 25% service credit
   - Below 95.0%: 30% service credit"
```

### The Relationship

```
SLI (measurement) → SLO (internal target) → SLA (customer contract)

SLI: "We measured 99.85% availability this month"
SLO: "Our target is 99.9% availability"
SLA: "We promise customers 99.9% or we pay credits"

SLA ≤ SLO ≤ Reality

Your SLO should be tighter than your SLA (buffer).
Your SLA should be achievable (based on actual SLIs).
```

```
 ┌──────────────────────────────────────────┐
 │              SLA: 99.5%                  │  ← Customer contract
 │  ┌──────────────────────────────────┐    │
 │  │          SLO: 99.9%              │    │  ← Internal target
 │  │  ┌──────────────────────────┐    │    │
 │  │  │    SLI: 99.95%           │    │    │  ← Actual measurement
 │  │  └──────────────────────────┘    │    │
 │  └──────────────────────────────────┘    │
 └──────────────────────────────────────────┘
```

---

## 4. The Production Way

### Uptime Percentages and Downtime Equivalents

```
Uptime %    Downtime/Year    Downtime/Month    Downtime/Week
────────────────────────────────────────────────────────────
99%         3.65 days        7.31 hours        1.68 hours
99.5%       1.83 days        3.65 hours        50.4 minutes
99.9%       8.77 hours       43.8 minutes      10.1 minutes
99.95%      4.38 hours       21.9 minutes      5.04 minutes
99.99%      52.6 minutes     4.38 minutes      1.01 minutes
99.999%     5.26 minutes     26.3 seconds      6.05 seconds
```

**Reading this table:**
- 99.9% uptime means you can have ~8.77 hours of downtime per year
- 99.99% means ~52 minutes per year
- Each "nine" roughly reduces downtime by 10x

**The cost of each nine:**
```
99% → 99.9%:   Requires monitoring, on-call, runbooks
99.9% → 99.99%: Requires redundancy, automated failover, multi-region
99.99% → 99.999%: Requires active-active multi-region, chaos engineering

Cost roughly doubles with each nine.
```

### Error Budgets

An error budget is the inverse of your SLO. It is the amount of unreliability you are allowed.

```
SLO: 99.9% availability over 30 days

Total minutes in 30 days: 43,200
Allowed downtime: 43,200 × 0.001 = 43.2 minutes

Error budget: 43.2 minutes per month
```

**How to use error budgets:**

```
Error budget remaining: 43.2 minutes (100%)
→ Team can take risks: deploy new features, try new infrastructure

Error budget remaining: 20 minutes (46%)
→ Proceed with caution: more testing, slower rollouts

Error budget remaining: 5 minutes (11%)
→ Stop and fix: no new features until reliability improves

Error budget remaining: 0 minutes (0%)
→ All hands on reliability: freeze feature development
```

This creates a natural balance between feature velocity and reliability. When you are within budget, ship features. When you are burning through budget, fix reliability.

### How to Calculate Availability

```python
def calculate_availability(incidents, total_minutes_in_period):
    """
    Availability = (total_time - downtime) / total_time
    """
    downtime_minutes = sum(incident.duration_minutes for incident in incidents)
    uptime_minutes = total_minutes_in_period - downtime_minutes
    availability = uptime_minutes / total_minutes_in_period
    return availability * 100  # As percentage

# Example: 30-day period (43,200 minutes)
incidents = [
    {"duration_minutes": 15},   # Database failover
    {"duration_minutes": 5},    # Bad deployment, rolled back
    {"duration_minutes": 2},    # DNS issue
]
total = 43200

availability = calculate_availability(incidents, total)
# = (43200 - 22) / 43200 = 99.95%
```

**What counts as downtime?**
- Full outage: nobody can use the service
- Partial outage: some users affected, some not
- Degraded performance: service works but is slow

You must define this upfront. Does a 500ms latency count as "down"? Does a 5% error rate count? This is where your SLI definitions matter.

### Multi-Region for High Availability

```
Single region (99.9%):
  ┌─────────────────┐
  │   us-east-1     │
  │  ┌────┐ ┌────┐  │
  │  │App1│ │App2│  │
  │  └────┘ └────┘  │
  │  ┌──────────┐   │
  │  │ Database │   │
  │  └──────────┘   │
  └─────────────────┘

  If the region goes down (AWS outage), everything is down.

Multi-region (99.99%):
  ┌─────────────────┐     ┌─────────────────┐
  │   us-east-1     │     │   eu-west-1     │
  │  ┌────┐ ┌────┐  │     │  ┌────┐ ┌────┐  │
  │  │App1│ │App2│  │     │  │App3│ │App4│  │
  │  └────┘ └────┘  │     │  └────┘ └────┘  │
  │  ┌──────────┐   │     │  ┌──────────┐   │
  │  │ DB (RW)  │───┼─────┼──│ DB (RO)  │   │
  │  └──────────┘   │     │  └──────────┘   │
  └─────────────────┘     └─────────────────┘

  If us-east-1 goes down, eu-west-1 continues serving traffic.
```

### Real-World Example: Defining SLOs for a Web Application

```
Service: E-commerce API

SLIs:
  1. Availability: % of HTTP requests with non-5xx status
  2. Latency: % of requests completing under 300ms (p99)
  3. Freshness: % of product data updated within 5 minutes

SLOs:
  1. Availability: 99.9% over 30-day rolling window
  2. Latency: 99% of requests under 300ms over 30-day rolling window
  3. Freshness: 99% of product data is current within 5 minutes

SLA (customer-facing):
  1. Availability: 99.5% (with 10% credit if missed, 25% if below 99%)

Error Budget:
  1. Availability: 43.2 minutes of downtime per month
  2. Latency: 432 minutes of "slow" time per month
  3. Freshness: 432 minutes of stale data per month
```

**Monitoring implementation:**
```python
from prometheus_client import Counter, Histogram
import time

# Counters for availability SLI
request_total = Counter('http_requests_total', 'Total HTTP requests', ['method', 'status'])
request_success = Counter('http_requests_success_total', 'Successful requests')

# Histogram for latency SLI
request_duration = Histogram('http_request_duration_seconds', 'Request duration',
                             buckets=[0.01, 0.05, 0.1, 0.2, 0.3, 0.5, 1.0, 2.0])

@app.route('/api/products')
def get_products():
    start = time.time()

    # ... business logic ...

    duration = time.time() - start
    request_duration.observe(duration)

    request_total.labels(method='GET', status=200).inc()
    request_success.inc()

    return jsonify(products)
```

**Prometheus queries for SLO dashboards:**
```promql
# Availability SLI (last 30 days)
sum(rate(http_requests_success_total[30d]))
/
sum(rate(http_requests_total[30d]))

# Latency SLI: % of requests under 300ms (last 30 days)
sum(rate(http_request_duration_seconds_bucket{le="0.3"}[30d]))
/
sum(rate(http_request_duration_seconds_count[30d]))

# Error budget remaining (minutes)
(1 - (
  1 - (sum(rate(http_requests_success_total[30d])) / sum(rate(http_requests_total[30d])))
) / 0.001) * 43200
```

---

## 5. Hands-On Lab

### Lab: Define and Measure SLOs

**Step 1: Define your SLOs**

Create a file `slo-definition.yml`:
```yaml
service: my-web-api
team: platform

slos:
  - name: availability
    sli: "Ratio of successful (non-5xx) requests to total requests"
    target: 99.9
    window: 30d
    error_budget_minutes: 43.2

  - name: latency
    sli: "Ratio of requests completing under 200ms"
    target: 99.0
    window: 30d
    error_budget_minutes: 432

  - name: error_rate
    sli: "Ratio of 5xx responses to total responses"
    target: 0.1  # Less than 0.1% errors
    window: 30d

sla:
  availability:
    target: 99.5
    penalties:
      - range: "99.0% - 99.5%"
        action: "10% service credit"
      - range: "95.0% - 99.0%"
        action: "25% service credit"
      - range: "Below 95.0%"
        action: "30% service credit"
```

**Step 2: Build a simple SLI calculator**

**slo_calculator.py:**
```python
from dataclasses import dataclass
from datetime import datetime, timedelta

@dataclass
class Incident:
    start: datetime
    end: datetime

    @property
    def duration_minutes(self):
        return (self.end - self.start).total_seconds() / 60

class SLOCalculator:
    def __init__(self, target_percent: float, window_days: int = 30):
        self.target = target_percent
        self.window_days = window_days

    @property
    def total_minutes(self):
        return self.window_days * 24 * 60

    @property
    def allowed_downtime_minutes(self):
        return self.total_minutes * (1 - self.target / 100)

    def calculate_availability(self, incidents: list[Incident]) -> float:
        downtime = sum(i.duration_minutes for i in incidents)
        uptime = self.total_minutes - downtime
        return (uptime / self.total_minutes) * 100

    def error_budget_remaining(self, incidents: list[Incident]) -> dict:
        downtime = sum(i.duration_minutes for i in incidents)
        budget = self.allowed_downtime_minutes
        remaining = max(0, budget - downtime)
        remaining_percent = (remaining / budget) * 100

        return {
            "target": f"{self.target}%",
            "allowed_downtime_minutes": round(budget, 2),
            "actual_downtime_minutes": round(downtime, 2),
            "remaining_minutes": round(remaining, 2),
            "remaining_percent": round(remaining_percent, 2),
            "status": self._budget_status(remaining_percent)
        }

    def _budget_status(self, remaining_percent):
        if remaining_percent > 50:
            return "HEALTHY — proceed with feature work"
        elif remaining_percent > 20:
            return "CAUTION — increase testing, slow rollouts"
        elif remaining_percent > 0:
            return "WARNING — focus on reliability"
        else:
            return "EXHAUSTED — freeze features, fix reliability"


# Example usage
if __name__ == "__main__":
    # 99.9% availability SLO
    slo = SLOCalculator(target_percent=99.9, window_days=30)

    print(f"SLO: {slo.target}% availability")
    print(f"Window: {slo.window_days} days ({slo.total_minutes} minutes)")
    print(f"Allowed downtime: {slo.allowed_downtime_minutes} minutes")
    print()

    # Simulate incidents this month
    now = datetime.now()
    incidents = [
        Incident(now - timedelta(days=20), now - timedelta(days=20, minutes=-15)),
        Incident(now - timedelta(days=10), now - timedelta(days=10, minutes=-8)),
        Incident(now - timedelta(days=3), now - timedelta(days=3, minutes=-5)),
    ]

    # Calculate
    availability = slo.calculate_availability(incidents)
    budget = slo.error_budget_remaining(incidents)

    print(f"Availability: {availability:.4f}%")
    print(f"Error Budget: {budget}")
```

Run it:
```bash
python slo_calculator.py
```

Expected output:
```
SLO: 99.9% availability
Window: 30 days (43200 minutes)
Allowed downtime: 43.2 minutes

Availability: 99.9352%
Error Budget: {
  'target': '99.9%',
  'allowed_downtime_minutes': 43.2,
  'actual_downtime_minutes': 28.0,
  'remaining_minutes': 15.2,
  'remaining_percent': 35.19,
  'status': 'CAUTION — increase testing, slow rollouts'
}
```

---

## 6. Limitation

You have defined your reliability targets. You measure SLIs, track error budgets, and know when to prioritize reliability over features.

But there is a gap between measuring uptime and actually maintaining it. When you deploy a new version, the old containers are stopped and new ones start. During that transition, in-flight requests are dropped. Users see errors. That counts against your error budget.

You need a way to restart or redeploy without dropping active requests.

---

## 7. Next Topic

**Module 23: Graceful Shutdown** — Handling in-flight requests during restarts, connection draining, and zero-downtime deployments. [Go to Module 23 →](../23-graceful-shutdown/README.md)
