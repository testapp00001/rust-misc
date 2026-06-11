# Solution 01: Error Budget Calculation

## Part A -- Error budgets in minutes

**Formula**: Error budget (minutes) = Total window minutes * (1 - SLO target)

**Total minutes in 30 days**: 30 * 24 * 60 = **43,200 minutes**

| SLO Target | Allowed Error Rate | 30-Day Error Budget (minutes) |
|------------|-------------------|-------------------------------|
| 99.0%      | 1.0%              | 432.0 minutes (7h 12m)       |
| 99.5%      | 0.5%              | 216.0 minutes (3h 36m)       |
| 99.9%      | 0.1%              | 43.2 minutes                 |
| 99.95%     | 0.05%             | 21.6 minutes                 |
| 99.99%     | 0.01%             | 4.32 minutes                 |
| 99.999%    | 0.001%            | 0.432 minutes (25.9 seconds) |

**Shown work for 99.9% SLO**:

```
Total minutes = 30 days * 24 hours * 60 minutes = 43,200 minutes
Allowed error rate = 1 - 0.999 = 0.001
Error budget = 43,200 * 0.001 = 43.2 minutes
```

**Why this works**: The SLO target defines the minimum acceptable success
ratio. The complement (1 - target) is the maximum tolerable failure ratio.
Multiplying by the window duration converts the ratio into a time unit.

**Common mistakes**:
- Using 365-day year instead of 30-day window
- Forgetting to convert percentages to decimals (using 99.9 instead of 0.999)
- Confusing the error *rate* with the error *budget* (they are the same
  concept at different scales)

---

## Part B -- Error budget burn-down

**1. Total error budget for 30-day window at 99.9% SLO**:

```
43,200 * 0.001 = 43.2 minutes
```

**2. Percentage consumed by day 10**:

```
Consumed = 20 minutes
Percentage = (20 / 43.2) * 100 = 46.3%
```

**3. Will you exhaust the budget before day 30?**

```
Daily burn rate = 20 minutes / 10 days = 2 minutes/day
Projected total consumption = 2 minutes/day * 30 days = 60 minutes
Budget = 43.2 minutes
60 > 43.2, so YES -- the budget will be exhausted.
```

**4. Day of exhaustion**:

```
Days until exhaustion = 43.2 / 2 = 21.6 days
The budget will be exhausted around day 22.
```

**Why this works**: Linear projection assumes the current burn rate continues.
In practice, incidents are bursty, so this is a rough estimate -- but it
provides a clear warning signal.

**Common mistakes**:
- Projecting from day 1 instead of the current day
- Using a non-linear model without justification
- Not showing the work (the calculation matters more than the number)

---

## Part C -- Comparing services

**Error budget calculations**:

| Service | SLO    | Budget (min) | Downtime | Consumed % | Remaining (min) |
|---------|--------|-------------|----------|------------|-----------------|
| API     | 99.9%  | 43.2        | 30 min   | 69.4%      | 13.2            |
| Auth    | 99.95% | 21.6        | 15 min   | 69.4%      | 6.6             |
| Search  | 99.99% | 4.32        | 3 min    | 69.4%      | 1.32            |

**1. Closest to violating SLO**:

**Search** is closest. It has only 1.32 minutes of budget remaining. Any
further downtime of more than ~1.3 minutes will violate the SLO.

**2. Most budget remaining in absolute minutes**:

**API** has the most remaining at 13.2 minutes.

**3. Which to fix first**:

**Search** should be the priority. Even though all three services have consumed
the same percentage of their budget, Search has the smallest absolute margin.
A single additional incident (even a 2-minute blip) would violate the SLO.
The tightest constraint deserves the most attention.

**Why this works**: Percentage-based comparison can be misleading when budgets
differ in absolute size. A 30% remaining budget on a 99.99% SLO is much more
urgent than 30% remaining on a 99.0% SLO.

**Common mistakes**:
- Prioritizing by consumed percentage alone (all are ~69%)
- Ignoring that tighter SLOs have less absolute margin
- Not considering the blast radius of each service

---

## Part D -- Converting between windows

**1. Error budget comparison**:

```
30-day window: 43,200 * 0.001 = 43.2 minutes
28-day window: 40,320 * 0.001 = 40.32 minutes

Difference: 43.2 - 40.32 = 2.88 minutes less budget with 28-day window
```

**2. More or less forgiving?**

A shorter window is **less forgiving**. You have 2.88 minutes less budget.

**Why**: With a shorter window, you have less total time to "absorb" failures.
The same SLO percentage translates to fewer absolute minutes of allowed
downtime. Additionally, incidents that occur near the boundary of a 28-day
window are counted more heavily because there is less surrounding time to
dilute their impact.

A 30-day window with one 43-minute outage is at the limit. A 28-day window
with the same outage is already over budget.

**Common mistakes**:
- Thinking the percentage stays the same so it does not matter
- Not realizing that window alignment affects which incidents are counted
- Ignoring that shorter windows are more sensitive to bursty incidents
