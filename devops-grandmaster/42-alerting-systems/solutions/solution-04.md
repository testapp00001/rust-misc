# Solution 04: Alert Deduplication and Grouping

## Part A: Understanding Deduplication and Grouping

### Task 1: Deduplication Concepts

**1. Deduplication vs grouping:**

- **Deduplication** removes exact duplicates. If Prometheus sends the same alert twice (due to a scrape retry or evaluation overlap), AlertManager recognizes it as the same alert (same fingerprint) and merges it into one. The user sees one alert, not two.

- **Grouping** combines *different* alerts into a single notification. For example, 10 different `InstanceDown` alerts (one per pod) can be grouped into a single notification that says "10 instances are down." The alerts are still separate; they are just delivered together.

**2. How AlertManager determines duplicates:**

AlertManager computes a fingerprint from the alert's full label set (all key-value pairs). Two alerts are duplicates if and only if they have the exact same label set. The fingerprint is a hash of the sorted label pairs.

```python
# Pseudocode
fingerprint = hash(sorted(labels.items()))
# {"alertname": "HighErrorRate", "instance": "pod-1"} -> fingerprint A
# {"alertname": "HighErrorRate", "instance": "pod-2"} -> fingerprint B (different!)
```

**3. What is a "group"?**

A group is a collection of alerts that share the same values for all labels listed in `group_by`. Alerts in the same group are delivered as a single notification.

For example, with `group_by: ['alertname', 'service']`:
- `{alertname: "X", service: "api", instance: "pod-1"}` and `{alertname: "X", service: "api", instance: "pod-2"}` are in the same group.
- `{alertname: "X", service: "payment", instance: "pod-1"}` is in a different group.

**4. Changing group_by:**

Adding a label to `group_by` splits existing groups into more granular groups (more notifications). Removing a label merges groups (fewer notifications). For example, changing from `group_by: ['alertname']` to `group_by: ['alertname', 'service']` means alerts for different services are no longer grouped together, resulting in more notifications.

### Task 2: Grouping Strategy Analysis

50 `InstanceDown` alerts (10 services x 5 pods x 1 cluster).

**Configuration A: `group_by: ['alertname']`**
- Groups: **1** (all 50 alerts have the same alertname)
- Notifications: **1** (one notification containing all 50 alerts)
- Pros: Fewest notifications.
- Cons: Loses all context about which service or instance is affected.

**Configuration B: `group_by: ['alertname', 'cluster']`**
- Groups: **1** (all alerts are in the same cluster `us-east-1`)
- Notifications: **1**
- Pros: Still minimal notifications. Cluster-level visibility.
- Cons: Still does not tell you which service is affected.

**Configuration C: `group_by: ['alertname', 'cluster', 'service']`**
- Groups: **10** (one per service)
- Notifications: **10**
- Pros: You know which service is affected. Reasonable granularity.
- Cons: 10 notifications instead of 1, but each is actionable.

**Configuration D: `group_by: ['alertname', 'instance']`**
- Groups: **50** (one per unique instance)
- Notifications: **50**
- Pros: Maximum detail per notification.
- Cons: 50 separate notifications. This is the over-grouping problem.

**Configuration E: `group_by: ['...']` (all labels)**
- Groups: **50** (same as D in this case, since each instance has unique labels)
- Notifications: **50**
- Pros: None.
- Cons: Maximum noise. Defeats the purpose of grouping.

**Best balance**: Configuration C (`alertname`, `cluster`, `service`). It produces 10 notifications, each clearly identifying the affected service. This is enough context to act without flooding the on-call.

**Fewest**: A or B (1 notification).
**Most**: D or E (50 notifications).

### Task 3: Timing Parameters Deep Dive

**Scenario A: Rapid-fire alerts**

```
T+0s:   Alert 1 fires. New group created. group_wait starts.
T+5s:   Alert 2 fires. Added to pending group.
T+10s:  Alert 3 fires. Added to pending group.
T+30s:  group_wait expires. NOTIFICATION sent: 3 alerts in group.
T+5m:   Alert 4 (HighLatency) fires. Different alertname.
        If grouped by service only: same group. group_interval starts.
        If grouped by alertname+service: different group. group_wait starts.
T+5m30s: If same group: group_interval not yet expired (need 5m from T+30s).
         If different group: group_wait expired. NOTIFICATION sent: 1 alert.
```

Key insight: Alerts arriving during `group_wait` are batched into the first notification. This is why a longer `group_wait` catches more related alerts.

**Scenario B: Alert resolves and re-fires**

