# Solution 02: 5 Whys Root Cause Analysis

## Exercise Recap

Perform a complete 5 Whys analysis starting from the symptom "Users could
not log in for 47 minutes" and trace it to a systemic root cause. Then
explain why stopping at "human error" is insufficient.

## Complete 5 Whys Chain

```
SYMPTOM: Users could not log in for 47 minutes.
         |
         v
WHY 1: Why could users not log in?
         |
         +---> The authentication service was returning 503 errors.
         |     The service depends on a session store (Redis) that was
         |     unreachable. All login attempts failed because the auth
         |     service could not validate or create sessions.
         |
         v
WHY 2: Why was the session store unreachable?
         |
         +---> The Redis cluster failed over to a replica that did not
         |     have the correct security group rules. The failover
         |     triggered automatically when the primary node was
         |     terminated during a scheduled maintenance window.
         |
         v
WHY 3: Why did the replica have incorrect security group rules?
         |
         +---> The Redis replica was provisioned by a different
         |     CloudFormation stack than the primary. The primary's
         |     security group was updated when the networking team
         |     made changes last quarter, but the replica's security
         |     group was in a separate stack and was not updated.
         |
         v
WHY 4: Why were the primary and replica in different CloudFormation stacks?
         |
         +---> The Redis cluster was originally a single-node setup.
         |     When it was upgraded to a replicated setup for HA, the
         |     replica was added manually to the existing stack by
         |     editing the template. The engineer used a separate
         |     resource block rather than a Redis ReplicationGroup
         |     resource because the existing template structure did not
         |     support ReplicationGroup without a full rewrite.
         |
         v
WHY 5: Why was the Redis cluster managed as a manually-edited
       CloudFormation template rather than a managed IaC module?
         |
         +---> There is no infrastructure module library or golden path
               for provisioning stateful services (databases, caches,
               message queues). Each team creates their own templates,
               leading to inconsistent patterns, drift between
               environments, and manual edits that bypass review.
```

## Root Cause Statement

The organization lacks a standardized infrastructure module library for
stateful services, leading to inconsistent provisioning patterns where
related resources (primary and replica) end up in separate management
stacks. This creates configuration drift that goes undetected until a
failover event exposes the inconsistency.

## How Each "Why" Peels Back a Layer

```
Layer Analysis:

WHY 1 -- Symptom Layer
  What:     Service returned errors
  Domain:   Application behavior
  Type:     Observable failure
  Action:   Immediate mitigation (restart, fail back, etc.)

WHY 2 -- Trigger Layer
  What:     Automatic failover exposed a misconfigured replica
  Domain:   Infrastructure behavior
  Type:     Triggering event
  Action:   Fix the immediate misconfiguration

WHY 3 -- Configuration Layer
  What:     Security groups were inconsistent between primary and replica
  Domain:   Configuration management
  Type:     Configuration drift
  Action:   Align configurations and add drift detection

WHY 4 -- Process Layer
  What:     Manual template editing created structural inconsistency
  Domain:   Engineering practices
  Type:     Process gap
  Action:   Enforce IaC review processes and template standards

WHY 5 -- Systemic Layer
  What:     No golden paths for stateful service provisioning
  Domain:   Organizational practices
  Type:     Systemic gap
  Action:   Build infrastructure module library with enforced patterns
```

Each layer reveals a different type of problem:

| Layer | Focus | Fix Duration | Recurrence Prevention |
|-------|-------|-------------|----------------------|
| Symptom | What broke | Minutes | None -- same thing will break again |
| Trigger | What caused it to break now | Hours | Low -- a different trigger will cause the same break |
| Configuration | What was wrong | Days | Medium -- will drift again without automation |
| Process | How the wrong thing happened | Weeks | High -- process changes affect future work |
| Systemic | Why the process existed | Months | Highest -- structural changes prevent entire classes of failures |

## Why Stopping at "Human Error" Is Insufficient

### The "Human Error" Trap

Many 5 Whys analyses terminate early:

```
Premature termination (WRONG):

  WHY 1: Auth service returned 503s
  WHY 2: Redis failover used a misconfigured replica
  WHY 3: The replica had wrong security group rules
  "ROOT CAUSE": Human error -- the engineer who set up the replica
                 did not configure the security group correctly.

  Action item: Retrain the engineer. (WRONG -- this does nothing.)
```

