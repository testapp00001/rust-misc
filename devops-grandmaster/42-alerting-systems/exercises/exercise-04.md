# Exercise 04: Alert Deduplication and Grouping

## Type: Challenge

## Objective

Understand how AlertManager deduplicates and groups alerts, configure grouping behavior to reduce notification noise, analyze the impact of different `group_by` strategies, and build a tool that visualizes how alerts flow through the grouping and deduplication pipeline.

## Prerequisites

- Completion of Exercises 01-03
- Docker and Docker Compose
- Understanding of AlertManager routing and inhibition

## Part A: Understanding Deduplication and Grouping

### Task 1: Deduplication Concepts

Answer the following questions:

1. What is the difference between alert **deduplication** and alert **grouping**?
2. How does AlertManager determine that two alerts are "the same" (duplicates)?
3. What is a "group" in AlertManager terms? What determines which alerts belong to the same group?
4. What happens when you add or change the `group_by` labels? How does this affect which notifications are sent?

### Task 2: Grouping Strategy Analysis

You have a microservices platform with 10 services, each running on 5 pods across 3 clusters. A network issue causes all pods in cluster `us-east-1` to lose connectivity simultaneously. This triggers `InstanceDown` for all 50 pods.

For each of the following `group_by` configurations, calculate:
- How many notification groups are created?
- How many individual webhook calls are made (assuming `group_wait: 30s`, `group_interval: 5m`)?

**Configuration A:**
```yaml
group_by: ['alertname']
```

**Configuration B:**
```yaml
group_by: ['alertname', 'cluster']
```

**Configuration C:**
```yaml
group_by: ['alertname', 'cluster', 'service']
```

**Configuration D:**
```yaml
group_by: ['alertname', 'instance']
```

**Configuration E:**
```yaml
group_by: [...]  # all labels
```

Which configuration produces the fewest notifications? Which produces the most? Which is the best balance for this scenario?

### Task 3: Timing Parameters Deep Dive

Explain what happens in each of the following scenarios. Walk through the timeline of events.

**Scenario A: Rapid-fire alerts**
```
group_wait: 30s
group_interval: 5m
repeat_interval: 4h
```

Timeline:
- T+0s: Alert 1 (HighErrorRate, service=api) fires
- T+5s: Alert 2 (HighErrorRate, service=api, different instance) fires
- T+10s: Alert 3 (HighErrorRate, service=api, another instance) fires
- T+30s: ???
- T+5m: Alert 4 (HighLatency, service=api) fires (same group if grouped by service)
- T+5m30s: ???

**Scenario B: Alert resolves and re-fires**
```
group_wait: 30s
group_interval: 5m
repeat_interval: 4h
```

Timeline:
- T+0s: HighErrorRate fires
- T+30s: Notification sent
- T+2m: Alert resolves
- T+3m: Alert fires again
- T+3m30s: ???
- T+8m: Alert fires again (still active)
- T+8m30s: ???

**Scenario C: New alert joins an existing group**
```
group_wait: 30s
group_interval: 5m
```

Timeline:
- T+0s: HighErrorRate (service=api) fires
- T+30s: Notification sent with 1 alert
- T+3m: HighLatency (service=api) fires (same group)
- T+3m30s: ???
- T+5m: InstanceDown (service=api) fires (same group)
- T+5m30s: ???

## Part B: Build a Grouping Simulator

### Task 4: Implement an Alert Grouping Simulator

Write a Python script (`grouping_simulator.py`) that simulates AlertManager's grouping behavior. The simulator should:

1. Accept a `group_by` list of label names
2. Accept timing parameters: `group_wait`, `group_interval`, `repeat_interval`
3. Accept a stream of incoming alerts (with labels, annotations, and timestamps)
4. Output a log of notifications that would be sent, including:
   - Timestamp of the notification
   - Group key (the combination of `group_by` label values)
   - Number of alerts in the group
   - Alert names and summaries in the group

The simulator should implement the following AlertManager grouping rules:

1. **Group assignment**: Alerts with the same values for all `group_by` labels belong to the same group.
2. **group_wait**: When a new group is created (first alert), wait `group_wait` before sending the first notification. Additional alerts arriving during this window are included in the first notification.
3. **group_interval**: After sending a notification, wait `group_interval` before sending another notification for the same group. Alerts arriving during this window are batched.
4. **repeat_interval**: If an alert group is still firing after `repeat_interval`, re-send the notification.

