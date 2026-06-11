# Exercise 04: Post-Mortem Metrics Dashboard

**Type:** Challenge | **Time:** 45 min | **Difficulty:** Medium-Hard

## Objective

Build a Python class that tracks post-mortem data and calculates key metrics
including MTTR (Mean Time to Resolve), MTTD (Mean Time to Detect), root cause
distribution, action item completion rates, and generates trend reports. This
exercise teaches you to quantify the effectiveness of your post-mortem process
and identify patterns over time.

## Background

Post-mortems are only valuable if they drive improvement. To measure whether
your post-mortem process is actually improving your organization's reliability,
you need metrics. Key metrics include:

- **MTTD (Mean Time to Detect):** How long between an incident starting and
  the team becoming aware. Lower is better.
- **MTTR (Mean Time to Resolve):** How long between detection and resolution.
  Lower is better.
- **Root Cause Distribution:** Categorizing root causes to identify systemic
  patterns. Are most incidents caused by deployments? Dependencies? Config?
- **Action Item Completion Rate:** What percentage of post-mortem action items
  are completed on time? Low rates indicate the post-mortem process is not
  driving real change.
- **Incident Frequency:** Are incidents increasing or decreasing over time?
- **Severity Distribution:** Are we preventing high-severity incidents?

## The Exercise

### Part A: Implement the PostMortemTracker Class

Create a Python file called `postmortem_tracker.py` with the following class.
Implement every method according to the specifications.

```python
from dataclasses import dataclass, field
from datetime import datetime, timedelta
from enum import Enum
from typing import Optional
import json


class Severity(Enum):
    SEV1 = "SEV-1"
    SEV2 = "SEV-2"
    SEV3 = "SEV-3"
    SEV4 = "SEV-4"


class ActionItemStatus(Enum):
    OPEN = "open"
    IN_PROGRESS = "in_progress"
    COMPLETED = "completed"
    WONT_FIX = "wont_fix"


@dataclass
class ActionItem:
    id: str
    title: str
    owner: str
    created_date: datetime
    due_date: datetime
    status: ActionItemStatus = ActionItemStatus.OPEN
    completed_date: Optional[datetime] = None
    category: str = ""  # e.g., "automation", "monitoring", "process", "documentation"


@dataclass
class PostMortem:
    incident_id: str
    title: str
    severity: Severity
    detected_at: datetime
    started_at: datetime
    resolved_at: datetime
    root_cause_category: str  # e.g., "deployment", "dependency", "config", "capacity"
    root_cause_description: str
    action_items: list[ActionItem] = field(default_factory=list)
    services_affected: list[str] = field(default_factory=list)
    customers_affected: int = 0
    author: str = ""
    blameless_score: Optional[float] = None  # 0-10, from language review


class PostMortemTracker:
    """Tracks post-mortem incidents and calculates reliability metrics."""

    def __init__(self):
        self.post_mortems: list[PostMortem] = []

    def add_post_mortem(self, pm: PostMortem) -> None:
        """Add a post-mortem to the tracker."""
        # IMPLEMENT THIS
        pass

    def get_mttd(self, start_date: Optional[datetime] = None,
                 end_date: Optional[datetime] = None,
                 severity: Optional[Severity] = None) -> timedelta:
        """Calculate Mean Time to Detect.

        MTTD = average of (detected_at - started_at) for all incidents
        in the given date range and severity filter.

        Args:
            start_date: Only include incidents after this date (inclusive)
            end_date: Only include incidents before this date (inclusive)
            severity: Only include incidents of this severity

        Returns:
            Average timedelta from incident start to detection

        Raises:
            ValueError: If no incidents match the filter criteria
        """
        # IMPLEMENT THIS
        pass

    def get_mttr(self, start_date: Optional[datetime] = None,
                 end_date: Optional[datetime] = None,
                 severity: Optional[Severity] = None) -> timedelta:
        """Calculate Mean Time to Resolve.

        MTTR = average of (resolved_at - detected_at) for all incidents
        in the given date range and severity filter.

        Note: MTTR measures from detection to resolution, NOT from
        incident start to resolution. This is intentional -- it measures
        the team's response effectiveness, not the incident's total duration.

        Args:
            start_date: Only include incidents after this date (inclusive)
            end_date: Only include incidents before this date (inclusive)
            severity: Only include incidents of this severity

        Returns:
            Average timedelta from detection to resolution

        Raises:
            ValueError: If no incidents match the filter criteria
        """
        # IMPLEMENT THIS
        pass

    def get_incident_frequency(self, interval: str = "monthly") -> dict[str, int]:
        """Count incidents per time interval.

        Args:
            interval: One of "daily", "weekly", "monthly", "yearly"

        Returns:
            Dict mapping interval label (e.g., "2026-03") to count

        Raises:
            ValueError: If interval is not a valid option
        """
        # IMPLEMENT THIS
        pass

    def get_severity_distribution(self) -> dict[str, int]:
        """Count incidents by severity level.

        Returns:
            Dict mapping severity string (e.g., "SEV-1") to count
        """
        # IMPLEMENT THIS
        pass

    def get_root_cause_distribution(self) -> dict[str, int]:
        """Count incidents by root cause category.

        Returns:
            Dict mapping root_cause_category to count
        """
        # IMPLEMENT THIS
        pass

    def get_action_item_completion_rate(self,
                                        start_date: Optional[datetime] = None,
                                        end_date: Optional[datetime] = None) -> dict[str, float]:
        """Calculate action item completion statistics.

        Returns a dict with:
            - "total": total number of action items
            - "completed": number of completed action items
            - "on_time": number completed before or on due date
            - "overdue": number still open/in_progress past due date
            - "completion_rate": completed / total (0.0 to 1.0)
            - "on_time_rate": on_time / completed (0.0 to 1.0, or 0 if no completed)

        Args:
            start_date: Only include action items from incidents after this date
            end_date: Only include action items from incidents before this date
        """
        # IMPLEMENT THIS
        pass

    def get_most_affected_services(self, top_n: int = 5) -> list[tuple[str, int]]:
        """Find the services most frequently affected by incidents.

        Args:
            top_n: Number of top services to return

        Returns:
            List of (service_name, incident_count) tuples, sorted by count descending
        """
        # IMPLEMENT THIS
        pass

    def get_blameless_score_trend(self) -> list[tuple[str, float]]:
        """Track blameless language scores over time.

        Returns:
            List of (month_label, average_score) tuples, sorted chronologically.
            Only includes months that have incidents with a blameless_score set.
        """
        # IMPLEMENT THIS
        pass

    def generate_trend_report(self, lookback_months: int = 6) -> str:
        """Generate a human-readable trend report.

        The report should include:
        1. Summary statistics (total incidents, average MTTR/MTTD)
        2. Severity distribution
        3. Root cause distribution
        4. Action item completion rate
        5. Top affected services
        6. MTTR trend (month over month)
        7. MTTD trend (month over month)
        8. Recommendations based on the data

        Args:
            lookback_months: How many months of data to include

        Returns:
            Formatted string report
        """
        # IMPLEMENT THIS
        pass

    def export_to_json(self, filepath: str) -> None:
        """Export all post-mortem data to a JSON file.

        The JSON should be a list of post-mortem objects, each containing
        all fields including nested action items.

        Datetime fields should be formatted as ISO 8601 strings.

        Args:
            filepath: Path to write the JSON file
        """
        # IMPLEMENT THIS
        pass
```

