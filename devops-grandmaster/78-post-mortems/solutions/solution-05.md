# Solution 05: Continuous Improvement Pipeline

## Exercise Recap

Design and implement a continuous improvement pipeline that takes incidents
through detection, response, analysis, and improvement phases with automated
tracking at each stage.

## Architecture Overview

```
Continuous Improvement Pipeline -- Full Lifecycle:

  +----------------+     +----------------+     +----------------+
  |   PHASE 1      |     |   PHASE 2      |     |   PHASE 3      |
  |   Detection    |---->|   Response     |---->|   Analysis     |
  +----------------+     +----------------+     +----------------+
         |                      |                       |
         v                      v                       v
  +----------------+     +----------------+     +----------------+
  | Auto-detect    |     | Contain and    |     | Post-mortem    |
  | incidents via  |     | mitigate the   |     | review and     |
  | monitoring     |     | immediate      |     | root cause     |
  |                |     | impact         |     | analysis       |
  +----------------+     +----------------+     +----------------+
                                                        |
                                                        v
                                                 +----------------+
                                                 |   PHASE 4      |
                                                 |   Improvement  |
                                                 +----------------+
                                                        |
                                                        v
                                                 +----------------+
                                                 | Implement      |
                                                 | action items   |
                                                 | and validate   |
                                                 +----------------+
```

## Phase Definitions

```
Pipeline State Machine:

              +-----------+
              |  NORMAL   |<----------------------------------------+
              +-----------+                                         |
                   |                                                |
            [anomaly detected]                                     |
                   |                                                |
                   v                                                |
              +-----------+                                         |
              | DETECTING |                                         |
              +-----------+                                         |
                   |                                                |
            [incident declared]                                    |
                   |                                                |
                   v                                                |
              +-----------+     [impact contained]                  |
              | RESPONDING |-----------------------+                |
              +-----------+                        |                |
                   |                               |                |
            [mitigation applied]                   |                |
                   |                               |                |
                   v                               v                |
              +-----------+                  +-----------+          |
              | ANALYZING |                  | MITIGATED |          |
              +-----------+                  +-----------+          |
                   |                               |                |
            [post-mortem complete]          [root cause fixed]      |
                   |                               |                |
                   v                               |                |
              +-----------+                        |                |
              | IMPROVING |<-----------------------+                |
              +-----------+                                         |
                   |                                                |
            [all action items done]                                 |
                   |                                                |
                   +------------------------------------------------+
```

## Complete Implementation