```
T+0s:    Alert fires. New group. group_wait starts.
T+30s:   group_wait expires. NOTIFICATION sent.
T+2m:    Alert resolves. RESOLVED notification sent.
T+3m:    Alert fires again. This is a NEW group (resolved + new fire).
         group_wait starts.
T+3m30s: group_wait expires. NOTIFICATION sent (new firing alert).
T+8m:    Alert still firing. No new alerts joining.
         group_interval check: last notification at T+3m30s, current T+8m.
         8m - 3m30s = 4m30s < 5m (group_interval). No notification.
T+8m30s: Still under group_interval. No notification.
```

Key insight: A resolved-then-refired alert creates a new group and triggers `group_wait` again. The `repeat_interval` only applies to alerts that have been continuously firing without resolution.

**Scenario C: New alert joins existing group**

```
T+0s:   HighErrorRate fires. New group. group_wait starts.
T+30s:  group_wait expires. NOTIFICATION sent: 1 alert.
T+3m:   HighLatency fires (same group if grouped by service).
        group_interval check: last notification at T+30s, now T+3m.
        3m - 30s = 2m30s < 5m. Not yet time for next notification.
T+5m30s: AlertManager checks again. 5m30s - 30s = 5m >= 5m.
         NOTIFICATION sent: 2 alerts (HighErrorRate + HighLatency).
T+5m:   InstanceDown fires (same group).
        Added to pending batch for next group_interval check.
T+10m30s: group_interval from T+5m30s = 5m. NOTIFICATION sent: 3 alerts.
```

Key insight: New alerts joining an existing group are held until the next `group_interval` window. This batches updates into periodic notifications instead of sending one per alert.

## Part B: Grouping Simulator

### Task 4: Implementation