### Part B: Write Tests

Create a file called `test_postmortem_tracker.py` with tests for every method.
Your tests should cover:

1. **Normal cases** with realistic data
2. **Edge cases** (empty tracker, single incident, incidents with no action items)
3. **Filter cases** (date range filtering, severity filtering)
4. **Boundary cases** (action items completed exactly on due date)

Use this test data seed as a starting point:

```python
import pytest
from datetime import datetime, timedelta
from postmortem_tracker import (
    PostMortemTracker, PostMortem, ActionItem,
    Severity, ActionItemStatus
)


@pytest.fixture
def sample_tracker():
    """Create a tracker with sample data spanning 3 months."""
    tracker = PostMortemTracker()

    # January incident
    pm1 = PostMortem(
        incident_id="INC-001",
        title="Database connection pool exhaustion",
        severity=Severity.SEV1,
        started_at=datetime(2026, 1, 10, 8, 0),
        detected_at=datetime(2026, 1, 10, 8, 15),
        resolved_at=datetime(2026, 1, 10, 9, 30),
        root_cause_category="capacity",
        root_cause_description="Connection pool size was not increased after traffic growth",
        services_affected=["api-gateway", "user-service", "order-service"],
        customers_affected=5000,
        author="sre-team",
        blameless_score=7.5,
        action_items=[
            ActionItem(
                id="AI-001",
                title="Implement connection pool auto-scaling",
                owner="platform-team",
                created_date=datetime(2026, 1, 11),
                due_date=datetime(2026, 2, 1),
                status=ActionItemStatus.COMPLETED,
                completed_date=datetime(2026, 1, 28),
                category="automation",
            ),
            ActionItem(
                id="AI-002",
                title="Add connection pool utilization alerts",
                owner="sre-team",
                created_date=datetime(2026, 1, 11),
                due_date=datetime(2026, 1, 25),
                status=ActionItemStatus.COMPLETED,
                completed_date=datetime(2026, 1, 20),
                category="monitoring",
            ),
        ],
    )

    # February incident
    pm2 = PostMortem(
        incident_id="INC-002",
        title="Bad deployment caused API 500s",
        severity=Severity.SEV2,
        started_at=datetime(2026, 2, 5, 14, 0),
        detected_at=datetime(2026, 2, 5, 14, 5),
        resolved_at=datetime(2026, 2, 5, 14, 45),
        root_cause_category="deployment",
        root_cause_description="Missing integration test for new API endpoint",
        services_affected=["api-gateway"],
        customers_affected=1200,
        author="backend-team",
        blameless_score=8.0,
        action_items=[
            ActionItem(
                id="AI-003",
                title="Add integration test suite to CI pipeline",
                owner="backend-team",
                created_date=datetime(2026, 2, 6),
                due_date=datetime(2026, 3, 1),
                status=ActionItemStatus.IN_PROGRESS,
                category="automation",
            ),
            ActionItem(
                id="AI-004",
                title="Document deployment validation checklist",
                owner="sre-team",
                created_date=datetime(2026, 2, 6),
                due_date=datetime(2026, 2, 20),
                status=ActionItemStatus.COMPLETED,
                completed_date=datetime(2026, 2, 18),
                category="documentation",
            ),
        ],
    )

    # March incident
    pm3 = PostMortem(
        incident_id="INC-003",
        title="DNS provider outage affected all services",
        severity=Severity.SEV1,
        started_at=datetime(2026, 3, 1, 10, 0),
        detected_at=datetime(2026, 3, 1, 10, 3),
        resolved_at=datetime(2026, 3, 1, 12, 0),
        root_cause_category="dependency",
        root_cause_description="Single DNS provider with no fallback",
        services_affected=["all"],
        customers_affected=50000,
        author="infra-team",
        blameless_score=9.0,
        action_items=[
            ActionItem(
                id="AI-005",
                title="Implement multi-DNS provider failover",
                owner="infra-team",
                created_date=datetime(2026, 3, 2),
                due_date=datetime(2026, 4, 1),
                status=ActionItemStatus.OPEN,
                category="architecture",
            ),
        ],
    )

    tracker.add_post_mortem(pm1)
    tracker.add_post_mortem(pm2)
    tracker.add_post_mortem(pm3)
    return tracker
```