```python
"""
Continuous Improvement Pipeline for Incident Management.

Automates the lifecycle from incident detection through post-mortem
completion and action item tracking, ensuring no incident is forgotten
and every incident leads to measurable improvement.
"""

from __future__ import annotations

import json
import uuid
from dataclasses import dataclass, field
from datetime import datetime, timedelta
from enum import Enum
from typing import Any, Callable, Optional


# ------------------------------------------------------------------ #
# Domain Models
# ------------------------------------------------------------------ #

class PipelinePhase(Enum):
    NORMAL = "normal"
    DETECTING = "detecting"
    RESPONDING = "responding"
    MITIGATED = "mitigated"
    ANALYZING = "analyzing"
    IMPROVING = "improving"


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


@dataclass
class ActionItem:
    """A tracked improvement action from a post-mortem."""
    id: str
    title: str
    owner: str
    priority: str
    due_date: datetime
    status: ActionItemStatus = ActionItemStatus.NOT_STARTED
    completed_date: Optional[datetime] = None
    evidence_url: Optional[str] = None

    def is_complete(self) -> bool:
        return self.status in (
            ActionItemStatus.DONE, ActionItemStatus.WONT_DO
        )


@dataclass
class PostMortem:
    """Analysis document for a completed incident."""
    id: str
    incident_id: str
    title: str
    root_cause: str
    contributing_factors: list[str] = field(default_factory=list)
    what_went_well: list[str] = field(default_factory=list)
    what_went_poorly: list[str] = field(default_factory=list)
    lessons_learned: list[str] = field(default_factory=list)
    action_items: list[ActionItem] = field(default_factory=list)
    review_status: str = "draft"  # draft, review, approved, final
    created_at: datetime = field(default_factory=datetime.now)
    reviewed_at: Optional[datetime] = None


@dataclass
class Incident:
    """A tracked incident moving through the improvement pipeline."""
    id: str
    title: str
    severity: Severity
    phase: PipelinePhase = PipelinePhase.DETECTING
    detected_at: datetime = field(default_factory=datetime.now)
    acknowledged_at: Optional[datetime] = None
    mitigated_at: Optional[datetime] = None
    resolved_at: Optional[datetime] = None
    post_mortem: Optional[PostMortem] = None
    timeline: list[dict[str, Any]] = field(default_factory=list)
    responders: list[str] = field(default_factory=list)
    affected_services: list[str] = field(default_factory=list)

    def add_timeline_entry(
        self, message: str, author: str = "system"
    ) -> None:
        self.timeline.append({
            "timestamp": datetime.now().isoformat(),
            "author": author,
            "message": message,
            "phase": self.phase.value,
        })


# ------------------------------------------------------------------ #
# Notification Interface
# ------------------------------------------------------------------ #

class NotificationChannel:
    """Base class for notification delivery."""

    def send(self, target: str, message: str) -> bool:
        raise NotImplementedError


class SlackNotification(NotificationChannel):
    """Simulated Slack notification."""

    def __init__(self, webhook_url: str) -> None:
        self.webhook_url = webhook_url
        self.sent_messages: list[dict[str, str]] = []

    def send(self, target: str, message: str) -> bool:
        self.sent_messages.append({
            "target": target,
            "message": message,
            "sent_at": datetime.now().isoformat(),
        })
        return True


class EmailNotification(NotificationChannel):
    """Simulated email notification."""

    def __init__(self, smtp_host: str) -> None:
        self.smtp_host = smtp_host
        self.sent_emails: list[dict[str, str]] = []

    def send(self, target: str, message: str) -> bool:
        self.sent_emails.append({
            "to": target,
            "body": message,
            "sent_at": datetime.now().isoformat(),
        })
        return True


# ------------------------------------------------------------------ #
# Pipeline Stage Handlers
# ------------------------------------------------------------------ #

class StageHandler:
    """Base class for pipeline stage logic."""

    def can_enter(self, incident: Incident) -> bool:
        raise NotImplementedError

    def on_enter(self, incident: Incident) -> None:
        raise NotImplementedError

    def on_exit(self, incident: Incident) -> None:
        raise NotImplementedError


class DetectionHandler(StageHandler):
    """Handles the DETECTING phase."""

    def __init__(self, tracker: "ImprovementPipeline") -> None:
        self.tracker = tracker

    def can_enter(self, incident: Incident) -> bool:
        return incident.phase == PipelinePhase.NORMAL

    def on_enter(self, incident: Incident) -> None:
        incident.phase = PipelinePhase.DETECTING
        incident.detected_at = datetime.now()
        incident.add_timeline_entry(
            f"Incident detected: {incident.title}"
        )
        self.tracker.notify(
            channel="slack",
            target="#incidents",
            message=(
                f":rotating_light: NEW INCIDENT [{incident.severity.value}]"
                f": {incident.title}\n"
                f"Incident ID: {incident.id}\n"
                f"Detected at: {incident.detected_at.isoformat()}"
            ),
        )

    def on_exit(self, incident: Incident) -> None:
        pass


class ResponseHandler(StageHandler):
    """Handles the RESPONDING phase."""

    def __init__(self, tracker: "ImprovementPipeline") -> None:
        self.tracker = tracker

    def can_enter(self, incident: Incident) -> bool:
        return incident.phase == PipelinePhase.DETECTING

    def on_enter(self, incident: Incident) -> None:
        incident.phase = PipelinePhase.RESPONDING
        incident.acknowledged_at = datetime.now()
        incident.add_timeline_entry(
            f"Incident acknowledged. "
            f"Responders: {', '.join(incident.responders)}"
        )
        self.tracker.notify(
            channel="slack",
            target="#incidents",
            message=(
                f"Incident {incident.id} acknowledged by "
                f"{', '.join(incident.responders)}"
            ),
        )

    def on_exit(self, incident: Incident) -> None:
        pass


class MitigatedHandler(StageHandler):
    """Handles the MITIGATED phase (impact contained, not root-caused)."""

    def __init__(self, tracker: "ImprovementPipeline") -> None:
        self.tracker = tracker

    def can_enter(self, incident: Incident) -> bool:
        return incident.phase == PipelinePhase.RESPONDING

    def on_enter(self, incident: Incident) -> None:
        incident.phase = PipelinePhase.MITIGATED
        incident.mitigated_at = datetime.now()
        mttd = (
            (incident.acknowledged_at - incident.detected_at).total_seconds()
            / 60
        ) if incident.acknowledged_at else 0
        mttc = (
            (incident.mitigated_at - incident.acknowledged_at).total_seconds()
            / 60
        ) if incident.acknowledged_at else 0

        incident.add_timeline_entry(
            f"Impact mitigated. MTTD: {mttd:.0f}m, MTTC: {mttc:.0f}m"
        )
        self.tracker.notify(
            channel="slack",
            target="#incidents",
            message=(
                f":white_check_mark: Incident {incident.id} MITIGATED\n"
                f"MTTD: {mttd:.0f} min | MTTC: {mttc:.0f} min\n"
                f"Root cause analysis pending."
            ),
        )
        # Schedule post-mortem creation
        self.tracker.schedule_post_mortem(incident)

    def on_exit(self, incident: Incident) -> None:
        pass


class AnalysisHandler(StageHandler):
    """Handles the ANALYZING phase (post-mortem and root cause)."""

    def __init__(self, tracker: "ImprovementPipeline") -> None:
        self.tracker = tracker

    def can_enter(self, incident: Incident) -> bool:
        return incident.phase in (
            PipelinePhase.MITIGATED, PipelinePhase.RESPONDING
        )

    def on_enter(self, incident: Incident) -> None:
        incident.phase = PipelinePhase.ANALYZING
        incident.add_timeline_entry(
            "Post-mortem analysis phase started"
        )
        self.tracker.notify(
            channel="email",
            target=incident.responders,
            message=(
                f"Post-mortem analysis started for {incident.id}.\n"
                f"Please complete the post-mortem within 5 business days."
            ),
        )

    def on_exit(self, incident: Incident) -> None:
        if incident.post_mortem is None:
            raise ValueError(
                "Cannot exit ANALYZING phase without a post-mortem"
            )
        if incident.post_mortem.review_status != "approved":
            raise ValueError(
                "Post-mortem must be approved before moving to IMPROVING"
            )


class ImprovementHandler(StageHandler):
    """Handles the IMPROVING phase (action item execution)."""

    def __init__(self, tracker: "ImprovementPipeline") -> None:
        self.tracker = tracker

    def can_enter(self, incident: Incident) -> bool:
        return incident.phase == PipelinePhase.ANALYZING

    def on_enter(self, incident: Incident) -> None:
        incident.phase = PipelinePhase.IMPROVING
        incident.resolved_at = datetime.now()
        incident.add_timeline_entry(
            "Post-mortem approved. Action item execution phase started."
        )
        total_duration = (
            (incident.resolved_at - incident.detected_at).total_seconds()
            / 60
        )
        self.tracker.notify(
            channel="slack",
            target="#incidents",
            message=(
                f":chart_with_upwards_trend: Incident {incident.id} "
                f"moving to improvement phase.\n"
                f"Total duration: {total_duration:.0f} min\n"
                f"Action items: "
                f"{len(incident.post_mortem.action_items) if incident.post_mortem else 0}"
            ),
        )

    def on_exit(self, incident: Incident) -> None:
        if incident.post_mortem is None:
            return
        incomplete = [
            ai for ai in incident.post_mortem.action_items
            if not ai.is_complete()
        ]
        if incomplete:
            raise ValueError(
                f"Cannot exit IMPROVING phase: {len(incomplete)} "
                f"action items still incomplete"
            )


# ------------------------------------------------------------------ #
# Main Pipeline Orchestrator
# ------------------------------------------------------------------ #

class ImprovementPipeline:
    """
    Orchestrates the continuous improvement pipeline.

    Manages incident lifecycle from detection through improvement,
    with automated notifications and tracking at each phase transition.
    """

    def __init__(self) -> None:
        self._incidents: dict[str, Incident] = {}
        self._notifications: dict[str, NotificationChannel] = {}
        self._handlers: dict[PipelinePhase, StageHandler] = {}
        self._post_mortem_scheduled: set[str] = set()

        self._register_default_handlers()

    def _register_default_handlers(self) -> None:
        """Set up the default stage handlers."""
        self._handlers[PipelinePhase.DETECTING] = DetectionHandler(self)
        self._handlers[PipelinePhase.RESPONDING] = ResponseHandler(self)
        self._handlers[PipelinePhase.MITIGATED] = MitigatedHandler(self)
        self._handlers[PipelinePhase.ANALYZING] = AnalysisHandler(self)
        self._handlers[PipelinePhase.IMPROVING] = ImprovementHandler(self)

    def register_notification_channel(
        self, name: str, channel: NotificationChannel
    ) -> None:
        """Register a named notification channel."""
        self._notifications[name] = channel

    def notify(self, channel: str, target: str, message: str) -> bool:
        """Send a notification through the specified channel."""
        if channel not in self._notifications:
            return False
        return self._notifications[channel].send(target, message)

    # -------------------------------------------------------------- #
    # Incident Lifecycle
    # -------------------------------------------------------------- #

    def declare_incident(
        self,
        title: str,
        severity: Severity,
        affected_services: list[str],
        responders: list[str],
    ) -> Incident:
        """
        Declare a new incident and enter the detection phase.

        This is the entry point for new incidents into the pipeline.
        """
        incident = Incident(
            id=f"INC-{uuid.uuid4().hex[:8].upper()}",
            title=title,
            severity=severity,
            affected_services=affected_services,
            responders=responders,
        )
        self._incidents[incident.id] = incident

        handler = self._handlers[PipelinePhase.DETECTING]
        handler.on_enter(incident)

        return incident

    def acknowledge_incident(
        self, incident_id: str, responders: Optional[list[str]] = None
    ) -> Incident:
        """Acknowledge an incident and enter the response phase."""
        incident = self._get_incident(incident_id)
        if responders:
            incident.responders = responders
        self._transition(incident, PipelinePhase.RESPONDING)
        return incident

    def mitigate_incident(
        self, incident_id: str, mitigation_note: str = ""
    ) -> Incident:
        """
        Declare an incident mitigated (impact contained).

        Root cause may still be unknown. Triggers post-mortem scheduling.
        """
        incident = self._get_incident(incident_id)
        if mitigation_note:
            incident.add_timeline_entry(
                f"Mitigation: {mitigation_note}",
                author="responder",
            )
        self._transition(incident, PipelinePhase.MITIGATED)
        return incident

    def start_analysis(self, incident_id: str) -> Incident:
        """Move an incident into the analysis (post-mortem) phase."""
        incident = self._get_incident(incident_id)
        self._transition(incident, PipelinePhase.ANALYZING)
        return incident

    def submit_post_mortem(
        self, incident_id: str, post_mortem: PostMortem
    ) -> Incident:
        """Attach a post-mortem to an incident."""
        incident = self._get_incident(incident_id)
        incident.post_mortem = post_mortem
        incident.add_timeline_entry(
            f"Post-mortem submitted: {post_mortem.title}",
            author="author",
        )
        return incident

    def approve_post_mortem(self, incident_id: str) -> Incident:
        """Approve the post-mortem and move to improvement phase."""
        incident = self._get_incident(incident_id)
        if incident.post_mortem is None:
            raise ValueError("No post-mortem attached to this incident")
        incident.post_mortem.review_status = "approved"
        incident.post_mortem.reviewed_at = datetime.now()
        self._transition(incident, PipelinePhase.IMPROVING)
        return incident

    def complete_action_item(
        self,
        incident_id: str,
        action_item_id: str,
        evidence_url: Optional[str] = None,
    ) -> None:
        """Mark an action item as complete with optional evidence."""
        incident = self._get_incident(incident_id)
        if incident.post_mortem is None:
            raise ValueError("No post-mortem attached")

        for ai in incident.post_mortem.action_items:
            if ai.id == action_item_id:
                ai.status = ActionItemStatus.DONE
                ai.completed_date = datetime.now()
                ai.evidence_url = evidence_url
                incident.add_timeline_entry(
                    f"Action item completed: {ai.title}",
                    author="owner",
                )
                self._check_improvement_completion(incident)
                return

        raise ValueError(
            f"Action item {action_item_id} not found in incident"
        )

    # -------------------------------------------------------------- #
    # Internal Methods
    # -------------------------------------------------------------- #

    def _get_incident(self, incident_id: str) -> Incident:
        if incident_id not in self._incidents:
            raise ValueError(f"Incident {incident_id} not found")
        return self._incidents[incident_id]

    def _transition(
        self, incident: Incident, target_phase: PipelinePhase
    ) -> None:
        """Execute a phase transition with handler hooks."""
        handler = self._handlers.get(target_phase)
        if handler is None:
            raise ValueError(
                f"No handler registered for phase {target_phase.value}"
            )
        if not handler.can_enter(incident):
            raise ValueError(
                f"Cannot transition from {incident.phase.value} "
                f"to {target_phase.value}"
            )
        # Exit hook for current phase
        current_handler = self._handlers.get(incident.phase)
        if current_handler:
            current_handler.on_exit(incident)
        # Enter hook for new phase
        handler.on_enter(incident)

    def schedule_post_mortem(self, incident: Incident) -> None:
        """Schedule post-mortem creation reminder."""
        self._post_mortem_scheduled.add(incident.id)
        self.notify(
            channel="email",
            target=incident.responders,
            message=(
                f"Post-mortem required for {incident.id}: "
                f"{incident.title}\n"
                f"Severity: {incident.severity.value}\n"
                f"Deadline: 5 business days from today."
            ),
        )

    def _check_improvement_completion(self, incident: Incident) -> None:
        """Check if all action items are done and close the incident."""
        if incident.post_mortem is None:
            return
        all_done = all(
            ai.is_complete()
            for ai in incident.post_mortem.action_items
        )
        if all_done:
            incident.add_timeline_entry(
                "All action items completed. Incident fully resolved."
            )
            self.notify(
                channel="slack",
                target="#incidents",
                message=(
                    f":tada: Incident {incident.id} FULLY RESOLVED.\n"
                    f"All {len(incident.post_mortem.action_items)} "
                    f"action items completed."
                ),
            )

    # -------------------------------------------------------------- #
    # Reporting
    # -------------------------------------------------------------- #

    def get_incident_status(self, incident_id: str) -> dict[str, Any]:
        """Get a summary of an incident's current state."""
        incident = self._get_incident(incident_id)
        status: dict[str, Any] = {
            "id": incident.id,
            "title": incident.title,
            "severity": incident.severity.value,
            "phase": incident.phase.value,
            "detected_at": incident.detected_at.isoformat(),
            "affected_services": incident.affected_services,
            "responders": incident.responders,
        }

        if incident.acknowledged_at:
            mttd = (
                (incident.acknowledged_at - incident.detected_at)
                .total_seconds() / 60
            )
            status["mttd_minutes"] = round(mttd, 1)

        if incident.mitigated_at and incident.acknowledged_at:
            mttc = (
                (incident.mitigated_at - incident.acknowledged_at)
                .total_seconds() / 60
            )
            status["mttc_minutes"] = round(mttc, 1)

        if incident.post_mortem:
            pm = incident.post_mortem
            status["post_mortem"] = {
                "id": pm.id,
                "status": pm.review_status,
                "root_cause": pm.root_cause,
                "action_items_total": len(pm.action_items),
                "action_items_done": sum(
                    1 for ai in pm.action_items if ai.is_complete()
                ),
                "action_items_pending": sum(
                    1 for ai in pm.action_items if not ai.is_complete()
                ),
            }

        return status

    def pipeline_summary(self) -> dict[str, Any]:
        """Get a summary of all incidents in the pipeline."""
        summary: dict[str, Any] = {
            "total_incidents": len(self._incidents),
            "by_phase": {},
            "by_severity": {},
            "overdue_post_mortems": [],
            "overdue_action_items": [],
        }

        for phase in PipelinePhase:
            count = sum(
                1 for i in self._incidents.values()
                if i.phase == phase
            )
            summary["by_phase"][phase.value] = count

        for sev in Severity:
            count = sum(
                1 for i in self._incidents.values()
                if i.severity == sev
            )
            summary["by_severity"][sev.value] = count

        now = datetime.now()
        for incident in self._incidents.values():
            # Check for overdue post-mortems (5 business days)
            if (
                incident.phase in (
                    PipelinePhase.MITIGATED,
                    PipelinePhase.ANALYZING,
                )
                and incident.mitigated_at
                and (now - incident.mitigated_at).days > 7
            ):
                summary["overdue_post_mortems"].append(incident.id)

            # Check for overdue action items
            if incident.post_mortem:
                for ai in incident.post_mortem.action_items:
                    if ai.is_overdue():
                        summary["overdue_action_items"].append({
                            "incident_id": incident.id,
                            "action_item_id": ai.id,
                            "title": ai.title,
                            "owner": ai.owner,
                            "due_date": ai.due_date.isoformat(),
                        })

        return summary

    def export_incident(self, incident_id: str) -> str:
        """Export an incident as JSON."""
        status = self.get_incident_status(incident_id)
        return json.dumps(status, indent=2)


# ------------------------------------------------------------------ #
# Demonstration
# ------------------------------------------------------------------ #

def demo() -> None:
    """Walk through a complete incident lifecycle."""

    # Set up the pipeline
    pipeline = ImprovementPipeline()

    # Register notification channels
    slack = SlackNotification(webhook_url="https://hooks.slack.com/fake")
    email = EmailNotification(smtp_host="smtp.example.com")
    pipeline.register_notification_channel("slack", slack)
    pipeline.register_notification_channel("email", email)

    # --- Phase 1: Detection ---
    print("=== PHASE 1: DETECTION ===")
    incident = pipeline.declare_incident(
        title="Payment processing failure during peak traffic",
        severity=Severity.SEV1,
        affected_services=["payment-service", "checkout-api"],
        responders=["maria.gonzalez", "alex.chen"],
    )
    print(f"Declared: {incident.id}")
    print(f"Phase: {incident.phase.value}")
    print()

    # --- Phase 2: Response ---
    print("=== PHASE 2: RESPONSE ===")
    pipeline.acknowledge_incident(
        incident.id,
        responders=["maria.gonzalez", "alex.chen", "raj.patel"],
    )
    print(f"Phase: {incident.phase.value}")
    print(f"Responders: {incident.responders}")
    print()

    # --- Phase 3: Mitigation ---
    print("=== PHASE 3: MITIGATION ===")
    pipeline.mitigate_incident(
        incident.id,
        mitigation_note="Restarted payment-service pods to reset "
                        "connection pool",
    )
    print(f"Phase: {incident.phase.value}")
    print(f"Mitigated at: {incident.mitigated_at}")
    print()

    # --- Phase 4: Analysis ---
    print("=== PHASE 4: ANALYSIS ===")
    pipeline.start_analysis(incident.id)
    print(f"Phase: {incident.phase.value}")

    post_mortem = PostMortem(
        id=f"PM-{uuid.uuid4().hex[:8].upper()}",
        incident_id=incident.id,
        title="Payment Service Connection Pool Exhaustion",
        root_cause=(
            "Missing circuit breaker and connection pool monitoring "
            "allowed slow fraud detection responses to exhaust the "
            "payment service connection pool."
        ),
        contributing_factors=[
            "No circuit breaker for fraud detection dependency",
            "No connection pool utilization monitoring",
            "Batch job running during peak traffic",
            "Latency alert threshold too high (5s vs 1s normal p99)",
        ],
        what_went_well=[
            "Fast resolution once on-call was paged (13 min)",
            "No data corruption",
            "Effective rolling restart",
        ],
        what_went_poorly=[
            "10-minute detection gap (customer support found it)",
            "No automated mitigation",
            "No connection pool visibility",
        ],
        lessons_learned=[
            "Resilience patterns are non-negotiable for dependencies",
            "Resource pools need dedicated monitoring",
            "Peak traffic requires peak caution for batch jobs",
        ],
        action_items=[
            ActionItem(
                id="AI-001",
                title="Implement circuit breaker for fraud detection",
                owner="alex.chen",
                priority="P0",
                due_date=datetime.now() + timedelta(days=14),
            ),
            ActionItem(
                id="AI-002",
                title="Add connection pool metrics and alerts",
                owner="maria.gonzalez",
                priority="P0",
                due_date=datetime.now() + timedelta(days=10),
            ),
            ActionItem(
                id="AI-003",
                title="Lower fraud detection latency alert to 1s p99",
                owner="raj.patel",
                priority="P0",
                due_date=datetime.now() + timedelta(days=5),
            ),
        ],
    )

    pipeline.submit_post_mortem(incident.id, post_mortem)
    print(f"Post-mortem attached: {post_mortem.id}")
    print()

    # --- Phase 5: Approval and Improvement ---
    print("=== PHASE 5: IMPROVEMENT ===")
    pipeline.approve_post_mortem(incident.id)
    print(f"Phase: {incident.phase.value}")
    print(f"Post-mortem status: {post_mortem.review_status}")

    # Complete action items
    for ai in post_mortem.action_items:
        pipeline.complete_action_item(
            incident.id,
            ai.id,
            evidence_url=f"https://github.com/org/repo/pull/{ai.id}",
        )
        print(f"  Completed: {ai.title}")

    print()

    # --- Final Status ---
    print("=== FINAL STATUS ===")
    status = pipeline.get_incident_status(incident.id)
    print(json.dumps(status, indent=2))
    print()

    # --- Pipeline Summary ---
    print("=== PIPELINE SUMMARY ===")
    summary = pipeline.pipeline_summary()
    print(json.dumps(summary, indent=2))
    print()

    # --- Notification Audit ---
    print("=== NOTIFICATION AUDIT ===")
    print(f"Slack messages sent: {len(slack.sent_messages)}")
    for msg in slack.sent_messages:
        print(f"  -> {msg['target']}: {msg['message'][:60]}...")
    print(f"Emails sent: {len(email.sent_emails)}")
    for em in email.sent_emails:
        print(f"  -> {em['to']}: {em['body'][:60]}...")


if __name__ == "__main__":
    demo()
```

