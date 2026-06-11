# Solution 02: Build an On-Call Rotation

## Part A: Primary and Secondary Rotation Design

An on-call rotation needs two layers: a primary responder who handles the
incident first, and a secondary who steps in if the primary does not respond.
The secondary is not a backup in the "they might not be needed" sense --
they are a critical safety net.

### Rotation Structure

```
         Primary Rotation (Weekly)
         ==========================
  Week 1: Alice    --> Mon 9am to Mon 9am (next week)
  Week 2: Bob      --> Mon 9am to Mon 9am (next week)
  Week 3: Charlie  --> Mon 9am to Mon 9am (next week)
  Week 4: Diana    --> Mon 9am to Mon 9am (next week)
  Week 5: (loops back to Alice)

         Secondary Rotation (Weekly, offset by 1)
         ==========================================
  Week 1: Bob      --> Mon 9am to Mon 9am (next week)
  Week 2: Charlie  --> Mon 9am to Mon 9am (next week)
  Week 3: Diana    --> Mon 9am to Mon 9am (next week)
  Week 4: Alice    --> Mon 9am to Mon 9am (next week)
  Week 5: (loops back to Bob)

         Overlap Visualization
         =====================
  Week    Primary    Secondary
  ----    -------    ---------
  1       Alice      Bob
  2       Bob        Charlie
  3       Charlie    Diana
  4       Diana      Alice

  Why offset by 1? The person who was primary last week is
  secondary this week. They have fresh context from the previous
  shift and can provide continuity.
```

### Why This Works

Offsetting primary and secondary by one position means the previous week's
primary becomes this week's secondary. They carry institutional knowledge from
the prior shift -- they know what issues were brewing, what changes were
deployed, and what to watch for. This is far more effective than random
pairing.

## Part B: PagerDuty YAML Schedule Configuration

```yaml
# schedules.yaml - PagerDuty Schedule Configuration
# ===================================================

schedules:
  - name: "Engineering Primary On-Call"
    time_zone: "America/New_York"
    description: "Primary on-call rotation for production incidents"
    layers:
      - name: "Weekly Primary Rotation"
        rotation_turn_length: 1_week
        start: "2024-01-01T09:00:00-05:00"
        users:
          - user: "Alice Chen"
            id: "PUSER001"
          - user: "Bob Martinez"
            id: "PUSER002"
          - user: "Charlie Kim"
            id: "PUSER003"
          - user: "Diana Patel"
            id: "PUSER004"
        restrictions:
          - type: "weekly"
            start_time_of_day: "09:00:00"
            end_time_of_day: "09:00:00"
            start_day_of_week: 1  # Monday
            duration_seconds: 604800  # 7 days

      - name: "After-Hours Override"
        rotation_turn_length: 1_day
        start: "2024-01-01T18:00:00-05:00"
        users:
          - user: "Eve Johnson"
            id: "PUSER005"
          - user: "Frank Lee"
            id: "PUSER006"
        restrictions:
          - type: "daily"
            start_time_of_day: "18:00:00"
            end_time_of_day: "09:00:00"
            duration_seconds: 54000  # 15 hours

    # Holiday coverage: override the regular rotation
    holiday_coverage:
      - name: "US Holidays 2024"
        holiday_ids:
          - "new_years_day"
          - "memorial_day"
          - "independence_day"
          - "labor_day"
          - "thanksgiving"
          - "christmas"
        coverage_user: "Frank Lee"
        backup_user: "Eve Johnson"

  - name: "Engineering Secondary On-Call"
    time_zone: "America/New_York"
    description: "Secondary on-call for escalation support"
    layers:
      - name: "Weekly Secondary Rotation"
        rotation_turn_length: 1_week
        start: "2024-01-08T09:00:00-05:00"  # Offset by 1 week
        users:
          - user: "Bob Martinez"
            id: "PUSER002"
          - user: "Charlie Kim"
            id: "PUSER003"
          - user: "Diana Patel"
            id: "PUSER004"
          - user: "Alice Chen"
            id: "PUSER001"
        restrictions:
          - type: "weekly"
            start_time_of_day: "09:00:00"
            end_time_of_day: "09:00:00"
            start_day_of_week: 1
            duration_seconds: 604800

escalation_policies:
  - name: "Production Incident Escalation"
    num_loops: 2
    escalation_delay_in_minutes: 5
    description: "Escalation path for production incidents"
    rules:
      - escalation_delay_in_minutes: 0
        targets:
          - type: "schedule"
            id: "engineering-primary-on-call"
            # Fires immediately to primary on-call

      - escalation_delay_in_minutes: 5
        targets:
          - type: "schedule"
            id: "engineering-secondary-on-call"
            # If primary does not ack in 5 min, page secondary

      - escalation_delay_in_minutes: 10
        targets:
          - type: "user"
            id: "PMGR001"  # Engineering Manager
            # If neither acks in 10 min, page manager

      - escalation_delay_in_minutes: 15
        targets:
          - type: "user"
            id: "VPENG001"  # VP of Engineering
            # If still no ack, page VP

  - name: "Database Team Escalation"
    num_loops: 1
    escalation_delay_in_minutes: 10
    rules:
      - escalation_delay_in_minutes: 0
        targets:
          - type: "schedule"
            id: "dba-primary-on-call"

      - escalation_delay_in_minutes: 10
        targets:
          - type: "user"
            id: "DBAMGR001"

# Timezone handling for distributed teams
timezone_schedules:
  - name: "Follow-the-Sun Americas"
    time_zone: "America/New_York"
    start: "09:00"
    end: "17:00"

  - name: "Follow-the-Sun EMEA"
    time_zone: "Europe/London"
    start: "09:00"
    end: "17:00"

  - name: "Follow-the-Sun APAC"
    time_zone: "Asia/Tokyo"
    start: "09:00"
    end: "17:00"

  # Handoff times (UTC):
  # Americas starts: 14:00 UTC (9am ET)
  # EMEA starts:     09:00 UTC (9am London)
  # APAC starts:     00:00 UTC (9am Tokyo)
  #
  # Coverage: APAC 00:00-09:00 UTC
  #           EMEA  09:00-14:00 UTC
  #           Americas 14:00-00:00 UTC
  #           (with overlap during handoffs)
```

