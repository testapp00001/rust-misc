# Exercise 05: Continuous Improvement Pipeline

**Type:** Integration | **Time:** 45 min | **Difficulty:** Hard

## Objective

Design and implement a system that connects incident response (Module 76),
runbooks (Module 77), and post-mortems (Module 78) into a continuous
improvement loop. This exercise tests your ability to think architecturally
about how reliability practices reinforce each other and create a feedback
cycle that drives ongoing improvement.

## Background

The three modules in this section form a natural cycle:

```
Incident Response (Module 76)
        |
        | Incident occurs and is handled
        v
Post-Mortem (Module 78)
        |
        | Root cause analysis produces action items
        v
Action Items & Runbooks (Module 77)
        |
        | Improved runbooks and automation
        v
Incident Response (Module 76) -- better prepared for next incident
```

In practice, this cycle often breaks down because:

1. **Action items are forgotten** -- tracked in a spreadsheet nobody updates
2. **Runbooks are not updated** -- post-mortem findings don't flow into runbooks
3. **No feedback loop** -- teams don't know if their improvements actually worked
4. **Siloed tools** -- incident management, runbook, and post-mortem tools don't
   communicate with each other
5. **No measurement** -- nobody tracks whether the cycle is working

This exercise asks you to design and partially implement a system that
solves these problems.

## The Exercise

### Part A: System Architecture Design

Draw (or describe in text) the architecture of a Continuous Improvement
Pipeline that connects:

1. **Incident Management System** (receives alerts, manages incident lifecycle)
2. **Runbook Repository** (stores and serves operational procedures)
3. **Post-Mortem System** (stores post-mortem documents and action items)
4. **Metrics Dashboard** (tracks MTTR, MTTD, action item rates -- from Exercise 04)
5. **Notification System** (reminds owners about overdue action items)
6. **Knowledge Base** (stores lessons learned for on-call engineers)

For each component, specify:
- What data it produces
- What data it consumes
- What APIs or interfaces it exposes
- How it connects to the other components

<details>
<summary>Hint: Data Flow</summary>
Think about what data flows between components:
- Incident -> Post-Mortem: incident timeline, severity, affected services
- Post-Mortem -> Runbook: new procedures, updated troubleshooting steps
- Post-Mortem -> Action Items: tracked items with owners and deadlines
- Action Items -> Notification: overdue alerts, completion confirmations
- Metrics -> Dashboard: aggregated trends over time
- Knowledge Base <- Lessons learned from each post-mortem
- Runbook -> Incident Response: next time, better procedures available
```

Write your architecture description here:

```
[Your architecture description]
```

---

### Part B: Implement the FeedbackLoop Class

Create a Python file called `feedback_loop.py` that implements the continuous
improvement pipeline. The class should orchestrate the connections between
the three reliability practices.

```python
from dataclasses import dataclass, field
from datetime import datetime, timedelta
from enum import Enum
from typing import Optional, Callable
import json


# --- Enums ---

class IncidentPhase(Enum):
    DETECTED = "detected"
    RESPONDING = "responding"
    MITIGATED = "mitigated"
    RESOLVED = "resolved"
    POST_MORTEM_PENDING = "post_mortem_pending"
    POST_MORTEM_COMPLETE = "post_mortem_complete"


class ActionItemStatus(Enum):
    OPEN = "open"
    IN_PROGRESS = "in_progress"
    COMPLETED = "completed"
    WONT_FIX = "wont_fix"


class RunbookUpdateType(Enum):
    NEW_RUNBOOK = "new_runbook"
    UPDATED_RUNBOOK = "updated_runbook"
    NEW_TROUBLESHOOTING_STEP = "new_troubleshooting_step"
    DEPRECATED_RUNBOOK = "deprecated_runbook"


# --- Data Classes ---

@dataclass
class Incident:
    id: str
    title: str
    severity: str
    started_at: datetime
    phase: IncidentPhase = IncidentPhase.DETECTED
    detected_at: Optional[datetime] = None
    mitigated_at: Optional[datetime] = None
    resolved_at: Optional[datetime] = None
    services_affected: list[str] = field(default_factory=list)
    runbooks_used: list[str] = field(default_factory=list)
    post_mortem_id: Optional[str] = None
    resolution_notes: str = ""


