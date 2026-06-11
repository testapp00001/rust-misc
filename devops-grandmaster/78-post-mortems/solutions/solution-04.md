# Solution 04: Post-Mortem Metrics Dashboard

## Exercise Recap

Build a Python `PostMortemTracker` class that tracks post-mortem metrics
including MTTR, MTTD, severity breakdowns, root cause distribution, action
item tracking, and trend analysis.

## Complete Implementation

```python
"""
PostMortemTracker -- Metrics and analytics for incident post-mortems.

Tracks MTTR (Mean Time To Resolve), MTTD (Mean Time To Detect),
severity breakdowns, root cause distribution, action item completion,
and trend analysis across a portfolio of incidents.
"""

from dataclasses import dataclass, field
from datetime import datetime, timedelta
from enum import Enum
from typing import Optional
from collections import Counter
import statistics


class Severity(Enum):
    SEV1 = "SEV-1"
    SEV2 = "SEV-2"
    SEV3 = "SEV-3"
    SEV4 = "SEV-4"


class ActionItemStatus(Enum):
    NOT_STARTED = "not_started"
    IN_PROGRESS = "in_progress"
    DONE = "done"
    WONT_DO = "wont_do"


class RootCauseCategory(Enum):
    CONFIGURATION = "configuration"
    DEPENDENCY = "dependency"
    CAPACITY = "capacity"
    CODE_BUG = "code_bug"
    HUMAN_ERROR = "human_error"
    MONITORING = "monitoring"
    PROCESS = "process"
    INFRASTRUCTURE = "infrastructure"
    SECURITY = "security"
    UNKNOWN = "unknown"


@dataclass
class ActionItem:
    """A single action item from a post-mortem."""
    id: str
    title: str
    owner: str
    priority: str  # P0, P1, P2
    due_date: datetime
    created_date: datetime
    status: ActionItemStatus = ActionItemStatus.NOT_STARTED
    completed_date: Optional[datetime] = None
    incident_id: str = ""

    def is_overdue(self, as_of: Optional[datetime] = None) -> bool:
        """Check if this action item is overdue."""
        reference = as_of or datetime.now()
        if self.status == ActionItemStatus.DONE:
            return False
        return reference > self.due_date

    def days_to_completion(self) -> Optional[int]:
        """Days from creation to completion. None if not completed."""
        if self.completed_date is None:
            return None
        return (self.completed_date - self.created_date).days


@dataclass
class Incident:
    """A single incident record."""
    id: str
    title: str
    severity: Severity
    detected_at: datetime
    acknowledged_at: datetime
    mitigated_at: datetime
    resolved_at: datetime
    root_cause: RootCauseCategory
    root_cause_detail: str
    affected_services: list[str]
    action_items: list[ActionItem] = field(default_factory=list)
    created_by: str = ""
    tags: list[str] = field(default_factory=list)

    @property
    def mttc_minutes(self) -> float:
        """Mean Time To Contain: detected to mitigated."""
        delta = self.mitigated_at - self.detected_at
        return delta.total_seconds() / 60

    @property
    def mttf_minutes(self) -> float:
        """Mean Time To Fix: mitigated to fully resolved."""
        delta = self.resolved_at - self.mitigated_at
        return delta.total_seconds() / 60

    @property
    def total_duration_minutes(self) -> float:
        """Total incident duration: detected to resolved."""
        delta = self.resolved_at - self.detected_at
        return delta.total_seconds() / 60

    @property
    def detection_time_minutes(self) -> float:
        """Time from when the incident likely started to detection.
        Uses acknowledged_at as a proxy for when the problem began
        impacting users."""
        delta = self.detected_at - self.acknowledged_at
        return abs(delta.total_seconds() / 60)


class PostMortemTracker:
    """
    Tracks and analyzes post-mortem metrics across a portfolio of incidents.

    Usage:
        tracker = PostMortemTracker()
        tracker.add_incident(incident)
        report = tracker.generate_report()
    """

    def __init__(self) -> None:
        self._incidents: dict[str, Incident] = {}

    def add_incident(self, incident: Incident) -> None:
        """Register an incident for tracking."""
        self._incidents[incident.id] = incident

    def get_incident(self, incident_id: str) -> Optional[Incident]:
        """Retrieve a specific incident by ID."""
        return self._incidents.get(incident_id)

    def list_incidents(
        self,
        severity: Optional[Severity] = None,
        root_cause: Optional[RootCauseCategory] = None,
        since: Optional[datetime] = None,
        until: Optional[datetime] = None,
    ) -> list[Incident]:
        """List incidents with optional filters."""
        results = list(self._incidents.values())

        if severity:
            results = [i for i in results if i.severity == severity]
        if root_cause:
            results = [i for i in results if i.root_cause == root_cause]
        if since:
            results = [i for i in results if i.detected_at >= since]
        if until:
            results = [i for i in results if i.detected_at <= until]

        return sorted(results, key=lambda i: i.detected_at, reverse=True)

    # ------------------------------------------------------------------ #
    # Core Metrics
    # ------------------------------------------------------------------ #

    def mttr(self, severity: Optional[Severity] = None) -> Optional[float]:
        """
        Mean Time To Resolve (minutes).

        MTTR = mean(resolved_at - detected_at) across incidents.

        Args:
            severity: Optional filter to calculate MTTR for a
                      specific severity level.

        Returns:
            MTTR in minutes, or None if no matching incidents.
        """
        incidents = self.list_incidents(severity=severity)
        if not incidents:
            return None
        durations = [i.total_duration_minutes for i in incidents]
        return statistics.mean(durations)

    def mttd(self, severity: Optional[Severity] = None) -> Optional[float]:
        """
        Mean Time To Detect (minutes).

        MTTD measures how long it takes from when an incident begins
        impacting users to when it is detected by monitoring or
        escalation.

        In this implementation, we approximate MTTD as the time between
        when the incident was acknowledged (likely start of impact) and
        when it was formally detected and triaged.

        Args:
            severity: Optional filter for specific severity level.

        Returns:
            MTTD in minutes, or None if no matching incidents.
        """
        incidents = self.list_incidents(severity=severity)
        if not incidents:
            return None
        detection_times = [i.detection_time_minutes for i in incidents]
        return statistics.mean(detection_times)

    def mttc(self, severity: Optional[Severity] = None) -> Optional[float]:
        """
        Mean Time To Contain (minutes).

        MTTC = mean(mitigated_at - detected_at).

        This measures how quickly the team stops the bleeding, separate
        from how long it takes to fully resolve the underlying cause.

        Args:
            severity: Optional filter for specific severity level.

        Returns:
            MTTC in minutes, or None if no matching incidents.
        """
        incidents = self.list_incidents(severity=severity)
        if not incidents:
            return None
        contain_times = [i.mttc_minutes for i in incidents]
        return statistics.mean(contain_times)

    def mttf(self, severity: Optional[Severity] = None) -> Optional[float]:
        """
        Mean Time To Fix (minutes).

        MTTF = mean(resolved_at - mitigated_at).

        This measures how long it takes to fully resolve after the
        immediate impact is contained.

        Args:
            severity: Optional filter for specific severity level.

        Returns:
            MTTF in minutes, or None if no matching incidents.
        """
        incidents = self.list_incidents(severity=severity)
        if not incidents:
            return None
        fix_times = [i.mttf_minutes for i in incidents]
        return statistics.mean(fix_times)

    # ------------------------------------------------------------------ #
    # Distribution Metrics
    # ------------------------------------------------------------------ #

    def severity_breakdown(self) -> dict[str, int]:
        """
        Count of incidents per severity level.

        Returns:
            Dict mapping severity string to count.
        """
        counts: Counter[str] = Counter()
        for incident in self._incidents.values():
            counts[incident.severity.value] += 1
        return dict(sorted(counts.items()))

    def root_cause_distribution(self) -> dict[str, int]:
        """
        Count of incidents per root cause category.

        Returns:
            Dict mapping root cause category to count.
        """
        counts: Counter[str] = Counter()
        for incident in self._incidents.values():
            counts[incident.root_cause.value] += 1
        return dict(
            sorted(counts.items(), key=lambda x: x[1], reverse=True)
        )

    def affected_services_frequency(self) -> dict[str, int]:
        """
        Which services appear most frequently in incidents.

        Returns:
            Dict mapping service name to incident count.
        """
        counts: Counter[str] = Counter()
        for incident in self._incidents.values():
            for service in incident.affected_services:
                counts[service] += 1
        return dict(
            sorted(counts.items(), key=lambda x: x[1], reverse=True)
        )

    # ------------------------------------------------------------------ #
    # Action Item Metrics
    # ------------------------------------------------------------------ #

    def all_action_items(self) -> list[ActionItem]:
        """Collect all action items across all incidents."""
        items = []
        for incident in self._incidents.values():
            items.extend(incident.action_items)
        return items

    def action_item_completion_rate(self) -> Optional[float]:
        """
        Fraction of action items that are completed (done or wont_do).

        Returns:
            Completion rate as a float 0.0-1.0, or None if no items.
        """
        items = self.all_action_items()
        if not items:
            return None
        completed = sum(
            1 for i in items
            if i.status in (ActionItemStatus.DONE, ActionItemStatus.WONT_DO)
        )
        return completed / len(items)

    def overdue_action_items(self) -> list[ActionItem]:
        """List all action items that are past their due date."""
        return [
            item for item in self.all_action_items()
            if item.is_overdue()
        ]

    def action_items_by_priority(self) -> dict[str, dict[str, int]]:
        """
        Action item status breakdown by priority level.

        Returns:
            Dict mapping priority to status counts.
            Example: {"P0": {"done": 3, "in_progress": 1, ...}}
        """
        result: dict[str, dict[str, int]] = {}
        for item in self.all_action_items():
            if item.priority not in result:
                result[item.priority] = {}
            status_key = item.status.value
            result[item.priority][status_key] = (
                result[item.priority].get(status_key, 0) + 1
            )
        return result

    def avg_action_item_completion_days(self) -> Optional[float]:
        """
        Average days from creation to completion for completed items.

        Returns:
            Average days, or None if no completed items.
        """
        completed = [
            item for item in self.all_action_items()
            if item.status == ActionItemStatus.DONE
            and item.completed_date is not None
        ]
        if not completed:
            return None
        days = [item.days_to_completion() for item in completed]
        return statistics.mean(d for d in days if d is not None)

    # ------------------------------------------------------------------ #
    # Trend Analysis
    # ------------------------------------------------------------------ #

    def incidents_per_month(self) -> dict[str, int]:
        """
        Incident count grouped by month (YYYY-MM).

        Returns:
            Dict mapping month string to incident count.
        """
        counts: Counter[str] = Counter()
        for incident in self._incidents.values():
            month_key = incident.detected_at.strftime("%Y-%m")
            counts[month_key] += 1
        return dict(sorted(counts.items()))

    def mttr_trend(self, months: int = 6) -> list[dict[str, object]]:
        """
        MTTR per month for the last N months.

        Args:
            months: Number of recent months to include.

        Returns:
            List of dicts with 'month', 'mttr_minutes', and 'count'.
        """
        now = datetime.now()
        cutoff = now - timedelta(days=months * 30)
        recent = [
            i for i in self._incidents.values()
            if i.detected_at >= cutoff
        ]

        monthly: dict[str, list[float]] = {}
        for incident in recent:
            key = incident.detected_at.strftime("%Y-%m")
            if key not in monthly:
                monthly[key] = []
            monthly[key].append(incident.total_duration_minutes)

        trend = []
        for month in sorted(monthly.keys()):
            durations = monthly[month]
            trend.append({
                "month": month,
                "mttr_minutes": round(statistics.mean(durations), 1),
                "count": len(durations),
            })
        return trend

    def severity_trend(self, months: int = 6) -> list[dict[str, object]]:
        """
        Severity breakdown per month for the last N months.

        Args:
            months: Number of recent months to include.

        Returns:
            List of dicts with 'month' and severity counts.
        """
        now = datetime.now()
        cutoff = now - timedelta(days=months * 30)
        recent = [
            i for i in self._incidents.values()
            if i.detected_at >= cutoff
        ]

        monthly: dict[str, Counter[str]] = {}
        for incident in recent:
            key = incident.detected_at.strftime("%Y-%m")
            if key not in monthly:
                monthly[key] = Counter()
            monthly[key][incident.severity.value] += 1

        trend = []
        for month in sorted(monthly.keys()):
            entry: dict[str, object] = {"month": month}
            for sev in Severity:
                entry[sev.value] = monthly[month].get(sev.value, 0)
            trend.append(entry)
        return trend

    # ------------------------------------------------------------------ #
    # Full Report
    # ------------------------------------------------------------------ #

    def generate_report(self) -> str:
        """
        Generate a formatted text report of all metrics.

        Returns:
            Multi-line string report.
        """
        lines = []
        lines.append("=" * 60)
        lines.append("  POST-MORTEM METRICS REPORT")
        lines.append("=" * 60)
        lines.append("")

        # Overview
        total = len(self._incidents)
        lines.append(f"Total Incidents Tracked: {total}")
        lines.append("")

        # MTTR / MTTD / MTTC / MTTF
        lines.append("--- Response Time Metrics ---")
        overall_mttr = self.mttr()
        overall_mttd = self.mttd()
        overall_mttc = self.mttc()
        overall_mttf = self.mttf()

        lines.append(
            f"  MTTR (Mean Time To Resolve):  "
            f"{overall_mttr:.1f} min" if overall_mttr else
            "  MTTR: N/A"
        )
        lines.append(
            f"  MTTD (Mean Time To Detect):   "
            f"{overall_mttd:.1f} min" if overall_mttd else
            "  MTTD: N/A"
        )
        lines.append(
            f"  MTTC (Mean Time To Contain):  "
            f"{overall_mttc:.1f} min" if overall_mttc else
            "  MTTC: N/A"
        )
        lines.append(
            f"  MTTF (Mean Time To Fix):      "
            f"{overall_mttf:.1f} min" if overall_mttf else
            "  MTTF: N/A"
        )
        lines.append("")

        # MTTR by Severity
        lines.append("--- MTTR by Severity ---")
        for sev in Severity:
            mttr_sev = self.mttr(severity=sev)
            if mttr_sev is not None:
                lines.append(f"  {sev.value}: {mttr_sev:.1f} min")
        lines.append("")

        # Severity Breakdown
        lines.append("--- Severity Breakdown ---")
        for sev, count in self.severity_breakdown().items():
            pct = (count / total * 100) if total > 0 else 0
            bar = "#" * int(pct / 2)
            lines.append(f"  {sev}: {count:3d} ({pct:5.1f}%) {bar}")
        lines.append("")

        # Root Cause Distribution
        lines.append("--- Root Cause Distribution ---")
        for cause, count in self.root_cause_distribution().items():
            pct = (count / total * 100) if total > 0 else 0
            bar = "#" * int(pct / 2)
            lines.append(f"  {cause:20s}: {count:3d} ({pct:5.1f}%) {bar}")
        lines.append("")

        # Top Affected Services
        lines.append("--- Most Affected Services ---")
        for svc, count in self.affected_services_frequency().items()[:5]:
            lines.append(f"  {svc}: {count} incidents")
        lines.append("")

        # Action Items
        all_items = self.all_action_items()
        lines.append("--- Action Items ---")
        lines.append(f"  Total: {len(all_items)}")
        completion = self.action_item_completion_rate()
        if completion is not None:
            lines.append(f"  Completion Rate: {completion:.1%}")
        overdue = self.overdue_action_items()
        lines.append(f"  Overdue: {len(overdue)}")

        priority_breakdown = self.action_items_by_priority()
        for priority in sorted(priority_breakdown.keys()):
            statuses = priority_breakdown[priority]
            lines.append(f"  {priority}: {statuses}")

        avg_days = self.avg_action_item_completion_days()
        if avg_days is not None:
            lines.append(
                f"  Avg Completion Time: {avg_days:.1f} days"
            )
        lines.append("")

        # Trends
        lines.append("--- Monthly Trend (Last 6 Months) ---")
        trend = self.mttr_trend(months=6)
        if trend:
            lines.append(f"  {'Month':10s} {'Count':>6s} {'MTTR(min)':>10s}")
            lines.append(f"  {'-'*10} {'-'*6} {'-'*10}")
            for entry in trend:
                lines.append(
                    f"  {entry['month']:10s} "
                    f"{entry['count']:>6d} "
                    f"{entry['mttr_minutes']:>10.1f}"
                )
        else:
            lines.append("  No data for the selected period.")
        lines.append("")
        lines.append("=" * 60)

        return "\n".join(lines)


# ------------------------------------------------------------------ #
# Example Usage
# ------------------------------------------------------------------ #

def demo() -> None:
    """Demonstrate the PostMortemTracker with sample data."""

    tracker = PostMortemTracker()

    # Incident 1: Database failover issue
    incident_1 = Incident(
        id="INC-2025-001",
        title="Payment service outage during Black Friday",
        severity=Severity.SEV1,
        detected_at=datetime(2025, 11, 28, 14, 22),
        acknowledged_at=datetime(2025, 11, 28, 14, 12),
        mitigated_at=datetime(2025, 11, 28, 14, 30),
        resolved_at=datetime(2025, 11, 28, 14, 35),
        root_cause=RootCauseCategory.DEPENDENCY,
        root_cause_detail="Fraud detection service latency caused "
                          "connection pool exhaustion in payment service",
        affected_services=[
            "payment-service", "checkout-api", "order-service"
        ],
        action_items=[
            ActionItem(
                id="AI-001",
                title="Implement circuit breaker for fraud detection",
                owner="alex.chen",
                priority="P0",
                due_date=datetime(2025, 12, 15),
                created_date=datetime(2025, 12, 2),
                status=ActionItemStatus.DONE,
                completed_date=datetime(2025, 12, 12),
                incident_id="INC-2025-001",
            ),
            ActionItem(
                id="AI-002",
                title="Add connection pool monitoring and alerts",
                owner="maria.gonzalez",
                priority="P0",
                due_date=datetime(2025, 12, 10),
                created_date=datetime(2025, 12, 2),
                status=ActionItemStatus.DONE,
                completed_date=datetime(2025, 12, 9),
                incident_id="INC-2025-001",
            ),
            ActionItem(
                id="AI-003",
                title="Implement batch job scheduling policy",
                owner="priya.kapoor",
                priority="P1",
                due_date=datetime(2025, 12, 20),
                created_date=datetime(2025, 12, 2),
                status=ActionItemStatus.IN_PROGRESS,
                incident_id="INC-2025-001",
            ),
        ],
    )

    # Incident 2: Config drift
    incident_2 = Incident(
        id="INC-2025-002",
        title="Auth service failure due to Redis failover misconfiguration",
        severity=Severity.SEV1,
        detected_at=datetime(2025, 10, 15, 9, 10),
        acknowledged_at=datetime(2025, 10, 15, 9, 0),
        mitigated_at=datetime(2025, 10, 15, 9, 30),
        resolved_at=datetime(2025, 10, 15, 9, 47),
        root_cause=RootCauseCategory.CONFIGURATION,
        root_cause_detail="Redis replica security group rules were "
                          "inconsistent with primary due to separate "
                          "CloudFormation stacks",
        affected_services=["auth-service", "session-service"],
        action_items=[
            ActionItem(
                id="AI-004",
                title="Standardize Redis provisioning with IaC modules",
                owner="sam.okafor",
                priority="P1",
                due_date=datetime(2025, 11, 15),
                created_date=datetime(2025, 10, 20),
                status=ActionItemStatus.DONE,
                completed_date=datetime(2025, 11, 10),
                incident_id="INC-2025-002",
            ),
            ActionItem(
                id="AI-005",
                title="Add config drift detection for stateful services",
                owner="sam.okafor",
                priority="P1",
                due_date=datetime(2025, 12, 1),
                created_date=datetime(2025, 10, 20),
                status=ActionItemStatus.OVERDUE
                if datetime.now() > datetime(2025, 12, 1)
                else ActionItemStatus.IN_PROGRESS,
                incident_id="INC-2025-002",
            ),
        ],
    )

    # Incident 3: Code bug
    incident_3 = Incident(
        id="INC-2025-003",
        title="Search service returning incorrect results after deploy",
        severity=Severity.SEV2,
        detected_at=datetime(2025, 9, 5, 16, 5),
        acknowledged_at=datetime(2025, 9, 5, 15, 45),
        mitigated_at=datetime(2025, 9, 5, 16, 20),
        resolved_at=datetime(2025, 9, 5, 17, 0),
        root_cause=RootCauseCategory.CODE_BUG,
        root_cause_detail="Off-by-one error in pagination logic caused "
                          "results to skip every other page",
        affected_services=["search-service", "catalog-api"],
        action_items=[
            ActionItem(
                id="AI-006",
                title="Add pagination integration tests",
                owner="chen.wei",
                priority="P1",
                due_date=datetime(2025, 10, 1),
                created_date=datetime(2025, 9, 10),
                status=ActionItemStatus.DONE,
                completed_date=datetime(2025, 9, 28),
                incident_id="INC-2025-003",
            ),
        ],
    )

    # Incident 4: Capacity issue
    incident_4 = Incident(
        id="INC-2025-004",
        title="API gateway rate limiting during product launch",
        severity=Severity.SEV2,
        detected_at=datetime(2025, 8, 20, 10, 5),
        acknowledged_at=datetime(2025, 8, 20, 10, 0),
        mitigated_at=datetime(2025, 8, 20, 10, 25),
        resolved_at=datetime(2025, 8, 20, 10, 45),
        root_cause=RootCauseCategory.CAPACITY,
        root_cause_detail="API gateway rate limit was not pre-scaled "
                          "for expected product launch traffic",
        affected_services=["api-gateway", "product-service"],
        action_items=[
            ActionItem(
                id="AI-007",
                title="Create launch capacity planning checklist",
                owner="jordan.lee",
                priority="P2",
                due_date=datetime(2025, 9, 30),
                created_date=datetime(2025, 8, 25),
                status=ActionItemStatus.DONE,
                completed_date=datetime(2025, 9, 15),
                incident_id="INC-2025-004",
            ),
        ],
    )

    # Incident 5: Monitoring gap
    incident_5 = Incident(
        id="INC-2025-005",
        title="Disk space exhaustion on log aggregation service",
        severity=Severity.SEV3,
        detected_at=datetime(2025, 7, 12, 3, 30),
        acknowledged_at=datetime(2025, 7, 12, 2, 0),
        mitigated_at=datetime(2025, 7, 12, 4, 0),
        resolved_at=datetime(2025, 7, 12, 4, 15),
        root_cause=RootCauseCategory.MONITORING,
        root_cause_detail="No disk space alerting configured for the "
                          "log aggregation service nodes",
        affected_services=["log-aggregator", "elasticsearch"],
        action_items=[
            ActionItem(
                id="AI-008",
                title="Add disk space alerts for all stateful services",
                owner="ops-team",
                priority="P0",
                due_date=datetime(2025, 7, 20),
                created_date=datetime(2025, 7, 15),
                status=ActionItemStatus.DONE,
                completed_date=datetime(2025, 7, 18),
                incident_id="INC-2025-005",
            ),
        ],
    )

    tracker.add_incident(incident_1)
    tracker.add_incident(incident_2)
    tracker.add_incident(incident_3)
    tracker.add_incident(incident_4)
    tracker.add_incident(incident_5)

    # Print the report
    print(tracker.generate_report())

    # Programmatic access to metrics
    print("\n--- Programmatic Access Examples ---")
    print(f"Overall MTTR: {tracker.mttr():.1f} minutes")
    print(f"SEV-1 MTTR: {tracker.mttr(Severity.SEV1):.1f} minutes")
    print(f"Action Items: {len(tracker.all_action_items())}")
    print(f"Completion Rate: {tracker.action_item_completion_rate():.1%}")
    print(f"Overdue Items: {len(tracker.overdue_action_items())}")


if __name__ == "__main__":
    demo()
```

