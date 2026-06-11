# Exercise 01: Calculate Downtime for Availability Targets

**Type:** Conceptual
**Objective:** Understand what common availability percentages actually mean in
terms of real downtime over different time periods.

## Background

When teams say "we target 99.9% uptime," that sounds nearly perfect. But the
remaining 0.1% translates into minutes or hours of downtime depending on the
measurement window. This exercise builds your intuition for those numbers.

## Instructions

### Part A -- Downtime Table

Complete the following table. For each availability percentage, calculate the
allowed downtime per **day**, **week** (7 days), **month** (30 days), and
**year** (365 days).

Round to the nearest whole second for per-day values, and to the nearest minute
for week/month/year values.

| Availability | Per Day | Per Week | Per Month | Per Year |
|--------------|---------|----------|-----------|----------|
| 99%          | ?       | ?        | ?         | ?        |
| 99.5%        | ?       | ?        | ?         | ?        |
| 99.9%        | ?       | ?        | ?         | ?        |
| 99.95%       | ?       | ?        | ?         | ?        |
| 99.99%       | ?       | ?        | ?         | ?        |
| 99.999%      | ?       | ?        | ?         | ?        |

**Formula:**

```
downtime = total_seconds_in_period * (1 - availability_percentage / 100)
```

### Part B -- Scenario Analysis

Your e-commerce platform has an SLO of 99.9% monthly availability. Answer the
following questions:

1. A deployment causes a 15-minute outage on June 1st. How much remaining
   downtime budget do you have for the rest of the month?

2. On June 15th there is a 20-minute partial degradation (counts as 50%
   downtime, so 10 minutes of "full" downtime equivalent). After accounting for
   both incidents, what percentage of your monthly error budget remains?

3. If the same pattern (one 15-minute outage + one 10-minute equivalent) repeats
   every two weeks, how many times can this happen before you exceed your
   **annual** error budget?

### Part C -- Reflection

In 2-3 sentences, explain why "five nines" (99.999%) availability is
significantly harder to achieve than "four nines" (99.99%), even though the
difference is only 0.009%.

## Success Criteria

- [ ] All downtime values in the table are correct within rounding tolerance.
- [ ] Scenario calculations in Part B are accurate.
- [ ] Part C reflection identifies at least one concrete reason beyond "it's a
      smaller number."

## Hints

<details>
<summary>Hint 1 -- Total seconds</summary>

A day has 86,400 seconds. A 30-day month has 2,592,000 seconds. A 365-day year
has 31,536,000 seconds.

</details>

<details>
<summary>Hint 2 -- Partial downtime</summary>

Partial degradations are typically weighted by their severity. If a service is
at 50% capacity for 20 minutes, that counts as 10 minutes of full downtime in
your budget.

</details>

<details>
<summary>Hint 3 -- The nonlinear challenge</summary>

Think about what it takes to go from 99.99% to 99.999%. You need to eliminate
almost all sources of failure -- including ones that are extremely rare or
expensive to prevent. Consider: can you deploy during business hours? Can you
tolerate any single point of failure?

</details>
