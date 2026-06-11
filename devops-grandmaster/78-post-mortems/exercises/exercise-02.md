# Exercise 02: 5 Whys Root Cause Analysis

**Type:** Guided | **Time:** 30 min | **Difficulty:** Easy-Medium

## Objective

Practice the 5 Whys technique for structured root cause analysis. This exercise
guides you through applying the 5 Whys to a realistic incident scenario,
teaching you to dig past surface-level symptoms to uncover systemic root causes.

## Background

The 5 Whys technique, originally developed by Sakichi Toyoda and used within
Toyota's manufacturing process, is a simple but powerful method for root cause
analysis. By repeatedly asking "Why?" after each answer, you peel back layers
of symptoms to reach the fundamental cause.

Key principles:
- Each "Why" must be answered with **facts and evidence**, not assumptions
- The chain should lead to a **systemic root cause**, not a person
- There may be **multiple branches** at any level (not just a single chain)
- Stop when you reach a cause that the **organization can act upon**
- Use **blameless language** throughout (Exercise 01 skill)

A good root cause is something the team or organization can change: a process,
a tool, a policy, an architecture decision -- not a person's behavior.

## The Incident Scenario

Read the following incident carefully before beginning the analysis.

---

### Incident: Cascading API Failure

**Date:** March 15, 2026
**Duration:** 2 hours 37 minutes (14:23 UTC to 17:00 UTC)
**Severity:** SEV-1 (Critical)
**Customer Impact:** 100% of API requests returning HTTP 500 errors

#### Timeline

| Time (UTC) | Event |
|------------|-------|
| 14:23 | API error rate jumps from 0.1% to 15% within 2 minutes |
| 14:25 | PagerDuty alert fires for "API error rate > 10%" |
| 14:28 | On-call engineer acknowledges alert, begins investigation |
| 14:35 | Engineer identifies that the `user-service` pod is OOM-killed repeatedly |
| 14:42 | Engineer restarts the `user-service` pods; error rate briefly drops |
| 14:47 | Error rate climbs to 100%; `order-service` and `payment-service` also failing |
| 14:52 | Incident escalated to SEV-1; incident commander joins |
| 15:10 | Engineer discovers that a new feature flag `ENABLE_USER_ANALYTICS` was enabled at 14:20 |
| 15:25 | Engineer disables the feature flag; `user-service` stabilizes |
| 15:40 | `order-service` and `payment-service` begin recovering as connection pools drain |
| 16:15 | Error rate returns to baseline 0.1% |
| 17:00 | Incident declared resolved after monitoring confirms stability |

#### Additional Context

- The `ENABLE_USER_ANALYTICS` feature flag was enabled by the product team
  via the LaunchDarkly dashboard. No engineering review was required.
- The feature caused `user-service` to load the full user profile with all
  analytics data for every API request, tripling memory usage per request.
- `user-service` had a memory limit of 512MB, set 18 months ago when the
  service handled fewer fields.
- The `order-service` and `payment-service` depend on `user-service` via
  synchronous HTTP calls with a 30-second timeout and no circuit breaker.
- There was no load testing performed with the new feature enabled.
- The feature flag was rolled out to 100% of users immediately (no gradual
  rollout).
- The on-call engineer's first time handling a SEV-1 at this company.

---

## The Exercise

### Part A: Identify the Immediate Cause

Before starting the 5 Whys, identify the **immediate cause** of the incident.
This is the most surface-level trigger.

```
Immediate cause: [Your answer here]
```

<details>
<summary>Hint</summary>
What was the direct technical trigger that caused the 500 errors?
</details>

---

### Part B: 5 Whys Analysis

Now perform the 5 Whys analysis. Start from the immediate cause and ask "Why?"
at each level. You may branch if there are multiple contributing factors at
any level.

#### Branch 1: Technical Root Cause Chain

**Why #1:** Why did [immediate cause] happen?

```
Answer: [Your answer]
```

**Why #2:** Why did [answer from #1] happen?

```
Answer: [Your answer]
```

**Why #3:** Why did [answer from #2] happen?

```
Answer: [Your answer]
```

**Why #4:** Why did [answer from #3] happen?

```
Answer: [Your answer]
```

**Why #5:** Why did [answer from #4] happen?

```
Answer: [Your answer]
```