This analysis stops at the **symptom of a systemic problem**, not the
root cause. It identifies *who* made the error but not *why the system
allowed the error to persist undetected*.

### Why "Human Error" Is Never the Root Cause

**1. Humans are unreliable by nature.** This is not a flaw -- it is a
fundamental property of human cognition. Any system that depends on a
human never making a mistake is a system designed to fail. The question
is not "why did a human make a mistake?" but "why did the system not
catch the mistake?"

**2. Hindsight bias distorts the analysis.** After an incident, the
"correct" action seems obvious. But at the time the decision was made,
the engineer may have had incomplete information, time pressure, or
competing priorities. "They should have known" assumes information
availability that may not have existed.

**3. "Human error" produces useless action items.** If your root cause
is "a person made a mistake," your action items will be:

```
Useless action items from "human error" analysis:
  - "Be more careful"              (not measurable, not enforceable)
  - "Double-check your work"       (vague, no definition of done)
  - "Retrain the engineer"         (does not change the system)
  - "Add a warning to the docs"    (ignored, not enforced)

Effective action items from systemic analysis:
  - "Implement IaC module library" (structural change)
  - "Add drift detection"          (automated guardrail)
  - "Enforce template review"      (process change with tooling)
  - "Standardize Redis provisioning" (eliminates the pattern)
```

**4. Blaming individuals erodes psychological safety.** As covered in
Solution 01, when people fear blame, they stop reporting incidents,
hide mistakes, and avoid risky-but-necessary changes. This leads to
*more* incidents, not fewer.

### The Swiss Cheese Model

The 5 Whys analysis aligns with James Reason's Swiss Cheese Model of
accident causation:

```
Swiss Cheese Model -- Layers of Defense:

  Layer 1        Layer 2        Layer 3        Layer 4
  IaC Module     Template       Security       Monitoring
  Library        Review         Group Sync     Drift Detection
  +--------+    +--------+    +--------+    +--------+
  |  hole  |    |        |    |        |    |        |
  |        |    |  hole  |    |        |    |        |
  |        |    |        |    |  hole  |    |        |
  |        |    |        |    |        |    |  hole  |
  +--------+    +--------+    +--------+    +--------+

When all holes align: the failure passes through every defense
and reaches the user.

The incident happened because ALL layers had gaps. Fixing only
one layer (the human) leaves the other gaps open.
```

The root cause analysis must identify *all* the holes in the Swiss Cheese
Model and propose action items that close them. Stopping at "human error"
identifies only one hole and proposes an unenforceable fix.

### The Test: Would Automation Have Prevented This?

The best test for whether you have reached the true root cause:

```
Ask: "If we replaced the human with a perfectly reliable automation,
      would the incident still have occurred?"

If YES: The root cause is in the system design, not the human.
        The automation would have followed the same flawed process.

If NO:  The root cause may be related to human judgment, and you
        should investigate what context or information was missing.
```

In our example, if the replica provisioning had been automated through
a standardized module, the security group rules would have been
consistent by construction. The "human error" was a symptom of the
lack of standardization.

## Common Mistakes to Avoid

1. **Stopping too early.** If you reach "human error" before Why 5,
   you have stopped too early. Keep asking "why did the system allow
   this?"

2. **Linear chains when the cause is multi-factorial.** Real incidents
   often have multiple contributing factors. Use "and" when necessary:
   "Why did X happen? Because A happened AND B was missing."

3. **Vague root causes.** "Poor communication" or "lack of documentation"
   are not actionable root causes. Be specific: "There is no automated
   mechanism to propagate security group changes from the networking
   team's stack to dependent service stacks."

4. **No action items per why level.** Each level of the 5 Whys may
   warrant its own action item. The immediate fix (Why 1-2), the
   process improvement (Why 3-4), and the systemic change (Why 5)
   all need separate tracked work.

5. **Performing the analysis alone.** The 5 Whys should be a group
   exercise involving people from different teams and perspectives.
   The person closest to the incident may not see the systemic factors.