@dataclass
class RunbookEntry:
    id: str
    title: str
    content: str
    service: str
    last_updated: datetime
    version: int = 1
    trigger_conditions: list[str] = field(default_factory=list)
    times_used: int = 0
    success_rate: float = 0.0  # percentage of times it led to resolution
    related_post_mortems: list[str] = field(default_factory=list)


@dataclass
class ActionItem:
    id: str
    title: str
    owner: str
    post_mortem_id: str
    created_date: datetime
    due_date: datetime
    status: ActionItemStatus = ActionItemStatus.OPEN
    category: str = ""
    runbook_update_required: bool = False
    runbook_update_done: bool = False
    completed_date: Optional[datetime] = None


@dataclass
class FeedbackEvent:
    """Represents a connection point in the improvement cycle."""
    timestamp: datetime
    event_type: str  # e.g., "action_item_created", "runbook_updated", "incident_improved"
    source: str  # which component generated this event
    details: dict = field(default_factory=dict)


# --- Main Class ---

class FeedbackLoop:
    """Orchestrates the continuous improvement cycle between
    incident response, runbooks, and post-mortems."""

    def __init__(self):
        self.incidents: dict[str, Incident] = {}
        self.runbooks: dict[str, RunbookEntry] = {}
        self.action_items: dict[str, ActionItem] = {}
        self.events: list[FeedbackEvent] = []
        self.notification_callbacks: list[Callable] = []

    # --- Incident Lifecycle ---

    def register_incident(self, incident: Incident) -> None:
        """Register a new incident in the system.

        Automatically logs a FeedbackEvent and transitions the incident
        to DETECTED phase.
        """
        # IMPLEMENT THIS
        pass

    def transition_incident(self, incident_id: str,
                            new_phase: IncidentPhase,
                            timestamp: Optional[datetime] = None) -> None:
        """Transition an incident to a new phase.

        Validates that the transition is legal:
        - DETECTED -> RESPONDING
        - RESPONDING -> MITIGATED
        - MITIGATED -> RESOLVED
        - RESOLVED -> POST_MORTEM_PENDING
        - POST_MORTEM_PENDING -> POST_MORTEM_COMPLETE

        Logs a FeedbackEvent for each transition.

        Raises:
            ValueError: If the incident ID is not found
            ValueError: If the transition is not valid
        """
        # IMPLEMENT THIS
        pass

    def record_runbook_usage(self, incident_id: str,
                             runbook_id: str,
                             led_to_resolution: bool) -> None:
        """Record that a runbook was used during an incident.

        Updates:
        - The incident's runbooks_used list
        - The runbook's times_used counter
        - The runbook's success_rate (running average)
        - Logs a FeedbackEvent
        """
        # IMPLEMENT THIS
        pass

    # --- Post-Mortem Integration ---

    def create_post_mortem_action_items(
        self,
        post_mortem_id: str,
        incident_id: str,
        action_items_data: list[dict]
    ) -> list[ActionItem]:
        """Create action items from a post-mortem and link them.

        Args:
            post_mortem_id: ID of the post-mortem document
            incident_id: ID of the related incident
            action_items_data: List of dicts with keys:
                - title: str
                - owner: str
                - due_date: datetime
                - category: str
                - runbook_update_required: bool

        Returns:
            List of created ActionItem objects

        Side effects:
        - Stores action items in self.action_items
        - Transitions incident to POST_MORTEM_PENDING if in RESOLVED phase
        - Logs a FeedbackEvent for each action item created
        """
        # IMPLEMENT THIS
        pass

    def complete_action_item(self, action_item_id: str,
                             completed_date: Optional[datetime] = None) -> None:
        """Mark an action item as completed.

        If the action item has runbook_update_required=True and
        runbook_update_done=False, log a warning FeedbackEvent indicating
        that the runbook still needs to be updated.

        Args:
            action_item_id: The ID of the action item to complete
            completed_date: When it was completed (defaults to now)
        """
        # IMPLEMENT THIS
        pass

    def update_runbook_from_post_mortem(
        self,
        runbook_id: str,
        post_mortem_id: str,
        update_type: RunbookUpdateType,
        new_content: str,
        action_item_id: Optional[str] = None
    ) -> None:
        """Update a runbook based on post-mortem findings.

        This is the key connection in the improvement cycle: post-mortem
        findings flow into runbooks that are used in future incident response.

        Args:
            runbook_id: ID of the runbook to update (or create)
            post_mortem_id: ID of the post-mortem driving the update
            update_type: Type of update
            new_content: The new or updated content
            action_item_id: Optional action item to mark as runbook_update_done

        Side effects:
        - Creates or updates the runbook entry
        - Links the post-mortem to the runbook
        - If action_item_id provided, marks runbook_update_done=True
        - Logs a FeedbackEvent
        """
        # IMPLEMENT THIS
        pass

    # --- Metrics and Analysis ---

    def get_improvement_cycle_metrics(self) -> dict:
        """Calculate metrics about the improvement cycle itself.

        Returns a dict with:
        - "incidents_total": total number of incidents
        - "post_mortems_completed": incidents in POST_MORTEM_COMPLETE phase
        - "post_mortem_rate": post_mortems_completed / incidents_total
        - "action_items_created": total action items
        - "action_items_completed": completed action items
        - "action_item_completion_rate": completed / created
        - "runbooks_updated_from_post_mortems": count of runbooks with
          related_post_mortems
        - "avg_time_to_post_mortem": average days from incident resolution
          to post-mortem completion
        - "cycle_health_score": 0-100 score based on all the above metrics

        The cycle_health_score formula:
        - post_mortem_rate * 25 (max 25 points)
        - action_item_completion_rate * 25 (max 25 points)
        - (runbooks_updated / action_items_requiring_runbook_update) * 25 (max 25 points)
        - 25 points if avg_time_to_post_mortem < 5 days, 0 otherwise
        """
        # IMPLEMENT THIS
        pass

    def get_stale_action_items(self,
                               overdue_threshold: timedelta = timedelta(days=7)
                               ) -> list[ActionItem]:
        """Find action items that are overdue.

        An action item is overdue if:
        - Status is OPEN or IN_PROGRESS
        - Due date + overdue_threshold < now

        Returns:
            List of overdue ActionItem objects, sorted by due_date ascending
        """
        # IMPLEMENT THIS
        pass

    def get_runbook_effectiveness_report(self) -> list[dict]:
        """Generate a report on runbook effectiveness.

        For each runbook, returns:
        - id, title, service
        - times_used
        - success_rate
        - related_post_mortems count
        - days_since_last_update
        - effectiveness_rating: "excellent" (>80%), "good" (60-80%),
          "needs_improvement" (40-60%), "poor" (<40%)

        Returns:
            List of dicts, one per runbook, sorted by success_rate ascending
        """
        # IMPLEMENT THIS
        pass

    def identify_improvement_gaps(self) -> list[dict]:
        """Identify gaps in the improvement cycle.

        Looks for:
        1. Incidents without post-mortems (resolved but no post-mortem)
        2. Post-mortems without action items
        3. Action items without owners or deadlines
        4. Runbook-related action items where runbook wasn't updated
        5. Services with repeated incidents but no runbook
        6. Runbooks with low success rates that need revision

        Returns:
            List of dicts with keys:
            - gap_type: str
            - description: str
            - severity: "high", "medium", "low"
            - recommended_action: str
        """
        # IMPLEMENT THIS
        pass

    # --- Notification ---

    def register_notification_callback(
        self, callback: Callable[[str, str, dict], None]
    ) -> None:
        """Register a callback for improvement cycle notifications.

        The callback receives:
        - event_type: str (e.g., "action_item_overdue", "post_mortem_needed")
        - message: str (human-readable description)
        - context: dict (related data)
        """
        self.notification_callbacks.append(callback)

    def check_and_notify(self) -> list[str]:
        """Check the improvement cycle state and send notifications.

        Sends notifications for:
        1. Overdue action items (use get_stale_action_items)
        2. Incidents in RESOLVED phase for >3 days without post-mortem
        3. Action items with runbook_update_required=True that haven't been done

        Returns:
            List of notification messages sent
        """
        # IMPLEMENT THIS
        pass

    # --- Import/Export ---

    def export_state(self) -> dict:
        """Export the entire feedback loop state as a JSON-serializable dict.

        Includes all incidents, runbooks, action items, and events.
        """
        # IMPLEMENT THIS
        pass

    def generate_cycle_report(self) -> str:
        """Generate a human-readable report on the improvement cycle.

        Sections:
        1. Cycle Health Score and summary
        2. Post-mortem completion rate
        3. Action item status breakdown
        4. Runbook effectiveness summary
        5. Identified gaps and recommendations
        6. Trend: is the cycle improving over time?
        """
        # IMPLEMENT THIS
        pass