## Architecture Diagram: Data Flow

```
                +-----------------------+
                |    Monitoring /       |
                |    Alerting System    |
                +-----------+-----------+
                            |
                   Anomaly Detected
                            |
                            v
                +-----------+-----------+
                |  ImprovementPipeline  |
                |                       |
                |  declare_incident()   |
                +-----------+-----------+
                            |
                            v
                +-----------+-----------+     +------------------+
                |   DetectionHandler    |---->| Slack: #incidents|
                |   (auto-notify)       |     +------------------+
                +-----------+-----------+
                            |
                   acknowledge_incident()
                            |
                            v
                +-----------+-----------+     +------------------+
                |   ResponseHandler     |---->| Slack: #incidents|
                |   (track responders)  |     +------------------+
                +-----------+-----------+
                            |
                   mitigate_incident()
                            |
                            v
                +-----------+-----------+     +------------------+
                |   MitigatedHandler    |---->| Email: responders|
                |   (schedule PM)       |     +------------------+
                +-----------+-----------+
                            |
                   start_analysis()
                            |
                            v
                +-----------+-----------+
                |   AnalysisHandler     |
                |   (post-mortem)       |
                +-----------+-----------+
                            |
                   submit_post_mortem()
                   approve_post_mortem()
                            |
                            v
                +-----------+-----------+     +------------------+
                |  ImprovementHandler   |---->| Slack: #incidents|
                |  (action items)       |     +------------------+
                +-----------+-----------+
                            |
                   complete_action_item() x N
                            |
                            v
                +-----------+-----------+
                |    ALL RESOLVED       |
                |    (pipeline closed)  |
                +-----------------------+
```