### Task 5: Test the Simulator

Run the simulator with the following test cases and record the output:

**Test Case 1: Single alert group**
```python
alerts = [
    {"labels": {"alertname": "HighErrorRate", "service": "api", "instance": "pod-1"}, "time": 0},
]
group_by = ["alertname", "service"]
group_wait = 30
group_interval = 300
repeat_interval = 14400
```

**Test Case 2: Multiple instances, same group**
```python
alerts = [
    {"labels": {"alertname": "HighErrorRate", "service": "api", "instance": f"pod-{i}"}, "time": i * 5}
    for i in range(10)
]
group_by = ["alertname", "service"]
group_wait = 30
group_interval = 300
repeat_interval = 14400
```

**Test Case 3: Different services, different groups**
```python
alerts = [
    {"labels": {"alertname": "HighErrorRate", "service": "api", "instance": "pod-1"}, "time": 0},
    {"labels": {"alertname": "HighErrorRate", "service": "payment", "instance": "pod-1"}, "time": 5},
    {"labels": {"alertname": "HighErrorRate", "service": "auth", "instance": "pod-1"}, "time": 10},
]
group_by = ["alertname", "service"]
group_wait = 30
group_interval = 300
repeat_interval = 14400
```

**Test Case 4: Over-grouping problem**
```python
alerts = [
    {"labels": {"alertname": "HighErrorRate", "service": "api", "instance": "pod-1"}, "time": 0},
    {"labels": {"alertname": "HighCPU", "service": "api", "instance": "pod-1"}, "time": 5},
    {"labels": {"alertname": "DiskFull", "service": "api", "instance": "pod-1"}, "time": 10},
]
group_by = ["service"]
group_wait = 30
group_interval = 300
repeat_interval = 14400
```

**Test Case 5: Under-grouping problem**
```python
alerts = [
    {"labels": {"alertname": "InstanceDown", "service": "api", "instance": f"pod-{i}", "cluster": "us-east-1"}, "time": 0}
    for i in range(20)
]
group_by = ["alertname", "instance"]
group_wait = 30
group_interval = 300
repeat_interval = 14400
```

For each test case, answer:
1. How many notifications are sent?
2. How many alerts are in each notification?
3. Is this the optimal grouping for this scenario? Why or why not?

### Task 6: Visualize the Grouping Pipeline

Extend your simulator to produce a text-based visualization of the grouping pipeline:

```
INCOMING ALERTS                    GROUPS                         NOTIFICATIONS
============                       ======                         =============

[HighErrorRate,api,pod-1] T=0  --> [api: HighErrorRate]      --> (waiting for group_wait)
[HighErrorRate,api,pod-2] T=5  --> [api: HighErrorRate]      --> (added to pending)
[HighErrorRate,api,pod-3] T=10 --> [api: HighErrorRate]      --> (added to pending)
                                   (group_wait=30s elapsed)    --> NOTIFICATION: 3 alerts
[HighErrorRate,pay,pod-1] T=35 --> [pay: HighErrorRate]      --> (waiting for group_wait)
                                   (group_wait=30s elapsed)    --> NOTIFICATION: 1 alert
[HighCPU,api,pod-1]       T=80 --> [api: HighCPU]            --> (waiting for group_wait)
                                   (group_wait=30s elapsed)    --> NOTIFICATION: 1 alert
```

## Part C: Analysis

### Task 7: Grouping Trade-offs

For each scenario below, recommend a `group_by` strategy and explain your reasoning:

1. **Small team (3 engineers)** with 5 services. They want to minimize pages but need to know which service is affected.
2. **Large team (20 engineers)** with dedicated teams per service. Each team has its own Slack channel and PagerDuty service.
3. **Platform team** that only cares about infrastructure alerts (nodes, clusters, networking). Application alerts go to other teams.
4. **On-call during a major outage** where 200 alerts fire in 2 minutes from a cascading failure.

### Task 8: The Grouping Paradox

Explain the "grouping paradox" in your own words:

- If you group too aggressively (by `alertname` only), you get fewer notifications but lose context about which service or instance is affected.
- If you group too granularly (by every label), you get maximum context but flood the on-call with individual notifications.

How do you resolve this tension? What real-world strategies can you use?