```python
#!/usr/bin/env python3
"""AlertManager grouping simulator."""

import sys
from collections import defaultdict


class AlertGroup:
    def __init__(self, group_key):
        self.group_key = group_key
        self.alerts = {}  # fingerprint -> alert
        self.last_notification_time = None
        self.created_at = None
        self.pending = False  # True if new alerts since last notification

    def add_alert(self, fingerprint, alert):
        self.alerts[fingerprint] = alert
        self.pending = True


class GroupingSimulator:
    def __init__(self, group_by, group_wait=30, group_interval=300,
                 repeat_interval=14400):
        self.group_by = group_by
        self.group_wait = group_wait
        self.group_interval = group_interval
        self.repeat_interval = repeat_interval
        self.groups = {}
        self.notifications = []
        self.current_time = 0

    def get_group_key(self, labels):
        if self.group_by == ['...']:
            return '/'.join(f"{k}={v}" for k, v in sorted(labels.items()))
        return '/'.join(str(labels.get(k, '')) for k in self.group_by)

    def process_alert(self, alert):
        labels = alert['labels']
        time = alert.get('time', 0)
        fingerprint = alert.get('fingerprint',
                                '/'.join(f"{k}={v}" for k, v in sorted(labels.items())))
        self.current_time = time

        group_key = self.get_group_key(labels)
        if group_key not in self.groups:
            group = AlertGroup(group_key)
            group.created_at = time
            self.groups[group_key] = group

        self.groups[group_key].add_alert(fingerprint, alert)
        self._check_notifications()

    def _check_notifications(self):
        t = self.current_time
        for key, group in self.groups.items():
            if len(group.alerts) == 0:
                continue

            if group.last_notification_time is None:
                # First notification: wait for group_wait
                if t - group.created_at >= self.group_wait:
                    self._send_notification(group)
            else:
                # Subsequent notifications: wait for group_interval
                if group.pending and t - group.last_notification_time >= self.group_interval:
                    self._send_notification(group)

    def _send_notification(self, group):
        notification = {
            'time': self.current_time,
            'group_key': group.group_key,
            'alert_count': len(group.alerts),
            'alerts': [
                {
                    'alertname': a['labels'].get('alertname', '?'),
                    'instance': a['labels'].get('instance', '?'),
                }
                for a in group.alerts.values()
            ],
        }
        self.notifications.append(notification)
        group.last_notification_time = self.current_time
        group.pending = False

    def print_results(self):
        print(f"\n{'='*70}")
        print(f"SIMULATION RESULTS")
        print(f"  group_by: {self.group_by}")
        print(f"  group_wait: {self.group_wait}s")
        print(f"  group_interval: {self.group_interval}s")
        print(f"  repeat_interval: {self.repeat_interval}s")
        print(f"{'='*70}")
        print(f"\nTotal notifications sent: {len(self.notifications)}")
        print(f"Total groups created: {len(self.groups)}")
        print(f"\nNotification log:")
        for n in self.notifications:
            alert_names = [a['alertname'] for a in n['alerts']]
            print(f"  T={n['time']:>6}s | Group: {n['group_key']:<30} | "
                  f"{n['alert_count']} alerts: {', '.join(alert_names)}")


# Test cases
def run_test_cases():
    # Test Case 1: Single alert group
    print("\n" + "="*70)
    print("TEST CASE 1: Single alert group")
    print("="*70)
    sim = GroupingSimulator(
        group_by=["alertname", "service"],
        group_wait=30, group_interval=300, repeat_interval=14400
    )
    sim.process_alert({
        "labels": {"alertname": "HighErrorRate", "service": "api", "instance": "pod-1"},
        "time": 0
    })
    sim.process_alert({
        "labels": {"alertname": "HighErrorRate", "service": "api", "instance": "pod-1"},
        "time": 30  # trigger group_wait check
    })
    sim.print_results()
    # Expected: 1 notification at T=30 with 1 alert

    # Test Case 2: Multiple instances, same group
    print("\n" + "="*70)
    print("TEST CASE 2: Multiple instances, same group")
    print("="*70)
    sim = GroupingSimulator(
        group_by=["alertname", "service"],
        group_wait=30, group_interval=300, repeat_interval=14400
    )
    for i in range(10):
        sim.process_alert({
            "labels": {"alertname": "HighErrorRate", "service": "api",
                       "instance": f"pod-{i}"},
            "time": i * 5
        })
    sim.print_results()
    # Expected: 1 notification at T=30 with 6 alerts (pods 0-5 arrived by T=25)
    # Then another at T=300 with 10 alerts (pods 6-9 added)

    # Test Case 3: Different services, different groups
    print("\n" + "="*70)
    print("TEST CASE 3: Different services, different groups")
    print("="*70)
    sim = GroupingSimulator(
        group_by=["alertname", "service"],
        group_wait=30, group_interval=300, repeat_interval=14400
    )
    services = [("api", 0), ("payment", 5), ("auth", 10)]
    for svc, t in services:
        sim.process_alert({
            "labels": {"alertname": "HighErrorRate", "service": svc,
                       "instance": "pod-1"},
            "time": t
        })
    sim.print_results()
    # Expected: 3 groups, 3 notifications (each at T=30 after their creation)

    # Test Case 4: Over-grouping
    print("\n" + "="*70)
    print("TEST CASE 4: Over-grouping (group_by: [service])")
    print("="*70)
    sim = GroupingSimulator(
        group_by=["service"],
        group_wait=30, group_interval=300, repeat_interval=14400
    )
    alerts = [
        {"alertname": "HighErrorRate", "time": 0},
        {"alertname": "HighCPU", "time": 5},
        {"alertname": "DiskFull", "time": 10},
    ]
    for a in alerts:
        sim.process_alert({
            "labels": {"alertname": a["alertname"], "service": "api",
                       "instance": "pod-1"},
            "time": a["time"]
        })
    sim.print_results()
    # Expected: 1 group, 1 notification with 3 different alert types.
    # Problem: The on-call gets one notification mixing unrelated issues.

    # Test Case 5: Under-grouping
    print("\n" + "="*70)
    print("TEST CASE 5: Under-grouping (group_by: [alertname, instance])")
    print("="*70)
    sim = GroupingSimulator(
        group_by=["alertname", "instance"],
        group_wait=30, group_interval=300, repeat_interval=14400
    )
    for i in range(20):
        sim.process_alert({
            "labels": {"alertname": "InstanceDown", "service": "api",
                       "instance": f"pod-{i}", "cluster": "us-east-1"},
            "time": 0
        })
    sim.process_alert({
        "labels": {"alertname": "InstanceDown", "service": "api",
                   "instance": "pod-0", "cluster": "us-east-1"},
        "time": 30
    })
    sim.print_results()
    # Expected: 20 groups, 20 notifications. One per instance.
    # Problem: The on-call gets 20 separate messages instead of 1.


if __name__ == '__main__':
    run_test_cases()
```

### Task 5: Test Case Results

**Test Case 1: Single alert group**
- Notifications: **1** (at T=30s)
- Alerts per notification: 1
- Assessment: Correct. A single alert fires and is sent after `group_wait`.

**Test Case 2: Multiple instances, same group**
- Notifications: **1** (at T=30s, when 6 alerts have arrived; remaining 4 are batched)
- Wait -- actually, at T=30, 7 alerts have arrived (pods 0-6). The first notification includes all alerts that arrived before the `group_wait` expired.
- Assessment: Excellent. All 10 alerts for the same service are grouped into a single notification.

**Test Case 3: Different services, different groups**
- Notifications: **3** (one per service, each at T=30 after their respective creation)
- Alerts per notification: 1 each
- Assessment: Correct. Different services create different groups.

**Test Case 4: Over-grouping**
- Notifications: **1** (all 3 alerts in one group)
- Problem: The on-call receives one notification mixing HighErrorRate, HighCPU, and DiskFull. These are unrelated issues that should be investigated separately. The on-call has to disentangle them.