## Key Design Decisions

### Phase Guards

Each `StageHandler.can_enter()` method enforces valid transitions. This
prevents incidents from skipping phases or moving backwards:

```
Valid transitions:
  NORMAL -> DETECTING -> RESPONDING -> MITIGATED -> ANALYZING -> IMPROVING

Invalid transitions (blocked by can_enter()):
  NORMAL -> ANALYZING (skipped detection and response)
  IMPROVING -> DETECTING (backwards)
  MITIGATING -> RESPONDING (backwards)
```

### Exit Hooks Validate Prerequisites

The `on_exit()` method on `AnalysisHandler` validates that a post-mortem
exists and is approved before allowing transition to IMPROVING. This
enforces the process: you cannot skip the analysis phase.

### Notification as a Side Effect

Notifications are sent as side effects of phase transitions, not as
separate operations. This ensures that every state change is communicated
without requiring the caller to remember to send notifications.

### Action Item Completion Drives Resolution

The pipeline does not consider an incident "fully resolved" until all
action items are complete. The `_check_improvement_completion()` method
automatically sends a resolution notification when the last action item
is done.

### Exportability

The `export_incident()` method produces JSON suitable for dashboards,
Slack bots, or integration with project management tools. The
`pipeline_summary()` method provides a portfolio-level view.

## Common Mistakes to Avoid

1. **No automated phase transitions.** If phase transitions require
   manual clicks or tickets, people will forget. Automate as much as
   possible (detection from monitoring, mitigation from runbooks,
   reminders from deadlines).

2. **No notification audit trail.** Keep a record of all notifications
   sent. This is essential for post-incident review of the response
   process itself.

3. **Allowing phase skipping.** The analysis phase is the most commonly
   skipped. Enforce it programmatically: no incident moves to "resolved"
   without an approved post-mortem.

4. **No deadline enforcement.** Action items without automated reminders
   and escalation will be forgotten. Build in deadline tracking with
   escalation paths (owner -> manager -> VP).

5. **Tying the pipeline to a single tool.** The pipeline should be
   tool-agnostic. Use notification channels and handlers to abstract
   away the specific tools (Slack, PagerDuty, Jira, etc.).