## Success Criteria

- [ ] You can explain the difference between deduplication and grouping
- [ ] You can calculate the number of groups and notifications for any `group_by` configuration
- [ ] You can trace the timeline of alerts through `group_wait`, `group_interval`, and `repeat_interval`
- [ ] The grouping simulator correctly implements AlertManager's grouping rules
- [ ] You can identify over-grouping and under-grouping problems
- [ ] You can recommend appropriate `group_by` strategies for different team sizes and architectures

## Hints

<details>
<summary>Hint 1: Deduplication vs grouping</summary>

**Deduplication**: AlertManager deduplicates alerts with the same fingerprint (same label set). If Prometheus sends the same alert twice (e.g., because of a scrape retry), AlertManager recognizes it as the same alert and does not create a duplicate.

**Grouping**: AlertManager groups *different* alerts together into a single notification based on the `group_by` labels. This reduces the number of notifications. For example, 10 `InstanceDown` alerts for the same service can be grouped into one notification.

Deduplication is automatic. Grouping is configured.

</details>

<details>
<summary>Hint 2: Group key calculation</summary>

The group key is the combination of all `group_by` label values. For example:

```yaml
group_by: ['alertname', 'cluster', 'service']
```

An alert with labels `{alertname: "HighErrorRate", cluster: "us-east-1", service: "api", instance: "pod-1"}` has group key `HighErrorRate/us-east-1/api`. The `instance` label is NOT part of the group key, so all instances for the same service/cluster/alertname are grouped together.

If `group_by: ['...']` (the special "all labels" value), each unique label set creates its own group. This effectively disables grouping.

</details>

<details>
<summary>Hint 3: Timing parameter interactions</summary>

- `group_wait`: Only applies to the *first* notification for a new group. After the first notification, `group_interval` takes over.
- `group_interval`: Minimum time between notifications for the same group. New alerts arriving during this window are batched into the next notification.
- `repeat_interval`: If the group is still firing (at least one alert is active), re-send the notification after this interval. This is for alerts that have been firing for a long time.

The key insight: `group_interval` and `repeat_interval` interact. If `group_interval` is 5 minutes and `repeat_interval` is 4 hours, the group will get a notification every 5 minutes if new alerts keep joining, but only every 4 hours if no new alerts arrive.

</details>

<details>
<summary>Hint 4: Implementing the simulator</summary>

A simple approach:

```python
import time
from collections import defaultdict

class AlertGroup:
    def __init__(self, group_key):
        self.group_key = group_key
        self.alerts = {}  # fingerprint -> alert
        self.last_notification_time = None
        self.created_at = None

class GroupingSimulator:
    def __init__(self, group_by, group_wait, group_interval, repeat_interval):
        self.group_by = group_by
        self.group_wait = group_wait
        self.group_interval = group_interval
        self.repeat_interval = repeat_interval
        self.groups = {}  # group_key -> AlertGroup
        self.notifications = []

    def get_group_key(self, labels):
        return '/'.join(str(labels.get(k, '')) for k in self.group_by)

    def process_alert(self, alert, current_time):
        group_key = self.get_group_key(alert['labels'])
        if group_key not in self.groups:
            self.groups[group_key] = AlertGroup(group_key)
            self.groups[group_key].created_at = current_time
        self.groups[group_key].alerts[alert.get('fingerprint', str(alert))] = alert

    def check_notifications(self, current_time):
        for key, group in self.groups.items():
            if group.last_notification_time is None:
                if current_time - group.created_at >= self.group_wait:
                    self.send_notification(group, current_time)
            elif current_time - group.last_notification_time >= self.group_interval:
                self.send_notification(group, current_time)

    def send_notification(self, group, current_time):
        self.notifications.append({
            'time': current_time,
            'group_key': group.group_key,
            'alert_count': len(group.alerts),
            'alerts': list(group.alerts.values()),
        })
        group.last_notification_time = current_time
```

</details>

<details>
<summary>Hint 5: The "all labels" group_by</summary>

AlertManager supports a special value `group_by: ['...']` which means "group by all labels." This effectively creates one group per unique alert (since every alert has a unique combination of all labels). Use this when you want no grouping at all — every alert gets its own notification. This is almost never what you want in production.

To simulate this in your code, if `group_by == ['...']`, use all labels from the alert to compute the group key.

</details>