<details>
<summary>Hint for Branch 1</summary>
Focus on the technical chain: OOM -> memory usage -> feature flag -> rollout
process -> resource limits. What systemic gaps exist in this chain?
</details>

#### Branch 2: Dependency/Cascading Failure Chain

**Why #1:** Why did the failure cascade from `user-service` to other services?

```
Answer: [Your answer]
```

**Why #2:** Why did [answer from #1] happen?

```
Answer: [Your answer]
```

**Why #3:** Why did [answer from #2] happen?

```
Answer: [Your answer]
```

**Why #4:** Why did [answer from #3] happen?

```
Answer: [Your answer]
```

**Why #5:** Why did [answer from #4] happen?

```
Answer: [Your answer]
```

<details>
<summary>Hint for Branch 2</summary>
Focus on the coupling between services. What architectural decisions allowed
one service's failure to bring down three services? Think about timeouts,
circuit breakers, and fallback behavior.
</details>

#### Branch 3: Detection and Response Chain

**Why #1:** Why did it take 2 hours 37 minutes to resolve?

```
Answer: [Your answer]
```

**Why #2:** Why did [answer from #1] happen?

```
Answer: [Your answer]
```

**Why #3:** Why did [answer from #2] happen?

```
Answer: [Your answer]
```

**Why #4:** Why did [answer from #3] happen?

```
Answer: [Your answer]
```

**Why #5:** Why did [answer from #4] happen?

```
Answer: [Your answer]
```

<details>
<summary>Hint for Branch 3</summary>
Focus on the response time. Why did it take 47 minutes from alert to identify
the feature flag? Was there a runbook? Was the feature flag deployment
correlated with the alert? Was there observability for memory usage per feature?
</details>

---

### Part C: Root Cause Summary

After completing all three branches, synthesize your findings into a root
cause summary. A good root cause summary:

1. Identifies the **systemic issue** (not a person or team)
2. Explains **why the system was vulnerable** to this type of failure
3. Is **actionable** -- the organization can change something to prevent recurrence

```
Root Cause Summary:

[Write 2-3 sentences summarizing the systemic root cause]
```

---

### Part D: Contributing Factors

List all **contributing factors** that, while not the root cause, made the
incident worse or prolonged it. For each factor, note which branch of your
5 Whys it belongs to.

| Contributing Factor | Branch | Systemic Gap |
|--------------------|--------|--------------|
| [Factor 1] | [Branch] | [What system/process should have caught this] |
| [Factor 2] | [Branch] | [What system/process should have caught this] |
| [Factor 3] | [Branch] | [What system/process should have caught this] |
| ... | ... | ... |

<details>
<summary>Hint</summary>
Consider: no load testing, no gradual rollout, no circuit breaker, stale memory
limits, no feature-flag-to-alert correlation, on-call experience level, no
runbook for cascading failures.
</details>

---

### Part E: Proposed Preventive Measures

For each branch of your 5 Whys, propose at least one preventive measure that
would break the chain and prevent this type of incident from recurring.

| Branch | Preventive Measure | Priority (High/Med/Low) | Effort (S/M/L) |
|--------|-------------------|------------------------|----------------|
| Branch 1 | [Measure] | [Priority] | [Effort] |
| Branch 2 | [Measure] | [Priority] | [Effort] |
| Branch 3 | [Measure] | [Priority] | [Effort] |

---

## Reflection Questions

1. **Branching:** Did you find that the 5 Whys naturally branched, or did you
   expect a single linear chain? In real incidents, why is branching common?

2. **Stopping point:** How did you decide when to stop asking "Why?"? Was your
   fifth "Why" a systemic issue the organization can address?

3. **Language check:** Review your answers. Did you use blameless language
   throughout? Identify any places where you slipped into blameful language
   and rewrite them.

4. **Depth:** Were there answers where you wanted to keep asking "Why?" beyond
   five levels? What does that tell you about the depth of the systemic issue?

## Self-Assessment

Rate yourself after completing this exercise:

- [ ] I can perform a structured 5 Whys analysis on an incident scenario
- [ ] I can identify when the analysis should branch into multiple chains
- [ ] I can distinguish between symptoms, contributing factors, and root causes
- [ ] I can write a root cause summary that is systemic and actionable
- [ ] I can propose preventive measures that break the causal chain

## Next Steps

Proceed to [Exercise 03: Write a Complete Post-Mortem](exercise-03.md) to
practice assembling a full post-mortem document from incident facts.
