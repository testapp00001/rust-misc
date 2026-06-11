# Solution 05: Automated Incident Response System

## Part A: System Architecture

An automated incident response system ties together three platforms:
PagerDuty for alerting, Slack for coordination, and a status page for
external communication. The system must handle the full lifecycle:
detection, notification, channel creation, status updates, and resolution.

```
  Architecture Overview
  =====================

  +----------------+     +-------------------+     +----------------+
  |   Monitoring   |     |  IncidentManager  |     |   Status Page  |
  |  (Prometheus,  |---->|    (Python)       |---->|   (Statuspage) |
  |   Datadog)     |     |                   |     |                |
  +----------------+     +---+-------+-------+     +----------------+
                             |       |
                    +--------+       +--------+
                    |                         |
             +------+------+          +-------+------+
             |  PagerDuty  |          |    Slack     |
             |   API v2    |          |   Web API    |
             +-------------+          +--------------+

  Flow:
  1. Monitoring detects anomaly -> fires webhook to IncidentManager
  2. IncidentManager creates PagerDuty incident
  3. IncidentManager creates Slack channel and invites responders
  4. IncidentManager updates status page
  5. IncidentManager posts initial timeline message
  6. Responder acknowledges in Slack -> IncidentManager updates PagerDuty
  7. Responder resolves in Slack -> IncidentManager closes all tickets
```

## Part B: Complete Python Implementation

