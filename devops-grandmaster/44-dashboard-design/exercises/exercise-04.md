# Exercise 04: Incident Response Dashboard

## Objective

Design a specialized dashboard for use during active incidents. Unlike a
monitoring dashboard that shows "is everything OK?", an incident dashboard
answers "what is broken, who is affected, and what changed?"

## Background

During an incident, operators need answers fast:

1. **What is the blast radius?** (how many users/requests are affected?)
2. **What changed?** (recent deployments, config changes, traffic spikes)
3. **Where is the bottleneck?** (which component is failing?)
4. **What is the SLO impact?** (how much error budget was consumed?)

A general-purpose dashboard is too noisy during an incident. A purpose-built
incident response dashboard strips away everything that is not immediately
relevant and highlights what matters.

## Instructions

### Part A -- Design the incident dashboard layout

Design the panel layout for an incident response dashboard. The dashboard
should have 5 rows, each answering a specific question:

| Row | Question | Panels |
|-----|----------|--------|
| 1 | What is the incident status? | ? |
| 2 | What is the blast radius? | ? |
| 3 | What changed recently? | ? |
| 4 | Which component is failing? | ? |
| 5 | What is the SLO impact? | ? |

For each row, decide:
- What panels to include (type, title, purpose)
- How many panels per row (aim for 2-4)
- What time range each panel should cover

### Part B -- Write the PromQL queries

Write PromQL queries for the following panels:

#### Blast Radius Panel: Affected Requests (stat)

Show the total number of failed requests in the last 15 minutes, compared to
the same period yesterday (as a percentage change).

```promql
# Current period (last 15 minutes)
# YOUR QUERY HERE

# Yesterday same period (for comparison)
# YOUR QUERY HERE
```

#### Blast Radius Panel: Error Rate by Service (bar chart)

Show the error rate for each service, sorted highest first. Use the `topk`
function to show only the 5 worst services.

#### What Changed Panel: Deployment Timeline (annotations)

Describe how to configure Grafana annotations to show:
- Kubernetes deployment events (new pod creation)
- ConfigMap/Secret changes
- Horizontal pod autoscaler events

Write the Prometheus queries for each annotation source.

#### Component Failure Panel: Dependency Health (state timeline)

Show the health status (up/down) of key dependencies over the last hour:
- PostgreSQL primary
- PostgreSQL replica
- Redis primary
- External payment API

Write a PromQL query that returns 1 for healthy and 0 for down.

#### SLO Impact Panel: Budget Consumed Today (gauge)

Show how much error budget was consumed specifically today (not the rolling
30-day window), as a gauge from 0 to 100%.

### Part C -- Comparison queries

A key feature of an incident dashboard is comparing current behavior to a
baseline. Write PromQL queries that compare:

1. **Current error rate vs. same time yesterday**
2. **Current latency vs. same time last week**
3. **Current request rate vs. average of the last 7 days**

Use Prometheus `offset` modifier for the historical comparisons.

### Part D -- Dashboard JSON for the status row

Write the Grafana JSON for Row 1 (Incident Status). This row should contain:

1. **Incident Status** (stat panel): A text panel showing the current incident
   phase (Investigating / Identified / Monitoring / Resolved). This should be
   driven by a Grafana variable that the incident commander sets manually.

2. **Duration** (stat panel): How long the incident has been active, calculated
   from the first error spike.

3. **Affected Users** (stat panel): Estimated number of affected users (derived
   from error count).

Write the full Grafana JSON for these three panels including their gridPos,
targets, and fieldConfig.

### Part E -- Incident workflow integration

Describe how you would integrate the incident dashboard with your incident
management workflow:

1. How would you automatically switch to the incident dashboard when a critical
   alert fires?
2. How would you persist the incident timeline (annotations) after the incident
   is resolved for post-mortem review?
3. How would you share the dashboard with stakeholders who do not have Grafana
   access?

## Success Criteria

- [ ] Dashboard layout has a clear logical flow from "what is happening" to "why"
- [ ] Blast radius queries correctly quantify user impact
- [ ] Comparison queries use Prometheus `offset` correctly
- [ ] Dependency health queries cover the critical path
- [ ] Grafana JSON is valid and includes the manual status variable
- [ ] Incident workflow integration is practical and realistic

## Hints

<details>
<summary>Hint 1: Comparison with offset</summary>

```promql
# Current error rate
sum(rate(http_requests_total{status=~"5.."}[5m]))

# Same time yesterday
sum(rate(http_requests_total{status=~"5.."}[5m] offset 1d))

# Percentage change
(
  sum(rate(http_requests_total{status=~"5.."}[5m]))
  - sum(rate(http_requests_total{status=~"5.."}[5m] offset 1d))
) / sum(rate(http_requests_total{status=~"5.."}[5m] offset 1d)) * 100
```

</details>

<details>
<summary>Hint 2: Dependency health as a number</summary>

```promql
# Returns 1 if up, 0 if down
up{job="postgres-primary"}

# Or for a specific check:
probe_success{instance="https://payment-api.example.com/health"}
```

Use a `state_timeline` panel type to show up/down transitions over time.

</details>

<details>
<summary>Hint 3: Manual variable for incident status</summary>

Use a custom variable with a textbox input:

```json
{
  "name": "incident_status",
  "type": "textbox",
  "label": "Incident Status",
  "current": {"text": "Investigating", "value": "Investigating"},
  "options": [
    {"text": "Investigating", "value": "Investigating"},
    {"text": "Identified", "value": "Identified"},
    {"text": "Monitoring", "value": "Monitoring"},
    {"text": "Resolved", "value": "Resolved"}
  ]
}
```

The incident commander updates this variable as the incident progresses.

</details>

<details>
<summary>Hint 4: Duration from first error spike</summary>

Use Grafana's math expressions or a PromQL query that finds the time of the
first error:

```promql
# Time since the error rate first exceeded 1%
time() - min_over_time(
  (timestamp(changes(http_requests_total{status=~"5.."}[1m]) > 0))[1h:1m]
)
```

Or more practically, use an annotation to mark the incident start time and
calculate duration from that.

</details>