Write at least 10 test cases covering all methods.

### Part C: Trend Report Analysis

After implementing the tracker, use the sample data to answer these questions:

1. What is the MTTR across all incidents? Is this acceptable for your organization?
2. What is the most common root cause category? What does this suggest about
   where to invest engineering effort?
3. What is the action item completion rate? Is the team following through on
   post-mortem commitments?
4. If you had to prioritize one investment based on this data, what would it be?

<details>
<summary>Hint: Expected MTTR calculation</summary>
Incident 1: 9:30 - 8:15 = 1h 15min = 75 min
Incident 2: 14:45 - 14:05 = 40 min
Incident 3: 12:00 - 10:03 = 1h 57min = 117 min
Average MTTR = (75 + 40 + 117) / 3 = 77.33 min
</details>

<details>
<summary>Hint: Expected MTTD calculation</summary>
Incident 1: 8:15 - 8:00 = 15 min
Incident 2: 14:05 - 14:00 = 5 min
Incident 3: 10:03 - 10:00 = 3 min
Average MTTD = (15 + 5 + 3) / 3 = 7.67 min
</details>

---

## Reflection Questions

1. **Metric selection:** Why did we track MTTD and MTTR separately instead of
   just total incident duration? What different insights do they provide?

2. **Action item tracking:** Why is the on-time completion rate a better metric
   than just the completion rate? What does a low on-time rate indicate about
   the organization?

3. **Root cause distribution:** If 60% of incidents have "deployment" as the
   root cause, what does this tell you? What investment would you recommend?

4. **Blameless score:** How would you operationalize a "blameless score"? Who
   would rate it? How would you ensure consistency across raters?

## Self-Assessment

Rate yourself after completing this exercise:

- [ ] I can implement a tracker that stores and retrieves post-mortem data
- [ ] I can calculate MTTR and MTTD correctly with filtering support
- [ ] I can generate distribution metrics (severity, root cause, services)
- [ ] I can calculate action item completion rates including on-time rates
- [ ] I can generate a human-readable trend report from the data
- [ ] I can write comprehensive tests covering normal, edge, and boundary cases

## Next Steps

Proceed to [Exercise 05: Continuous Improvement Pipeline](exercise-05.md) to
design a system that connects incident response, runbooks, and post-mortems
into a continuous improvement loop.