```python
"""
IncidentManager - Automated Incident Response System

Integrates PagerDuty, Slack, and Statuspage for end-to-end
incident management automation.

Dependencies:
    pip install requests slack_sdk

Environment Variables:
    PAGERDUTY_API_KEY      - PagerDuty API v2 key
    PAGERDUTY_SERVICE_ID   - PagerDuty service ID for routing
    SLACK_BOT_TOKEN        - Slack bot OAuth token
    SLACK_TEAM_ID          - Slack workspace team ID
    STATUSPAGE_API_KEY     - Statuspage.io API key
    STATUSPAGE_PAGE_ID     - Statuspage page ID
"""

import os
import json
import logging
import time
from datetime import datetime, timezone
from dataclasses import dataclass, field
from enum import Enum
from typing import Optional

import requests
from slack_sdk import WebClient
from slack_sdk.errors import SlackApiError

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [%(levelname)s] %(name)s: %(message)s"
)
logger = logging.getLogger("IncidentManager")


class Severity(Enum):
    """Incident severity levels following P1-P4 convention."""
    P1 = "critical"
    P2 = "high"
    P3 = "medium"
    P4 = "low"

    @property
    def pagerduty_urgency(self) -> str:
        """Map severity to PagerDuty urgency."""
        return "high" if self in (Severity.P1, Severity.P2) else "low"

    @property
    def statuspage_impact(self) -> str:
        """Map severity to Statuspage impact level."""
        mapping = {
            Severity.P1: "critical",
            Severity.P2: "major",
            Severity.P3: "minor",
            Severity.P4: "maintenance",
        }
        return mapping[self]

    @property
    def update_interval_minutes(self) -> int:
        """How often to post updates based on severity."""
        mapping = {
            Severity.P1: 15,
            Severity.P2: 30,
            Severity.P3: 120,
            Severity.P4: 1440,  # daily
        }
        return mapping[self]


class IncidentStatus(Enum):
    """Incident lifecycle states."""
    INVESTIGATING = "investigating"
    IDENTIFIED = "identified"
    MONITORING = "monitoring"
    RESOLVED = "resolved"


@dataclass
class Incident:
    """Represents a single incident through its lifecycle."""
    title: str
    severity: Severity
    description: str
    affected_services: list[str]
    incident_id: Optional[str] = None
    pagerduty_incident_id: Optional[str] = None
    slack_channel_id: Optional[str] = None
    slack_channel_name: Optional[str] = None
    statuspage_incident_id: Optional[str] = None
    status: IncidentStatus = IncidentStatus.INVESTIGATING
    created_at: datetime = field(default_factory=lambda: datetime.now(timezone.utc))
    timeline: list[dict] = field(default_factory=list)
    ic_name: Optional[str] = None

    def add_timeline_entry(self, message: str, author: str = "system"):
        """Add an entry to the incident timeline."""
        entry = {
            "timestamp": datetime.now(timezone.utc).isoformat(),
            "author": author,
            "message": message,
        }
        self.timeline.append(entry)
        logger.info(f"Timeline [{self.incident_id}]: {message}")


class IncidentManager:
    """
    Manages the full incident lifecycle across PagerDuty, Slack,
    and Statuspage.

    Usage:
        manager = IncidentManager()
        incident = manager.create_incident(
            title="API Response Time Degradation",
            severity=Severity.P2,
            description="API p99 latency exceeding 5s threshold",
            affected_services=["payment-api", "web-frontend"],
        )
        manager.update_status(incident, IncidentStatus.IDENTIFIED,
                              "Root cause: database connection pool exhaustion")
        manager.resolve_incident(incident, "Rolled back to v2.14.2")
    """

    def __init__(self):
        """Initialize API clients from environment variables."""
        self.pagerduty_api_key = os.environ["PAGERDUTY_API_KEY"]
        self.pagerduty_service_id = os.environ["PAGERDUTY_SERVICE_ID"]
        self.slack_client = WebClient(token=os.environ["SLACK_BOT_TOKEN"])
        self.slack_team_id = os.environ["SLACK_TEAM_ID"]
        self.statuspage_api_key = os.environ["STATUSPAGE_API_KEY"]
        self.statuspage_page_id = os.environ["STATUSPAGE_PAGE_ID"]

        # PagerDuty API v2 base URL
        self.pagerduty_base = "https://api.pagerduty.com"
        # Statuspage API base URL
        self.statuspage_base = (
            f"https://api.statuspage.io/v1/pages/{self.statuspage_page_id}"
        )

        logger.info("IncidentManager initialized")

    def _pagerduty_headers(self) -> dict:
        """Return headers for PagerDuty API requests."""
        return {
            "Authorization": f"Token token={self.pagerduty_api_key}",
            "Accept": "application/vnd.pagerduty+json;version=2",
            "Content-Type": "application/json",
        }

    def _statuspage_headers(self) -> dict:
        """Return headers for Statuspage API requests."""
        return {
            "Authorization": f"OAuth {self.statuspage_api_key}",
            "Content-Type": "application/json",
        }

    # ----------------------------------------------------------------
    # Incident Creation
    # ----------------------------------------------------------------

    def create_incident(
        self,
        title: str,
        severity: Severity,
        description: str,
        affected_services: list[str],
        ic_name: Optional[str] = None,
    ) -> Incident:
        """
        Create a new incident and trigger the full response workflow.

        Steps:
        1. Create Incident object
        2. Create PagerDuty incident
        3. Create Slack channel
        4. Update status page
        5. Post initial messages to Slack

        Args:
            title: Short, descriptive incident title
            severity: P1-P4 severity level
            description: Detailed description of the issue
            affected_services: List of affected service names
            ic_name: Incident Commander name (optional)

        Returns:
            Fully initialized Incident object
        """
        incident = Incident(
            title=title,
            severity=severity,
            description=description,
            affected_services=affected_services,
            ic_name=ic_name,
        )

        # Generate incident ID
        timestamp = datetime.now(timezone.utc).strftime("%Y%m%d")
        incident.incident_id = f"INC-{timestamp}-{id(incident) % 10000:04d}"

        logger.info(
            f"Creating incident {incident.incident_id}: {title} "
            f"(severity={severity.name})"
        )
        incident.add_timeline_entry(
            f"Incident declared: {title}", author="system"
        )

        # Step 1: Create PagerDuty incident
        try:
            self._create_pagerduty_incident(incident)
        except Exception as e:
            logger.error(f"Failed to create PagerDuty incident: {e}")
            # Continue even if PagerDuty fails -- other systems still work

        # Step 2: Create Slack channel
        try:
            self._create_slack_channel(incident)
        except Exception as e:
            logger.error(f"Failed to create Slack channel: {e}")

        # Step 3: Update status page
        try:
            self._create_statuspage_incident(incident)
        except Exception as e:
            logger.error(f"Failed to update status page: {e}")

        # Step 4: Post initial Slack messages
        if incident.slack_channel_id:
            try:
                self._post_incident_header(incident)
                self._post_timeline_message(incident)
            except Exception as e:
                logger.error(f"Failed to post Slack messages: {e}")

        return incident

    # ----------------------------------------------------------------
    # PagerDuty Integration
    # ----------------------------------------------------------------

    def _create_pagerduty_incident(self, incident: Incident) -> None:
        """
        Create an incident in PagerDuty via API v2.

        The incident is routed to the service's escalation policy,
        which pages the appropriate on-call engineer.
        """
        payload = {
            "incident": {
                "type": "incident",
                "title": f"[{incident.severity.name}] {incident.title}",
                "service": {
                    "id": self.pagerduty_service_id,
                    "type": "service_reference",
                },
                "urgency": incident.severity.pagerduty_urgency,
                "body": {
                    "type": "incident_body",
                    "details": (
                        f"Incident ID: {incident.incident_id}\n"
                        f"Severity: {incident.severity.name}\n"
                        f"Affected Services: {', '.join(incident.affected_services)}\n"
                        f"\n{incident.description}"
                    ),
                },
                "escalation_policy": {
                    "id": os.environ.get(
                        "PAGERDUTY_ESCALATION_POLICY_ID",
                        "default"
                    ),
                    "type": "escalation_policy_reference",
                },
            }
        }

        response = requests.post(
            f"{self.pagerduty_base}/incidents",
            headers=self._pagerduty_headers(),
            json=payload,
            timeout=10,
        )
        response.raise_for_status()

        data = response.json()
        incident.pagerduty_incident_id = data["incident"]["id"]
        logger.info(
            f"PagerDuty incident created: {incident.pagerduty_incident_id}"
        )
        incident.add_timeline_entry(
            f"PagerDuty incident created: {incident.pagerduty_incident_id}",
            author="system",
        )

    def acknowledge_pagerduty(self, incident: Incident, user_email: str) -> None:
        """
        Acknowledge the PagerDuty incident on behalf of a user.

        Args:
            incident: The incident to acknowledge
            user_email: Email of the acknowledging user
        """
        if not incident.pagerduty_incident_id:
            logger.warning("No PagerDuty incident to acknowledge")
            return

        payload = {
            "incident": {
                "type": "incident",
                "status": "acknowledged",
            }
        }

        response = requests.put(
            f"{self.pagerduty_base}/incidents/{incident.pagerduty_incident_id}",
            headers=self._pagerduty_headers(),
            json=payload,
            timeout=10,
        )
        response.raise_for_status()

        logger.info(f"PagerDuty incident acknowledged by {user_email}")
        incident.add_timeline_entry(
            f"Incident acknowledged by {user_email}", author=user_email
        )

    def resolve_pagerduty(self, incident: Incident, resolution_note: str) -> None:
        """
        Resolve the PagerDuty incident with a resolution note.

        Args:
            incident: The incident to resolve
            resolution_note: Description of how the incident was resolved
        """
        if not incident.pagerduty_incident_id:
            logger.warning("No PagerDuty incident to resolve")
            return

        # Add resolution note first
        note_payload = {
            "note": {
                "content": resolution_note,
            }
        }
        requests.post(
            f"{self.pagerduty_base}/incidents/{incident.pagerduty_incident_id}/notes",
            headers=self._pagerduty_headers(),
            json=note_payload,
            timeout=10,
        )

        # Resolve the incident
        payload = {
            "incident": {
                "type": "incident",
                "status": "resolved",
            }
        }

        response = requests.put(
            f"{self.pagerduty_base}/incidents/{incident.pagerduty_incident_id}",
            headers=self._pagerduty_headers(),
            json=payload,
            timeout=10,
        )
        response.raise_for_status()

        logger.info("PagerDuty incident resolved")

    # ----------------------------------------------------------------
    # Slack Integration
    # ----------------------------------------------------------------

    def _create_slack_channel(self, incident: Incident) -> None:
        """
        Create a dedicated Slack channel for the incident.

        Channel naming convention: #inc-YYYYMMDD-short-title
        The channel is created as a public channel so anyone can
        join to follow the incident.
        """
        # Sanitize title for channel name (Slack requires lowercase, no spaces)
        sanitized_title = (
            incident.title.lower()
            .replace(" ", "-")
            .replace("_", "-")
        )
        # Truncate to fit Slack's 80-char channel name limit
        date_str = datetime.now(timezone.utc).strftime("%Y%m%d")
        max_title_len = 80 - len("inc-") - len(date_str) - 1
        sanitized_title = sanitized_title[:max_title_len]

        channel_name = f"inc-{date_str}-{sanitized_title}"

        response = self.slack_client.conversations_create(
            name=channel_name,
            is_private=False,
        )

        incident.slack_channel_id = response["channel"]["id"]
        incident.slack_channel_name = channel_name

        logger.info(f"Slack channel created: #{channel_name}")
        incident.add_timeline_entry(
            f"Slack channel created: #{channel_name}", author="system"
        )

        # Invite relevant people to the channel
        self._invite_responders(incident)

    def _invite_responders(self, incident: Incident) -> None:
        """
        Invite on-call engineers and IC to the incident channel.

        Looks up users by their PagerDuty escalation or by name.
        """
        # In a real system, you would look up the current on-call
        # engineers from PagerDuty's on-call API
        try:
            # Get list of users to invite (customize per your org)
            on_call_users = self._get_on_call_users()

            user_ids = []
            for email in on_call_users:
                try:
                    result = self.slack_client.users_lookupByEmail(email=email)
                    user_ids.append(result["user"]["id"])
                except SlackApiError as e:
                    logger.warning(f"Could not find Slack user for {email}: {e}")

            if user_ids:
                self.slack_client.conversations_invite(
                    channel=incident.slack_channel_id,
                    users=",".join(user_ids),
                )
                logger.info(f"Invited {len(user_ids)} responders to channel")
        except Exception as e:
            logger.error(f"Failed to invite responders: {e}")

    def _get_on_call_users(self) -> list[str]:
        """
        Fetch current on-call users from PagerDuty.

        Returns:
            List of email addresses for current on-call engineers
        """
        response = requests.get(
            f"{self.pagerduty_base}/oncalls",
            headers=self._pagerduty_headers(),
            params={
                "escalation_policy_ids[]": os.environ.get(
                    "PAGERDUTY_ESCALATION_POLICY_ID", ""
                ),
                "earliest": True,
            },
            timeout=10,
        )
        response.raise_for_status()

        emails = []
        for oncall in response.json().get("oncalls", []):
            user = oncall.get("user", {})
            email = user.get("email")
            if email:
                emails.append(email)

        return emails

    def _post_incident_header(self, incident: Incident) -> None:
        """
        Post the incident header block to the Slack channel.

        This is the first message in the channel, formatted as a
        clear incident declaration.
        """
        header_text = (
            f":rotating_light: *INCIDENT DECLARED: {incident.severity.name}* "
            f":rotating_light:\n\n"
            f"*Title:* {incident.title}\n"
            f"*Severity:* {incident.severity.name} ({incident.severity.value})\n"
            f"*Incident ID:* {incident.incident_id}\n"
            f"*Affected Services:* {', '.join(incident.affected_services)}\n"
            f"*Incident Commander:* {incident.ic_name or 'TBD'}\n"
            f"*Started:* {incident.created_at.strftime('%Y-%m-%d %H:%M UTC')}\n\n"
            f"*Description:* {incident.description}\n\n"
            f"Use this channel for all incident communication. "
            f"React with :eyes: to acknowledge."
        )

        self.slack_client.chat_postMessage(
            channel=incident.slack_channel_id,
            text=header_text,
            unfurl_links=False,
        )

    def _post_timeline_message(self, incident: Incident) -> None:
        """Post the current timeline to the Slack channel."""
        if not incident.timeline:
            return

        timeline_text = "*Timeline:*\n"
        for entry in incident.timeline:
            ts = entry["timestamp"][:19]  # Trim microseconds
            timeline_text += f"> [{ts}] {entry['message']} — _{entry['author']}_\n"

        self.slack_client.chat_postMessage(
            channel=incident.slack_channel_id,
            text=timeline_text,
            unfurl_links=False,
        )

    def post_slack_update(
        self,
        incident: Incident,
        message: str,
        author: str = "system",
    ) -> None:
        """
        Post an update to the incident Slack channel and add to timeline.

        Args:
            incident: The incident to update
            message: Update message
            author: Who is posting the update
        """
        incident.add_timeline_entry(message, author=author)

        timestamp = datetime.now(timezone.utc).strftime("%H:%M UTC")
        formatted_message = f"[{timestamp}] {message}"

        try:
            self.slack_client.chat_postMessage(
                channel=incident.slack_channel_id,
                text=formatted_message,
                unfurl_links=False,
            )
        except SlackApiError as e:
            logger.error(f"Failed to post Slack update: {e}")

    # ----------------------------------------------------------------
    # Status Page Integration
    # ----------------------------------------------------------------

    def _create_statuspage_incident(self, incident: Incident) -> None:
        """
        Create an incident on Statuspage.io.

        This publishes a public-facing incident that customers can see
        on the status page.
        """
        # Map affected service names to Statuspage component IDs
        # In production, this mapping would be stored in a config file
        component_ids = self._resolve_component_ids(incident.affected_services)

        payload = {
            "incident": {
                "name": f"[{incident.severity.name}] {incident.title}",
                "status": "investigating",
                "impact": incident.severity.statuspage_impact,
                "body": incident.description,
                "component_ids": component_ids,
                "metadata": {
                    "incident_id": incident.incident_id,
                },
            }
        }

        response = requests.post(
            f"{self.statuspage_base}/incidents",
            headers=self._statuspage_headers(),
            json=payload,
            timeout=10,
        )
        response.raise_for_status()

        data = response.json()
        incident.statuspage_incident_id = data["incident"]["id"]
        logger.info(
            f"Statuspage incident created: {incident.statuspage_incident_id}"
        )
        incident.add_timeline_entry(
            "Status page updated: Investigating", author="system"
        )

    def _resolve_component_ids(self, service_names: list[str]) -> list[str]:
        """
        Map service names to Statuspage component IDs.

        In production, this would query the Statuspage API for
        components or use a configuration file.
        """
        # Placeholder mapping -- replace with actual component IDs
        service_to_component = {
            "payment-api": "comp_payment_001",
            "web-frontend": "comp_web_001",
            "auth-service": "comp_auth_001",
            "search-service": "comp_search_001",
        }
        return [
            service_to_component[name]
            for name in service_names
            if name in service_to_component
        ]

    def update_statuspage(
        self,
        incident: Incident,
        new_status: IncidentStatus,
        message: str,
    ) -> None:
        """
        Update the Statuspage incident status and post a message.

        Args:
            incident: The incident to update
            new_status: New status (investigating, identified, monitoring, resolved)
            message: Status update message for customers
        """
        if not incident.statuspage_incident_id:
            logger.warning("No Statuspage incident to update")
            return

        payload = {
            "incident": {
                "status": new_status.value,
                "body": message,
            }
        }

        response = requests.put(
            f"{self.statuspage_base}/incidents/{incident.statuspage_incident_id}",
            headers=self._statuspage_headers(),
            json=payload,
            timeout=10,
        )
        response.raise_for_status()

        incident.status = new_status
        logger.info(f"Statuspage updated to: {new_status.value}")
        incident.add_timeline_entry(
            f"Status page updated: {new_status.value}", author="system"
        )

    # ----------------------------------------------------------------
    # Status Management
    # ----------------------------------------------------------------

    def update_status(
        self,
        incident: Incident,
        new_status: IncidentStatus,
        message: str,
        author: str = "system",
        root_cause: Optional[str] = None,
    ) -> None:
        """
        Update incident status across all platforms.

        This is the primary method for moving an incident through
        its lifecycle. It updates PagerDuty, Slack, and Statuspage
        in a single call.

        Args:
            incident: The incident to update
            new_status: New status
            message: Human-readable update message
            author: Who initiated the update
            root_cause: If status is IDENTIFIED, the root cause description
        """
        incident.status = new_status

        # Update PagerDuty
        if incident.pagerduty_incident_id:
            try:
                self._update_pagerduty_status(incident, new_status)
            except Exception as e:
                logger.error(f"Failed to update PagerDuty status: {e}")

        # Update Slack
        if incident.slack_channel_id:
            try:
                status_emoji = {
                    IncidentStatus.INVESTIGATING: ":mag:",
                    IncidentStatus.IDENTIFIED: ":dart:",
                    IncidentStatus.MONITORING: ":eyes:",
                    IncidentStatus.RESOLVED: ":white_check_mark:",
                }
                emoji = status_emoji.get(new_status, ":information_source:")
                update_msg = f"{emoji} *Status: {new_status.value.upper()}* — {message}"
                if root_cause:
                    update_msg += f"\n*Root Cause:* {root_cause}"
                self.post_slack_update(incident, update_msg, author=author)
            except Exception as e:
                logger.error(f"Failed to post Slack status update: {e}")

        # Update Statuspage
        try:
            self.update_statuspage(incident, new_status, message)
        except Exception as e:
            logger.error(f"Failed to update Statuspage: {e}")

    def _update_pagerduty_status(
        self, incident: Incident, new_status: IncidentStatus
    ) -> None:
        """Map incident status to PagerDuty status and update."""
        status_mapping = {
            IncidentStatus.INVESTIGATING: "acknowledged",
            IncidentStatus.IDENTIFIED: "acknowledged",
            IncidentStatus.MONITORING: "acknowledged",
            IncidentStatus.RESOLVED: "resolved",
        }

        pd_status = status_mapping.get(new_status, "acknowledged")

        payload = {
            "incident": {
                "type": "incident",
                "status": pd_status,
            }
        }

        response = requests.put(
            f"{self.pagerduty_base}/incidents/{incident.pagerduty_incident_id}",
            headers=self._pagerduty_headers(),
            json=payload,
            timeout=10,
        )
        response.raise_for_status()

    # ----------------------------------------------------------------
    # Resolution
    # ----------------------------------------------------------------

    def resolve_incident(
        self,
        incident: Incident,
        resolution_summary: str,
        author: str = "system",
    ) -> None:
        """
        Resolve an incident across all platforms.

        This is the final step in the incident lifecycle. It:
        1. Updates status to RESOLVED on all platforms
        2. Resolves the PagerDuty incident
        3. Archives the Slack channel topic
        4. Posts the final timeline

        Args:
            incident: The incident to resolve
            resolution_summary: Summary of how the incident was resolved
            author: Who resolved the incident
        """
        logger.info(f"Resolving incident {incident.incident_id}")

        # Update status across all platforms
        self.update_status(
            incident,
            IncidentStatus.RESOLVED,
            resolution_summary,
            author=author,
        )

        # Resolve PagerDuty with resolution note
        try:
            self.resolve_pagerduty(incident, resolution_summary)
        except Exception as e:
            logger.error(f"Failed to resolve PagerDuty incident: {e}")

        # Post final timeline to Slack
        if incident.slack_channel_id:
            try:
                self._post_final_summary(incident, resolution_summary)
            except Exception as e:
                logger.error(f"Failed to post final summary: {e}")

        # Update Slack channel topic to indicate resolution
        if incident.slack_channel_id:
            try:
                self.slack_client.conversations_setTopic(
                    channel=incident.slack_channel_id,
                    topic=f"RESOLVED — {resolution_summary[:200]}",
                )
            except SlackApiError as e:
                logger.error(f"Failed to update channel topic: {e}")

        logger.info(f"Incident {incident.incident_id} resolved")

    def _post_final_summary(
        self, incident: Incident, resolution_summary: str
    ) -> None:
        """Post the final incident summary to Slack."""
        duration = datetime.now(timezone.utc) - incident.created_at
        duration_minutes = int(duration.total_seconds() / 60)

        summary = (
            f":white_check_mark: *INCIDENT RESOLVED*\n\n"
            f"*Incident ID:* {incident.incident_id}\n"
            f"*Duration:* {duration_minutes} minutes\n"
            f"*Resolution:* {resolution_summary}\n\n"
            f"*Full Timeline:*\n"
        )

        for entry in incident.timeline:
            ts = entry["timestamp"][:19]
            summary += f"> [{ts}] {entry['message']}\n"

        summary += (
            f"\n*Next Steps:*\n"
            f"> 1. Post-incident review to be scheduled within 48 hours\n"
            f"> 2. Follow-up tickets to be created\n"
            f"> 3. This channel will be archived\n"
        )

        self.slack_client.chat_postMessage(
            channel=incident.slack_channel_id,
            text=summary,
            unfurl_links=False,
        )


# ----------------------------------------------------------------
# Example Usage
# ----------------------------------------------------------------

def main():
    """Demonstrate the IncidentManager with a sample incident."""
    manager = IncidentManager()

    # Create incident
    incident = manager.create_incident(
        title="API Response Time Degradation",
        severity=Severity.P2,
        description=(
            "API p99 latency has increased from 200ms to 8.2s. "
            "Error rate is at 12%. Database connection pool appears "
            "exhausted. Affects all API endpoints."
        ),
        affected_services=["payment-api", "web-frontend"],
        ic_name="Alice Chen",
    )

    # Simulate investigation
    time.sleep(2)
    manager.post_slack_update(
        incident,
        "Investigating: Checking pod logs and deployment history",
        author="bob-martinez",
    )

    # Identify root cause
    time.sleep(2)
    manager.update_status(
        incident,
        IncidentStatus.IDENTIFIED,
        "Root cause found: deployment v2.14.3 introduced a migration "
        "that references a dropped table. Pods cannot start.",
        author="bob-martinez",
        root_cause="Migration 20240315_add_payment_index references "
                   "payments_old table which was dropped in cleanup.",
    )

    # Monitor after fix
    time.sleep(2)
    manager.update_status(
        incident,
        IncidentStatus.MONITORING,
        "Rollback to v2.14.2 completed. All pods healthy. "
        "Monitoring for 30 minutes.",
        author="bob-martinez",
    )

    # Resolve
    time.sleep(2)
    manager.resolve_incident(
        incident,
        resolution_summary=(
            "Rolled back deployment from v2.14.3 to v2.14.2. "
            "All services restored to normal operation. "
            "Follow-up: add migration validation to CI/CD pipeline."
        ),
        author="alice-chen",
    )


if __name__ == "__main__":
    main()
```