### Why This Works

The configuration uses PagerDuty's layered schedule system. Each layer acts as
a fallback: if the primary layer's on-call person is unavailable (e.g., they
are in a restricted time window), the next layer applies. The after-hours
override layer ensures that night-time incidents go to a different set of
people who are prepared for off-hours work, rather than randomly waking up
whoever happens to be in the primary rotation.

## Part C: Escalation Policy Timing Rationale

```
  Escalation Timeline
  ===================

  0 min     5 min      15 min      30 min       60 min
  |---------|----------|-----------|------------|
  |         |          |           |            |
  v         v          v           v            v
  Page      Page       Page        Page         All-hands
  Primary   Secondary  Eng Mgr     VP Eng       war room

  Why 5 minutes for secondary?
  - PagerDuty sends push, SMS, email, and phone call
  - 5 minutes is enough for 2 full phone call attempts
  - Longer delays mean the primary might be unreachable (phone died, in a tunnel)

  Why 10 minutes for manager?
  - Gives primary + secondary a fair chance
  - Manager may need time to assess and mobilize additional resources
  - Manager is not expected to debug -- they coordinate

  Why 15 minutes for VP?
  - VP is a coordination role, not a technical role
  - They authorize cross-team resources, vendor escalation, customer comms
  - At 15 minutes, the incident is likely P1 and needs executive visibility
```

### Why This Works

The delays are deliberately short for technical roles and longer for management
roles. Engineers are expected to respond immediately; managers are expected to
coordinate. The escalation loop (`num_loops: 2`) means that after reaching
the VP, the cycle restarts from the primary -- in case the original on-call
engineer's phone had a temporary issue and they become available.

## Part D: Holiday Coverage Strategy

