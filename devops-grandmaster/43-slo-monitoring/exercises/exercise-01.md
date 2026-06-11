# Exercise 01: Error Budget Calculation

## Objective

Understand how to calculate error budgets for different SLO targets and time
windows. This is the foundational math behind every other exercise in this module.

## Background

An **error budget** is the inverse of an SLO. If your SLO is 99.9% availability,
your error budget is 0.1% -- the amount of failure you can tolerate before
violating the promise to users.

Error budgets are expressed in **time units** (minutes, hours) within a
measurement window (typically 30 days).

## Instructions

### Part A -- Calculate error budgets in minutes

For each SLO target below, calculate the allowed downtime over a **30-day
rolling window**. Express your answer in **minutes**.

| SLO Target | Allowed Error Rate | 30-Day Error Budget (minutes) |
|------------|-------------------|-------------------------------|
| 99.0%      | ?                 | ?                             |
| 99.5%      | ?                 | ?                             |
| 99.9%      | ?                 | ?                             |
| 99.95%     | ?                 | ?                             |
| 99.99%     | ?                 | ?                             |
| 99.999%    | ?                 | ?                             |

Show your work for at least one row.

### Part B -- Error budget burn-down

Your service has a **99.9% SLO** over a 30-day window.

On day 10 of the window, you have already consumed **20 minutes** of downtime.

1. What is your total error budget for the 30-day window?
2. What percentage of the error budget have you consumed by day 10?
3. At the current burn rate, will you exhaust the budget before day 30?
4. If yes, on approximately which day will the budget be exhausted?

### Part C -- Comparing services

You manage three microservices:

| Service | SLO Target | Window   | Downtime this window |
|---------|-----------|----------|----------------------|
| API     | 99.9%     | 30 days  | 30 minutes           |
| Auth    | 99.95%    | 30 days  | 15 minutes           |
| Search  | 99.99%    | 30 days  | 3 minutes            |

1. Which service is closest to violating its SLO?
2. Which service has the most error budget remaining (in absolute minutes)?
3. If you can only fix one service this sprint, which should you prioritize
   and why?

### Part D -- Converting between windows

Your team wants to switch from a 30-day window to a **28-day window** for
alignment with calendar weeks.

1. For a 99.9% SLO, what is the error budget in minutes for 28 days vs 30 days?
2. Does a shorter window make the SLO more or less forgiving? Explain why.

## Success Criteria

- [ ] All table cells in Part A are correctly filled
- [ ] Part B answers include specific minute/day values with shown work
- [ ] Part C ranking is correct with clear justification
- [ ] Part D demonstrates understanding of how window length affects budgets

## Hints

<details>
<summary>Hint 1: Formula</summary>

Error budget (minutes) = Total window minutes * (1 - SLO target as decimal)

For 30 days: 30 * 24 * 60 = 43,200 minutes total

</details>

<details>
<summary>Hint 2: Burn rate</summary>

Burn rate = Actual consumption / Expected consumption at this point in the
window.

A burn rate > 1.0 means you are spending the budget faster than sustainable.

</details>

<details>
<summary>Hint 3: Remaining budget</summary>

Remaining budget = Total budget - Consumed budget

Compare remaining budget against remaining time to determine if the current
rate is sustainable.

</details>