## Why This Design Works

### Data Model Separation

The `Incident`, `ActionItem`, `Severity`, and `RootCauseCategory` are
separate classes. This allows each to be validated, serialized, and tested
independently. The `PostMortemTracker` operates on these domain objects
rather than raw dictionaries, providing type safety and IDE support.

### Computed Properties on Incident

Metrics like `mttc_minutes`, `mttf_minutes`, and `total_duration_minutes`
are computed as properties on the `Incident` class. This keeps the
calculation logic close to the data and makes the tracker methods cleaner.

```
Timeline of an Incident:

  |<-- MTTD -->|<-- MTTC -->|<-- MTTF -->|
  |            |            |            |
  Start     Detected    Mitigated    Resolved
  (unknown)  acknowledged  mitigated    resolved

  Total Duration (MTTR) = Detected -> Resolved
  MTTC = Detected -> Mitigated
  MTTF = Mitigated -> Resolved
```

### Filterable List Method

The `list_incidents` method supports optional filters for severity, root
cause, and date range. All other methods use this as their foundation,
ensuring consistent filtering behavior.

### Metric Methods Return Optional

Methods like `mttr()`, `mttd()`, and `action_item_completion_rate()` return
`Optional[float]` -- they return `None` when no data is available. This
prevents division-by-zero errors and makes callers explicitly handle the
"no data" case.