```python
# Holiday coverage logic
# =====================

HOLIDAYS_2024 = {
    "2024-01-01": "New Year's Day",
    "2024-05-27": "Memorial Day",
    "2024-07-04": "Independence Day",
    "2024-09-02": "Labor Day",
    "2024-11-28": "Thanksgiving",
    "2024-12-25": "Christmas Day",
}

def get_on_call_for_date(date_str, primary_schedule, holiday_schedule):
    """
    Holiday coverage takes priority over regular rotation.
    If the date is a holiday, use the designated holiday on-call person.
    Otherwise, use the regular rotation.
    """
    if date_str in HOLIDAYS_2024:
        return holiday_schedule.get_coverage(date_str)
    return primary_schedule.get_on_call(date_str)

# Why this works:
# 1. Holidays are defined centrally, so all schedules reference the same list
# 2. Holiday coverage is a dedicated person who volunteered and is compensated
# 3. The backup_user is paged if the holiday coverage person does not respond
# 4. Regular rotation resumes automatically after the holiday
```

### Why This Works

Holiday on-call is a separate concern from the regular rotation. The person
covering holidays has agreed in advance and receives additional compensation.
This prevents the situation where someone is unexpectedly on-call during a
holiday because the rotation happened to land on them. Dedicated holiday
coverage also means that person has prepared for it -- they have their laptop,
they are not traveling, and they are mentally ready to respond.

## Part E: Follow-the-Sun for Distributed Teams

```
  Follow-the-Sun Coverage Map (UTC)
  ==================================

  00:00  03:00  06:00  09:00  12:00  15:00  18:00  21:00  24:00
  |------|------|------|------|------|------|------|------|
  |<--- APAC (Tokyo) --->|                                 |
  |                |<--- EMEA (London) --->|               |
  |                |                |<--- Americas (NY) --->|
  |                |                |                      |
  |   Handoff:     |   Handoff:     |   Handoff:          |
  |   APAC->EMEA   |   EMEA->AMS    |   AMS->APAC         |

  Handoff Protocol:
  1. Outgoing team writes handoff document (issues, ongoing incidents, changes)
  2. Incoming team joins a 15-minute overlap call
  3. Outgoing team remains secondary for 1 hour after handoff
  4. All incidents during handoff are co-managed by both teams

  Benefits:
  - No one is woken up at 3am
  - Each team works during their normal business hours
  - Incidents get immediate attention regardless of time
```

### Why This Works

Follow-the-sun eliminates after-hours on-call entirely by distributing
coverage across time zones. The critical element is the handoff -- without a
structured handoff, the incoming team starts their shift blind. The 1-hour
overlap where the outgoing team remains as secondary prevents gaps during the
transition.

## Common Mistakes

1. **Not having a secondary rotation.** Teams often set up a primary rotation
   and forget the secondary. When the primary's phone dies or they are in a
   tunnel, no one gets paged until the escalation to management 15 minutes
   later. The secondary is cheap insurance against unreachable primaries.

2. **Same person on primary and secondary.** If Alice is primary this week and
   also secondary this week, you have no redundancy. The offset-by-one design
   ensures primary and secondary are always different people.

3. **Ignoring timezone restrictions in PagerDuty.** If your schedule says
   "9am to 9am" but does not specify the timezone, PagerDuty defaults to the
   account timezone. A distributed team member in Tokyo assigned to a schedule
   in America/New_York will be on-call at 3am their time without realizing it.

4. **No holiday coverage plan.** The rotation will blindly assign someone to
   holidays. This leads to resentment when a team member discovers they are
   on-call during Christmas. Proactive holiday coverage with volunteers and
   compensation prevents this.

5. **Escalation delays that are too long.** A 30-minute delay before paging
   the secondary means a P1 incident has 30 minutes of one person debugging
   alone. Shorter delays (5 minutes) ensure help arrives quickly. You can
   always de-escalate if the primary resolves it.

## Key Takeaway

An on-call rotation is a system, not a list of names. It needs primary and
secondary layers, timezone-aware scheduling, holiday coverage, and an
escalation policy with appropriate delays. The goal is simple: when something
breaks, the right person is paged within 5 minutes, every time, regardless
of time zone or day of year.
