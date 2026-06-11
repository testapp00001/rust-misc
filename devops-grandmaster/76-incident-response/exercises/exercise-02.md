# Exercise 02: Build an On-Call Rotation

**Type:** Guided
**Time:** 30 minutes
**Difficulty:** Easy-Medium

## Objective

Create a production-ready on-call rotation schedule in YAML that defines primary and secondary
rotations, escalation policies, timezone-aware handoffs, and override rules. This exercise
translates incident response theory into a concrete PagerDuty-compatible configuration.

## Prerequisites

- Completion of Exercise 01 (understanding of severity levels).
- Basic familiarity with YAML syntax.
- Understanding of timezone concepts (UTC offsets, DST).

## Scenario

**Acme Corp** has a 6-person platform engineering team distributed across three timezones:

```
┌──────────────────────────────────────────────────────────────────┐
│                    ACME CORP PLATFORM TEAM                       │
├──────────┬──────────────┬────────────────┬───────────────────────┤
│ Engineer │ Location     │ Timezone       │ Skills                │
├──────────┼──────────────┼────────────────┼───────────────────────┤
│ Alice    │ New York     │ America/New_York│ Kubernetes, Networking│
│ Bob      │ New York     │ America/New_York│ Databases, SRE        │
│ Carlos   │ London       │ Europe/London  │ Kubernetes, Security  │
│ Diana    │ London       │ Europe/London  │ Databases, Monitoring │
│ Eve      │ Tokyo        │ Asia/Tokyo     │ Networking, SRE       │
│ Frank    │ Tokyo        │ Asia/Tokyo     │ Security, Monitoring  │
└──────────┴──────────────┴────────────────┴───────────────────────┘
```

The team wants follow-the-sun coverage with handoffs at 08:00 local time in each timezone.
Primary on-call is always in the engineer's local timezone. Secondary on-call covers the
overlapping gap between timezone shifts.

## Tasks

### Part A: Define the Team Roster YAML

Create a YAML file that defines each team member with their timezone, contact methods, and
skills. Use the following structure:

```yaml
# oncall-roster.yaml
team:
  name: "platform-engineering"
  members:
    - name: "Alice"
      # ... fill in the rest
```

<details><summary>Hint</summary>
Each member needs at minimum: name, timezone (IANA format), email, phone (E.164 format),
and a skills array. PagerDuty uses IANA timezone identifiers like `America/New_York`, not
abbreviations like `EST`.
</details>

### Part B: Create the Rotation Schedule

Define a rotation schedule that implements follow-the-sun coverage. The rotation should have:

1. A primary on-call rotation with 12-hour shifts.
2. A secondary on-call rotation that covers the handoff gaps.
3. Three timezone groups: Americas, EMEA, APAC.

Write the rotation YAML:

```yaml
schedules:
  primary:
    name: "Primary On-Call"
    time_zone: "UTC"
    layers:
      - name: "Americas Shift"
        # ... 08:00-20:00 America/New_York
      - name: "EMEA Shift"
        # ... 08:00-20:00 Europe/London
      - name: "APAC Shift"
        # ... 08:00-20:00 Asia/Tokyo
  secondary:
    name: "Secondary On-Call"
    # ... define the secondary rotation
```

<details><summary>Hint</summary>
Convert each local 08:00 to UTC for the schedule definition. America/New_York (EST) is UTC-5,
Europe/London (GMT) is UTC+0, Asia/Tokyo (JST) is UTC+9. Remember DST adjustments:
America/New_York shifts to UTC-4 during summer months.
</details>

### Part C: Define the Escalation Policy

Create an escalation policy that defines what happens when an incident is triggered and not
acknowledged. The policy should include:

1. Alert the primary on-call (wait 5 minutes).
2. Escalate to secondary on-call (wait 5 minutes).
3. Escalate to team lead (wait 10 minutes).
4. Escalate to VP of Engineering (final).

```yaml
escalation_policy:
  name: "Platform Engineering Escalation"
  num_loops: 2
  rules:
    - severity: "P1"
      targets:
        # ... define escalation targets
    - severity: "P2"
      targets:
        # ... define escalation targets
```

<details><summary>Hint</summary>
PagerDuty escalation policies have ordered rules with delay_in_minutes between each step.
P1 incidents should escalate faster than P2. Consider whether P3/P4 incidents need an
escalation policy at all -- they might just create a ticket.
</details>

### Part D: Handle Overrides and Vacation

Write the YAML configuration for:

1. A temporary override when Alice goes on vacation for two weeks.
2. A manual override when Bob swaps his on-call shift with Carlos.
3. A restriction rule that prevents the same engineer from being primary on-call for more
   than 7 consecutive days.

```yaml
overrides:
  vacation:
    # ... Alice's vacation override
  swaps:
    # ... Bob-Carlos swap
restrictions:
  max_consecutive_days: 7
  # ... additional restriction rules
```

<details><summary>Hint</summary>
PagerDuty overrides replace the scheduled person for a specific time window. Vacation
overrides should auto-assign the next available engineer in the same timezone group.
Swaps are bidirectional -- both engineers must confirm.
</details>

### Part E: Validate with a Dry Run

Simulate one week of on-call coverage by creating a table that shows who is primary and
secondary on-call for each 12-hour block, Monday through Sunday. Include timezone
conversions so it is clear when handoffs occur.

```
| Day       | UTC Window       | Primary   | Secondary | Local Handoff        |
|-----------|------------------|-----------|-----------|----------------------|
| Monday    | 00:00-12:00      | ???       | ???       | 08:00 ??? (UTC+9)    |
| Monday    | 12:00-24:00      | ???       | ???       | ...                  |
| ...       | ...              | ...       | ...       | ...                  |
```

<details><summary>Hint</summary>
Start by mapping each timezone's 08:00 local to UTC:
- Asia/Tokyo 08:00 = UTC 23:00 (previous day)
- Europe/London 08:00 = UTC 08:00
- America/New_York 08:00 = UTC 13:00 (EST) or UTC 12:00 (EDT)

Then fill in the table by assigning engineers to their timezone group's shift.
</details>

## Success Criteria

- [ ] The roster YAML is valid and includes all 6 engineers with correct IANA timezones.
- [ ] The rotation schedule provides 24/7 coverage with no gaps.
- [ ] Escalation policies differ by severity (P1 escalates faster than P2).
- [ ] Override rules handle vacations and swaps correctly.
- [ ] The dry-run table shows correct UTC-to-local-time handoff conversions.

## What You Should Understand After This Exercise

On-call rotations are more than just a list of names. A well-designed rotation accounts for
timezone differences, handoff overlaps, escalation policies, and human factors like vacation
and fatigue. The YAML configuration is the source of truth that automated tools like PagerDuty
use to route alerts -- getting it right means the right person gets paged at the right time.
