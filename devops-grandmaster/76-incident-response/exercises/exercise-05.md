# Exercise 05: Automated Incident Response System

**Type:** Integration
**Time:** 45 minutes
**Difficulty:** Hard

## Objective

Build a Python-based automated incident response manager that integrates with PagerDuty,
Slack, and a status page API. This system will detect alerts, classify severity, create
incident channels, page on-call engineers, post status updates, and track the incident
through its lifecycle -- all with minimal human intervention.

## Prerequisites

- Completion of Exercises 01-04.
- Python 3.10+ installed.
- Familiarity with Python async/await, dataclasses, and type hints.
- Basic understanding of REST APIs and webhooks.
- (Optional) A PagerDuty developer account or mock API server.

## Scenario

Acme Corp wants to automate the boilerplate parts of incident response. The Incident
Commander should focus on investigation and decision-making, not creating Slack channels
and drafting status page updates. Your job is to build the automation layer.

```
┌──────────────────────────────────────────────────────────────────────────┐
│                AUTOMATED INCIDENT RESPONSE ARCHITECTURE                  │
│                                                                          │
│  ┌──────────┐    ┌─────────────────┐    ┌──────────────────────────────┐ │
│  │Prometheus │───>│  Alert Webhook  │───>│  Incident Response Manager  │ │
│  │ /Alertmgr│    │  (HTTP POST)    │    │  (Python Application)       │ │
│  └──────────┘    └─────────────────┘    │                              │ │
│                                          │  ┌────────────────────────┐ │ │
│                                          │  │  Severity Classifier   │ │ │
│                                          │  └──────────┬─────────────┘ │ │
│                                          │             │               │ │
│                                          │  ┌──────────v─────────────┐ │ │
│                                          │  │  Incident Lifecycle    │ │ │
│                                          │  │  State Machine         │ │ │
│                                          │  └──────────┬─────────────┘ │ │
│                                          │             │               │ │
│                        ┌─────────────────┤  ┌──────────v─────────────┐ │ │
│                        │                 │  │  Communication Engine  │ │ │
│                        │                 │  └──────────┬─────────────┘ │ │
│                        │                 │             │               │ │
│                        │                 └─────────────┼───────────────┘ │
│                        │                               │                 │
│          ┌─────────────v───────┐    ┌─────────────────v──────────────┐  │
│          │    PagerDuty API    │    │        Slack API                │  │
│          │  - Create Incident  │    │  - Create Channel              │  │
│          │  - Escalate         │    │  - Post Messages               │  │
│          │  - Acknowledge      │    │  - Invite Members              │  │
│          └─────────────────────┘    └────────────────────────────────┘  │
│                                                                         │
│          ┌─────────────────────────────────────────────────────────────┐ │
│          │              Status Page API                                │ │
│          │  - Create Incident  - Update Status  - Resolve             │ │
│          └─────────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────────────┘
```

## Tasks

### Part A: Define the Data Model

Create a Python module `models.py` that defines the core data structures using dataclasses
and enums:

```python
# models.py
from dataclasses import dataclass, field
from enum import Enum
from datetime import datetime
from typing import Optional

class Severity(Enum):
    P1 = "critical"
    P2 = "high"
    P3 = "medium"
    P4 = "low"

class IncidentState(Enum):
    TRIGGERED = "triggered"
    ASSESSED = "assessed"
    MOBILIZED = "mobilized"
    INVESTIGATING = "investigating"
    IDENTIFIED = "identified"
    MONITORING = "monitoring"
    RESOLVED = "resolved"

# Define Alert, Incident, EscalationTarget, and CommunicationRecord dataclasses
```

Define the following dataclasses:

1. `Alert` -- represents an incoming alert from Prometheus/Alertmanager.
2. `Incident` -- the core incident object with state tracking, timeline, and communications.
3. `EscalationTarget` -- a person to page with contact info and escalation order.
4. `CommunicationRecord` -- tracks each message sent (channel, content, timestamp).

<details><summary>Hint</summary>
The `Incident` dataclass should have a `timeline: list[TimelineEvent]` field that records
every state transition with a timestamp. The `CommunicationRecord` should store the
channel type (slack, status_page, pagerduty), the content, and the response from the API.
Use `field(default_factory=list)` for mutable defaults.
</details>

### Part B: Build the Severity Classifier

Create a module `classifier.py` that automatically classifies alerts into severity levels:

```python
# classifier.py
from models import Alert, Severity

class SeverityClassifier:
    """
    Classifies incoming alerts based on:
    - Alert labels (service, team, environment)
    - Alert annotations (description, runbook)
    - Historical patterns (firing duration, repeat count)
    """

    RULES: dict[str, Severity] = {
        # Define classification rules
    }

    def classify(self, alert: Alert) -> Severity:
        """Classify an alert into a severity level."""
        # Implement classification logic
        pass
```

Implement classification rules for these alert types:

| Alert Name | Condition | Severity |
|------------|-----------|----------|
| `KubePodCrashLooping` | All pods in deployment affected | P1 |
| `KubePodCrashLooping` | Some pods affected | P2 |
| `HighErrorRate` | Error rate > 50% | P1 |
| `HighErrorRate` | Error rate > 10% | P2 |
| `HighLatency` | p99 > 10s | P2 |
| `HighLatency` | p99 > 5s | P3 |
| `DiskSpaceLow` | < 5% free | P2 |
| `DiskSpaceLow` | < 15% free | P3 |
| `CertificateExpiring` | < 7 days | P2 |
| `CertificateExpiring` | < 30 days | P4 |

<details><summary>Hint</summary>
Use a rule-based approach with pattern matching on the alert name and conditions. Parse
the alert annotations for numeric thresholds. A scoring system works well: assign points
based on the alert type, the percentage of affected resources, and the environment
(production = 2x multiplier). Map the total score to a severity level.
</details>

### Part C: Implement the PagerDuty Integration

Create a module `pagerduty_client.py` that wraps the PagerDuty v2 Events API:

```python
# pagerduty_client.py
import httpx
from models import Incident, Severity

class PagerDutyClient:
    BASE_URL = "https://events.pagerduty.com/v2"

    def __init__(self, routing_key: str):
        self.routing_key = routing_key
        self.client = httpx.AsyncClient()

    async def create_incident(self, incident: Incident) -> dict:
        """Create a PagerDuty incident via Events API v2."""
        # POST to /enqueue
        pass

    async def acknowledge(self, incident_key: str) -> dict:
        """Acknowledge a PagerDuty incident."""
        # POST to /enqueue with event_action: "acknowledge"
        pass

    async def resolve(self, incident_key: str) -> dict:
        """Resolve a PagerDuty incident."""
        # POST to /enqueue with event_action: "resolve"
        pass
```

Implement all three methods using the PagerDuty Events API v2 format:

```json
{
  "routing_key": "...",
  "event_action": "trigger",
  "dedup_key": "incident-12345",
  "payload": {
    "summary": "...",
    "severity": "critical",
    "source": "incident-response-manager",
    "custom_details": { ... }
  }
}
```

<details><summary>Hint</summary>
The PagerDuty Events API v2 uses the `/v2/enqueue` endpoint. The `event_action` field
determines the action: `trigger`, `acknowledge`, or `resolve`. The `dedup_key` is used to
deduplicate events -- use the incident ID. The `severity` field maps directly from your
Severity enum: critical, error, warning, info.
</details>

### Part D: Implement the Slack Integration

Create a module `slack_client.py` that manages incident channels:

```python
# slack_client.py
import httpx
from models import Incident, Severity

class SlackClient:
    BASE_URL = "https://slack.com/api"

    def __init__(self, bot_token: str):
        self.bot_token = bot_token
        self.client = httpx.AsyncClient()
        self.headers = {"Authorization": f"Bearer {bot_token}"}

    async def create_incident_channel(self, incident: Incident) -> str:
        """Create a dedicated Slack channel for the incident."""
        # channels.create with naming convention: inc-YYYYMMDD-<short-id>
        pass

    async def post_message(self, channel: str, blocks: list[dict]) -> dict:
        """Post a formatted message to a Slack channel."""
        # chat.postMessage with Block Kit blocks
        pass

    async def invite_to_channel(self, channel_id: str, user_ids: list[str]) -> None:
        """Invite users to the incident channel."""
        # conversations.invite
        pass

    def build_status_blocks(self, incident: Incident) -> list[dict]:
        """Build Slack Block Kit blocks for an incident status update."""
        # Return Block Kit JSON for a formatted status message
        pass
```

Implement the channel creation with the naming convention `inc-YYYYMMDD-<short-id>`, and
build a Block Kit message that includes:

- Header with severity badge and incident title.
- Current state and duration.
- Affected services.
- Latest timeline entry.
- Links to dashboards and runbooks.

<details><summary>Hint</summary>
Slack Block Kit messages use a JSON structure with `blocks` as the top-level array. Use
`section` blocks for text, `divider` for separators, and `actions` for interactive buttons.
The severity badge can be an emoji: `:red_circle:` for P1, `:large_orange_circle:` for P2,
`:large_yellow_circle:` for P3, `:white_circle:` for P4.
</details>

### Part E: Build the Incident Manager (Orchestrator)

Create the main module `incident_manager.py` that ties everything together:

```python
# incident_manager.py
import asyncio
from models import Alert, Incident, IncidentState, Severity
from classifier import SeverityClassifier
from pagerduty_client import PagerDutyClient
from slack_client import SlackClient

class IncidentManager:
    def __init__(self, config: dict):
        self.classifier = SeverityClassifier()
        self.pagerduty = PagerDutyClient(config["pagerduty_routing_key"])
        self.slack = SlackClient(config["slack_bot_token"])
        self.incidents: dict[str, Incident] = {}

    async def handle_alert(self, alert: Alert) -> Incident:
        """Main entry point: process an incoming alert."""
        # 1. Classify severity
        # 2. Create incident object
        # 3. Transition to ASSESSED state
        # 4. Create PagerDuty incident
        # 5. Create Slack channel
        # 6. Post initial status
        # 7. Transition to MOBILIZED state
        # Return the incident
        pass

    async def update_incident_state(self, incident_id: str, new_state: IncidentState) -> None:
        """Transition an incident to a new state and trigger communications."""
        # Validate state transition
        # Update state
        # Trigger appropriate communications based on new state
        # Post to Slack
        # Update PagerDuty (acknowledge, resolve, etc.)
        pass

    async def run_health_check(self) -> dict:
        """Check the health of all integrations."""
        # Ping PagerDuty, Slack, and status page
        # Return health status
        pass
```

