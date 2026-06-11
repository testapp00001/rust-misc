# Exercise 01: Blameless Language Translation

**Type:** Conceptual | **Time:** 15 min | **Difficulty:** Easy

## Objective

Learn to identify and rewrite blameful language commonly found in post-mortem
discussions. Blameless language focuses on systems, processes, and conditions
rather than individual fault. This exercise trains you to recognize harmful
patterns and replace them with constructive, system-focused alternatives.

## Background

In a blameless post-mortem culture, the language we use matters enormously.
Blameful language discourages participation, hides systemic issues, and
prevents honest reporting. Blameless language acknowledges that humans make
errors and that the system should be resilient to human mistakes.

Common blameful patterns include:
- Naming individuals as the cause
- Using words like "failed to," "negligent," "careless"
- Implying intent behind mistakes
- Focusing on who rather than what/why

Common blameless replacements include:
- Referencing roles or teams instead of individuals
- Describing conditions that enabled the error
- Focusing on systemic factors and contributing conditions
- Using neutral, descriptive language

## The Exercise

Below are 10 statements that might be heard during a post-mortem meeting.
Each contains blameful language. For each statement:

1. **Rewrite** the statement using blameless language.
2. **Explain** why the original language is harmful (what behavior it discourages,
   what information it obscures, or what systemic issue it ignores).

---

### Statement 1

> "John pushed the wrong config to production because he didn't bother to check
> the environment variables."

<details>
<summary>Hint</summary>
Think about what conditions allowed the wrong config to reach production.
Should a single person's attention be the only safety net?
</details>

**Your rewrite:**

```
[Your blameless version here]
```

**Why the original is harmful:**

```
[Your explanation here]
```

---

### Statement 2

> "Sarah made a typo in the SQL migration that corrupted the database."

<details>
<summary>Hint</summary>
Where was the review process? What tooling was missing that would catch
a typo before it reaches the database?
</details>

**Your rewrite:**

```
[Your blameless version here]
```

**Why the original is harmful:**

```
[Your explanation here]
```

---

### Statement 3

> "The on-call engineer was negligent and didn't respond to the PagerDuty alert
> for 45 minutes."

<details>
<summary>Hint</summary>
Consider the alert fatigue context, on-call scheduling, and escalation
policies. Was the alert actionable? Was the engineer overwhelmed?
</details>

**Your rewrite:**

```
[Your blameless version here]
```

**Why the original is harmful:**

```
[Your explanation here]
```

---

### Statement 4

> "The junior developer broke the CI pipeline by merging without running tests."

<details>
<summary>Hint</summary>
Why was it possible to merge without running tests? What branch protection
rules or CI gates were missing?
</details>

**Your rewrite:**

```
[Your blameless version here]
```

**Why the original is harmful:**

```
[Your explanation here]
```

---

### Statement 5

> "Dave forgot to update the SSL certificate, which caused the site to go down."

<details>
<summary>Hint</summary>
Why was certificate renewal a manual, memory-dependent process? What
automation was missing?
</details>

**Your rewrite:**

```
[Your blameless version here]
```

**Why the original is harmful:**

```
[Your explanation here]
```

---

### Statement 6

> "The contractor wrote terrible code that nobody caught during review."

<details>
<summary>Hint</summary>
Focus on the review process, coding standards enforcement, and onboarding.
Why was the code quality standard not enforced automatically?
</details>

**Your rewrite:**

```
[Your blameless version here]
```

**Why the original is harmful:**

```
[Your explanation here]
```

---

### Statement 7

> "Lisa ignored the monitoring dashboard warnings and the service eventually
> crashed."

<details>
<summary>Hint</summary>
Were the warnings actionable? Was there alerting configured? Was the
dashboard the right tool, or should automated responses have been in place?
</details>

**Your rewrite:**

```
[Your blameless version here]
```

**Why the original is harmful:**

```
[Your explanation here]
```

---

### Statement 8

> "The DevOps team was lazy and never set up proper backup procedures."

<details>
<summary>Hint</summary>
Consider organizational priorities, resource allocation, and whether backup
procedures were ever scoped or requested. Focus on what was missing, not why.
</details>

**Your rewrite:**

```
[Your blameless version here]
```

**Why the original is harmful:**

```
[Your explanation here]
```

---

### Statement 9

> "Mark was reckless and ran the database migration during peak traffic hours."

<details>
<summary>Hint</summary>
Were there documented change windows? Was the migration time-sensitive?
Were there policies or guardrails to prevent risky deployment times?
</details>

**Your rewrite:**

```
[Your blameless version here]
```

**Why the original is harmful:**

```
[Your explanation here]
```

---

### Statement 10

> "The intern accidentally deleted the production S3 bucket."

<details>
<summary>Hint</summary>
Why did the intern have delete access to production? What IAM policies,
confirmation mechanisms, or access controls were missing?
</details>

**Your rewrite:**

```
[Your blameless version here]
```

**Why the original is harmful:**

```
[Your explanation here]
```

---

## Reflection Questions

After completing all 10 translations, answer these questions:

1. **Pattern recognition:** What patterns did you notice in the blameful statements?
   Did most blame individuals for things that are really system failures?

2. **Consistency:** Were there statements where you found it harder to remove blame?
   What made them harder?

3. **Culture impact:** How might hearing blameful language affect an on-call
   engineer's willingness to report incidents honestly? How might it affect
   their willingness to participate in post-mortems?

4. **Systemic thinking:** For each statement, identify at least one system-level
   improvement (automation, process, tooling, policy) that would have prevented
   or mitigated the issue -- independent of who was involved.

## Self-Assessment

Rate yourself after completing this exercise:

- [ ] I can identify blameful language patterns in post-mortem discussions
- [ ] I can rewrite statements to focus on systems rather than individuals
- [ ] I understand why blameless language leads to better incident outcomes
- [ ] I can articulate the systemic improvements implied by each rewritten statement

## Next Steps

Proceed to [Exercise 02: 5 Whys Root Cause Analysis](exercise-02.md) to practice
structured root cause analysis -- where blameless language is applied to
deeper investigation.