### Trend Analysis

The `mttr_trend` and `severity_trend` methods group incidents by month,
enabling teams to see whether their metrics are improving over time.

```
MTTR Trend Visualization (conceptual):

  120 |  *
  100 |  *
   80 |     *
   60 |        *
   40 |           *  *
   20 |                 *
    0 +--+--+--+--+--+--+---> Month
      Jul Aug Sep Oct Nov Dec

  Trend: MTTR is decreasing -- team is getting better at response.
```

### Full Report Generation

The `generate_report` method produces a human-readable text report with
bar charts and formatted tables. This is suitable for inclusion in team
dashboards, Slack messages, or email summaries.

## Common Mistakes to Avoid

1. **Using wall-clock time without timezone awareness.** Always store
   and compare datetimes with timezone information. UTC is the standard
   for incident timestamps.

2. **Not filtering "wont_do" from completion rate.** Action items marked
   "wont_do" are intentionally not completed and should be counted as
   resolved, not as incomplete.

3. **Calculating MTTR without severity stratification.** Overall MTTR
   can be misleading if SEV-1 incidents are fast but SEV-3 incidents
   drag out. Always provide severity-level breakdowns.

4. **Not handling empty data sets.** If there are no incidents or no
   action items, metric calculations must not crash with division by
   zero. Return `None` or an appropriate default.

5. **Mixing detection and acknowledgment times.** MTTD should measure
   the time from when impact begins to when it is detected. Be clear
   about which timestamp represents "impact start" versus "detection."