Implement the full `handle_alert` flow. Use Python's `asyncio` to run the PagerDuty and
Slack operations concurrently where possible (e.g., create the PagerDuty incident and
Slack channel at the same time).

Add a state machine that validates transitions -- for example, you cannot go from
`TRIGGERED` directly to `RESOLVED`; you must pass through the intermediate states.

<details><summary>Hint</summary>
Use `asyncio.gather()` to run the PagerDuty incident creation and Slack channel creation
in parallel. For the state machine, define valid transitions as a dictionary:

```python
VALID_TRANSITIONS = {
    IncidentState.TRIGGERED: {IncidentState.ASSESSED},
    IncidentState.ASSESSED: {IncidentState.MOBILIZED},
    IncidentState.MOBILIZED: {IncidentState.INVESTIGATING},
    IncidentState.INVESTIGATING: {IncidentState.IDENTIFIED},
    IncidentState.IDENTIFIED: {IncidentState.MONITORING},
    IncidentState.MONITORING: {IncidentState.RESOLVED, IncidentState.INVESTIGATING},
    IncidentState.RESOLVED: set(),
}
```

Allow `MONITORING -> INVESTIGATING` for incidents that recur during the monitoring period.
</details>

### Part F: Test with a Simulated Alert

Write a test script `test_incident_manager.py` that simulates the full flow:

```python
# test_incident_manager.py
import asyncio
from models import Alert
from incident_manager import IncidentManager

async def test_full_lifecycle():
    """Simulate a P1 incident through the full lifecycle."""
    config = {
        "pagerduty_routing_key": "test-routing-key",
        "slack_bot_token": "xoxb-test-token",
    }

    manager = IncidentManager(config)

    # Create a simulated alert
    alert = Alert(
        name="KubePodCrashLooping",
        labels={
            "alertname": "KubePodCrashLooping",
            "namespace": "production",
            "deployment": "api-gateway",
            "severity": "critical",
        },
        annotations={
            "summary": "Pod api-gateway-7f8b9-xk4lp is crash looping",
            "description": "All 12 pods in api-gateway are in CrashLoopBackOff",
        },
    )

    # Run through the lifecycle
    incident = await manager.handle_alert(alert)
    print(f"Incident created: {incident.incident_id}")
    print(f"Severity: {incident.severity}")
    print(f"State: {incident.state}")

    # Simulate state transitions
    await manager.update_incident_state(incident.incident_id, IncidentState.INVESTIGATING)
    await manager.update_incident_state(incident.incident_id, IncidentState.IDENTIFIED)
    await manager.update_incident_state(incident.incident_id, IncidentState.MONITORING)
    await manager.update_incident_state(incident.incident_id, IncidentState.RESOLVED)

    # Print final timeline
    for event in incident.timeline:
        print(f"  {event.timestamp} - {event.state.value}: {event.note}")

if __name__ == "__main__":
    asyncio.run(test_full_lifecycle())
```

Since you likely do not have real PagerDuty or Slack credentials, create mock classes that
simulate the API responses:

```python
# mocks.py
class MockPagerDutyClient:
    """Simulates PagerDuty API responses."""
    async def create_incident(self, incident):
        return {"status": "success", "incident_key": f"pd-{incident.incident_id}"}

class MockSlackClient:
    """Simulates Slack API responses."""
    async def create_incident_channel(self, incident):
        return f"C{incident.incident_id[:8]}"
```

<details><summary>Hint</summary>
Use dependency injection to swap real clients with mocks. Add a `use_mocks: bool` parameter
to the IncidentManager constructor. For the test, also add assertions that verify:
(1) the incident transitions through all states in order, (2) the timeline has entries for
each state, and (3) the severity was classified correctly.
</details>

## Success Criteria

- [ ] The `models.py` module defines all required dataclasses with proper type hints.
- [ ] The severity classifier correctly categorizes all 10 alert types from the table.
- [ ] The PagerDuty client implements trigger, acknowledge, and resolve operations.
- [ ] The Slack client creates channels with the correct naming convention and posts Block Kit messages.
- [ ] The Incident Manager orchestrates the full lifecycle with concurrent API calls.
- [ ] The state machine validates transitions and rejects invalid ones.
- [ ] The test script runs end-to-end with mock clients and prints a complete timeline.

## What You Should Understand After This Exercise

Automated incident response is about reducing toil, not replacing human judgment. The
automation handles the mechanical parts -- creating channels, paging people, posting updates,
tracking state -- so that the Incident Commander can focus on investigation and
decision-making. The key architectural insight is that the incident manager is a state
machine: each state transition triggers specific actions, and the transitions are governed
by rules that enforce the incident lifecycle. This pattern is extensible to any incident
response workflow.