## Part C: Error Handling Strategy

### Why This Works

The implementation wraps every external API call in try/except blocks. This is
deliberate: during an incident, you cannot afford to have the automation fail
because one integration is down. If PagerDuty is unreachable, the Slack channel
still gets created. If Statuspage is down, the PagerDuty incident still fires.
Each failure is logged but does not block the rest of the workflow.

```
  Error Handling Flow
  ===================

  create_incident()
       |
       +-- _create_pagerduty_incident()
       |    [fails?] -> log error, continue
       |
       +-- _create_slack_channel()
       |    [fails?] -> log error, continue
       |
       +-- _create_statuspage_incident()
       |    [fails?] -> log error, continue
       |
       +-- _post_incident_header()
            [fails?] -> log error, continue

  Result: Even if 2 of 3 integrations fail, the remaining one
          is working. The incident is still partially automated.
          A human can manually handle the failed integrations.
```

### Retry Strategy

```python
def _retry_with_backoff(self, func, max_retries=3, base_delay=1):
    """
    Retry a function with exponential backoff.

    Used for transient failures like network timeouts or
    rate limiting (HTTP 429).
    """
    for attempt in range(max_retries):
        try:
            return func()
        except requests.exceptions.HTTPError as e:
            if e.response.status_code == 429:
                # Rate limited -- respect Retry-After header
                retry_after = int(
                    e.response.headers.get("Retry-After", base_delay * (2 ** attempt))
                )
                logger.warning(f"Rate limited, retrying in {retry_after}s")
                time.sleep(retry_after)
            elif e.response.status_code >= 500:
                # Server error -- retry with backoff
                delay = base_delay * (2 ** attempt)
                logger.warning(f"Server error, retrying in {delay}s")
                time.sleep(delay)
            else:
                # Client error (4xx) -- do not retry
                raise
        except requests.exceptions.Timeout:
            delay = base_delay * (2 ** attempt)
            logger.warning(f"Timeout, retrying in {delay}s")
            time.sleep(delay)

    raise Exception(f"Failed after {max_retries} retries")
```