```

### Part C: Integration Scenario

Walk through the following scenario using your FeedbackLoop implementation.
Document each step and the state of the system after each step.

**Scenario:**

1. An incident occurs: "Redis cache failure causes API timeouts"
   - Severity: SEV-2
   - Services affected: api-gateway, recommendation-service
   - Started: 2026-04-01 09:00
   - Detected: 2026-04-01 09:05 (by alert)
   - The on-call engineer uses runbook "RB-001: Redis Troubleshooting"
   - Mitigated: 2026-04-01 09:30 (manual cache clear)
   - Resolved: 2026-04-01 10:00 (cache rebuilt)

2. A post-mortem is held on 2026-04-02. Three action items are created:
   - "Implement Redis cluster with automatic failover" (infra-team, due: 2026-05-01)
   - "Update Redis runbook with cache-clear procedure" (sre-team, due: 2026-04-10, runbook_update_required: True)
   - "Add Redis memory utilization alerts" (sre-team, due: 2026-04-15)

3. On 2026-04-08, the SRE team updates runbook "RB-001" with the new
   cache-clear procedure found during the post-mortem.

4. On 2026-04-09, the alert action item is completed.

5. On 2026-04-15, you run `check_and_notify()`. What notifications are sent?

6. Generate a cycle report. What does it show?

```
[Document your step-by-step walkthrough here, including the state
of the system after each step and the final cycle report]
```

---

## Reflection Questions

1. **Feedback loops:** In your design, what prevents the cycle from breaking
   down? What mechanisms ensure action items actually get completed?

2. **Runbook evolution:** How does the system ensure that runbooks improve over
   time rather than becoming stale? What triggers a runbook update?

3. **Automation vs. human judgment:** Which parts of the improvement cycle should
   be automated, and which require human judgment? Why?

4. **Scaling:** How would this system change if you had 50 incidents per month
   instead of 5? What would break? What would you add?

5. **Measurement:** The cycle_health_score combines multiple metrics into a
   single number. What are the pros and cons of this approach? How would you
   present this to engineering leadership?

## Self-Assessment

Rate yourself after completing this exercise:

- [ ] I can design an architecture connecting incident response, runbooks, and post-mortems
- [ ] I can implement a feedback loop that tracks the improvement cycle
- [ ] I can calculate improvement cycle metrics (post-mortem rate, action item rate, etc.)
- [ ] I can identify gaps in the improvement cycle automatically
- [ ] I can implement notification logic for overdue items and broken cycles
- [ ] I understand how the three reliability practices reinforce each other
- [ ] I can reason about scaling and operationalizing this system

## Conclusion

This exercise ties together the three pillars of operational reliability:

- **Incident Response** (Module 76) -- handling incidents effectively
- **Runbooks** (Module 77) -- documenting operational knowledge
- **Post-Mortems** (Module 78) -- learning from incidents to prevent recurrence

The continuous improvement cycle is what transforms these from isolated
practices into a compounding reliability advantage. Each incident makes the
team better prepared for the next one, and the system tracks whether that
improvement is actually happening.

If you completed all five exercises in this module, you have practiced:

1. Blameless communication (Exercise 01)
2. Root cause analysis (Exercise 02)
3. Post-mortem documentation (Exercise 03)
4. Metrics and measurement (Exercise 04)
5. System design for continuous improvement (Exercise 05)

These skills form the foundation of a mature reliability culture. The goal
is not to prevent all incidents -- that is impossible. The goal is to ensure
that every incident makes the system and the team stronger.