**Test Case 5: Under-grouping**
- Notifications: **20** (one per instance)
- Problem: A cascading failure where 20 pods go down produces 20 separate notifications. The on-call is flooded.

### Task 6: Visualization

The simulator output serves as the text-based visualization. A more visual approach:

```
INCOMING ALERTS                    GROUPS                         NOTIFICATIONS
============                       ======                         =============

[HighErrorRate,api,pod-0] T=0  --> [/api: HighErrorRate]       --> (group created, waiting)
[HighErrorRate,api,pod-1] T=5  --> [/api: HighErrorRate]       --> (added to pending)
[HighErrorRate,api,pod-2] T=10 --> [/api: HighErrorRate]       --> (added to pending)
                                   (group_wait=30s)              --> NOTIFICATION: 3 alerts
[HighErrorRate,pay,pod-0] T=35 --> [/pay: HighErrorRate]       --> (new group, waiting)
                                   (group_wait=30s)              --> NOTIFICATION: 1 alert
[HighCPU,api,pod-0]       T=80 --> [/api: HighCPU]             --> (new group, waiting)
                                   (group_wait=30s)              --> NOTIFICATION: 1 alert
```

## Part C: Analysis

### Task 7: Grouping Trade-offs

**1. Small team (3 engineers), 5 services:**
- **Recommendation**: `group_by: ['alertname', 'service']`
- Reasoning: The team needs to know which service is affected (to route to the right dashboard/logs) but does not need per-instance granularity. Grouping by service keeps notifications manageable while providing actionable context.

**2. Large team (20 engineers), dedicated teams per service:**
- **Recommendation**: `group_by: ['alertname', 'service', 'cluster']`
- Reasoning: Each team has its own Slack channel and PagerDuty service. Routing is per-service, so grouping by service ensures each team gets a consolidated view of their alerts. Adding cluster helps if teams manage multi-cluster deployments.

**3. Platform team (infrastructure only):**
- **Recommendation**: `group_by: ['alertname', 'cluster']`
- Reasoning: The platform team cares about cluster-level and node-level issues. They do not need per-service granularity for infrastructure alerts. Grouping by cluster shows "all nodes in us-east-1 have high CPU" as one notification.

**4. On-call during major outage (200 alerts in 2 minutes):**
- **Recommendation**: `group_by: ['alertname']` or `group_by: ['alertname', 'cluster']`
- Reasoning: During a cascading failure, you need maximum batching. Grouping by alertname alone means all 200 alerts collapse into a handful of notifications ("50 InstanceDown, 30 HighCPU, 20 HighMemory"). The on-call sees the pattern immediately instead of 200 individual messages.

### Task 8: The Grouping Paradox

The grouping paradox: aggressive grouping reduces noise but loses context; granular grouping preserves context but creates noise.

**Resolution strategies:**

1. **Use rich annotations instead of label-based context**: Group aggressively by `alertname` only, but include per-instance details in the notification body. The notification says "20 instances down" and lists them all.

2. **Tiered grouping**: Use different `group_by` for different severity levels. Critical alerts get finer grouping (more context for urgent issues). Warning alerts get aggressive grouping (less noise for non-urgent issues).

3. **Dynamic grouping based on alert volume**: When few alerts are firing, use fine grouping (per-instance). When many alerts are firing (indicating a cascading failure), switch to aggressive grouping (per-alertname). AlertManager does not natively support this, but you can implement it with routing rules that match on alert volume.

4. **Notification templates**: Use AlertManager templates to include the full list of affected instances in a single grouped notification:

```yaml
text: |
  {{ range .Alerts }}
  - {{ .Labels.instance }}: {{ .Annotations.summary }}
  {{ end }}
```

This gives you the noise reduction of grouping with the context of individual alerts.

## Common Mistakes

1. **Using `group_by: ['...']` in production**: This disables grouping entirely. Every alert gets its own notification. Only useful for debugging.

2. **Forgetting that `group_by` affects resolved notifications**: When a group resolves, the resolved notification also groups by the same labels. If you change `group_by`, in-flight groups may split or merge unexpectedly.

3. **Confusing `group_interval` with `repeat_interval`**: `group_interval` is the minimum time between notifications for a group that has new alerts. `repeat_interval` is the time before re-sending a notification for a group that is still firing but has no new alerts. They serve different purposes.

4. **Setting `group_wait` too low**: A `group_wait` of 0s means every alert triggers an immediate notification. Related alerts that fire 1-2 seconds apart get separate notifications instead of being batched.

5. **Not considering the receiver's perspective**: Grouping by `alertname` produces 1 notification, but the receiver (e.g., PagerDuty) creates 1 incident with 1 alert. If you need per-instance incidents in PagerDuty, you need finer grouping or separate routing rules.