### Why This Works

The retry strategy distinguishes between transient failures (timeout, 429, 5xx)
and permanent failures (4xx). Retrying a 400 Bad Request is pointless -- the
payload is wrong. Retrying a 503 Service Unavailable might work 10 seconds
later. The exponential backoff prevents hammering a struggling API.

## Part D: Testing the System

```python
"""
Unit tests for IncidentManager.

Run with: pytest test_incident_manager.py -v
"""

import pytest
from unittest.mock import Mock, patch, MagicMock
from incident_manager import IncidentManager, Incident, Severity, IncidentStatus


@pytest.fixture
def manager():
    """Create an IncidentManager with mocked environment variables."""
    with patch.dict("os.environ", {
        "PAGERDUTY_API_KEY": "test-pd-key",
        "PAGERDUTY_SERVICE_ID": "test-service-id",
        "SLACK_BOT_TOKEN": "test-slack-token",
        "SLACK_TEAM_ID": "test-team-id",
        "STATUSPAGE_API_KEY": "test-sp-key",
        "STATUSPAGE_PAGE_ID": "test-page-id",
    }):
        return IncidentManager()


@pytest.fixture
def sample_incident():
    """Create a sample Incident for testing."""
    return Incident(
        title="Test Incident",
        severity=Severity.P2,
        description="Test description",
        affected_services=["test-service"],
    )


class TestSeverity:
    """Test severity level properties."""

    def test_p1_is_high_urgency(self):
        assert Severity.P1.pagerduty_urgency == "high"

    def test_p3_is_low_urgency(self):
        assert Severity.P3.pagerduty_urgency == "low"

    def test_p1_is_critical_impact(self):
        assert Severity.P1.statuspage_impact == "critical"

    def test_update_intervals(self):
        assert Severity.P1.update_interval_minutes == 15
        assert Severity.P2.update_interval_minutes == 30
        assert Severity.P3.update_interval_minutes == 120
        assert Severity.P4.update_interval_minutes == 1440


class TestIncident:
    """Test Incident dataclass behavior."""

    def test_add_timeline_entry(self, sample_incident):
        sample_incident.add_timeline_entry("Test message", author="tester")
        assert len(sample_incident.timeline) == 1
        assert sample_incident.timeline[0]["message"] == "Test message"
        assert sample_incident.timeline[0]["author"] == "tester"

    def test_default_status(self, sample_incident):
        assert sample_incident.status == IncidentStatus.INVESTIGATING

    def test_incident_id_not_set_by_default(self, sample_incident):
        assert sample_incident.incident_id is None


class TestIncidentManager:
    """Test IncidentManager integration methods."""

    @patch("incident_manager.requests.post")
    @patch.object(IncidentManager, "_create_slack_channel")
    @patch.object(IncidentManager, "_create_statuspage_incident")
    @patch.object(IncidentManager, "_post_incident_header")
    @patch.object(IncidentManager, "_post_timeline_message")
    def test_create_incident_calls_all_integrations(
        self, mock_timeline, mock_header, mock_sp, mock_slack, mock_post, manager
    ):
        mock_post.return_value = Mock(
            json=lambda: {"incident": {"id": "pd-123"}},
            raise_for_status=Mock(),
        )

        incident = manager.create_incident(
            title="Test",
            severity=Severity.P1,
            description="Test desc",
            affected_services=["svc-1"],
        )

        mock_post.assert_called_once()
        mock_slack.assert_called_once()
        mock_sp.assert_called_once()
        mock_header.assert_called_once()
        assert incident.incident_id is not None

    @patch("incident_manager.requests.post")
    def test_create_incident_continues_on_pagerduty_failure(
        self, mock_post, manager
    ):
        """Incident creation should not fail if PagerDuty is down."""
        mock_post.side_effect = Exception("PagerDuty unavailable")

        # Should not raise
        with patch.object(IncidentManager, "_create_slack_channel"):
            with patch.object(IncidentManager, "_create_statuspage_incident"):
                with patch.object(IncidentManager, "_post_incident_header"):
                    with patch.object(IncidentManager, "_post_timeline_message"):
                        incident = manager.create_incident(
                            title="Test",
                            severity=Severity.P2,
                            description="Test",
                            affected_services=["svc-1"],
                        )
                        assert incident.incident_id is not None
```

### Why This Works

The tests mock external API calls to verify that the IncidentManager calls the
right APIs in the right order, and that failures in one integration do not
block others. The critical test is `test_create_incident_continues_on_pagerduty_failure`
-- this proves that the error handling strategy works. In production, you would
also add integration tests that hit real sandbox APIs.

## Common Mistakes

1. **Not handling API rate limits.** PagerDuty and Slack both enforce rate
   limits. During a major incident, you might create dozens of updates in
   quick succession. Without rate limit handling (429 responses with
   Retry-After headers), your automation will fail at the worst possible
   time.

2. **Tying all integrations together with hard dependencies.** If your code
   requires PagerDuty to succeed before creating the Slack channel, a
   PagerDuty outage blocks your entire response. Each integration should be
   independent -- the failure of one must not prevent the others from working.

3. **Not preserving the timeline across restarts.** If your IncidentManager
   process crashes and restarts, the in-memory timeline is lost. Persist the
   timeline to a database or file so you can reconstruct it. The timeline is
   the most valuable artifact from an incident.

4. **Hardcoding component IDs and service mappings.** The mapping from service
   names to Statuspage component IDs should be in configuration, not code.
   When a new service is added, you should not need to redeploy the
   IncidentManager.

5. **Not testing the automation before you need it.** Run the IncidentManager
   against sandbox/staging environments regularly. The worst time to discover
   that your PagerDuty API key is expired is during a P1 incident at 2 AM.
   Schedule monthly "fire drills" that exercise the full automation path.

## Key Takeaway

Automated incident response is about removing mechanical steps from the
incident process so humans can focus on debugging. The IncidentManager
handles channel creation, status page updates, and PagerDuty routing --
tasks that are important but do not require human judgment. The key design
principle is resilience: every integration can fail independently, and the
system continues to function with whatever is available.
