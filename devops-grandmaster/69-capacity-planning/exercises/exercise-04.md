# Exercise 04: Growth Forecasting

**Type:** Challenge
**Time:** 45 minutes
**Difficulty:** Medium-Hard

## Objective
Implement a capacity forecasting system that uses historical growth data to predict when infrastructure capacity will be exhausted and recommend when to take action.

## Scenario
Your platform has been growing steadily. You have collected weekly RPS data for the past 12 weeks:

```
Week 1:  2,000 RPS
Week 2:  2,100 RPS
Week 3:  2,250 RPS
Week 4:  2,300 RPS
Week 5:  2,500 RPS
Week 6:  2,700 RPS
Week 7:  2,900 RPS
Week 8:  3,100 RPS
Week 9:  3,400 RPS
Week 10: 3,700 RPS
Week 11: 4,000 RPS
Week 12: 4,400 RPS
```

Your current sustainable capacity is 8,000 RPS (determined by load testing). You need to forecast when you will hit capacity and plan accordingly.

## Tasks

### Part A: Implement Linear Forecasting
Write a Python function `forecast_linear()` that:
1. Takes a list of (date, value) data points and the current capacity
2. Calculates the average daily growth rate using linear regression
3. Projects when the metric will reach capacity
4. Returns the projected exhaustion date and days remaining

<details>
<summary>Hint</summary>
Linear growth: daily_growth = (last_value - first_value) / days_elapsed. Days to exhaustion = (capacity - current_value) / daily_growth. Use `timedelta` to calculate the exhaustion date.
</details>

### Part B: Implement Exponential Forecasting
Write a Python function `forecast_exponential()` that:
1. Takes the same inputs as the linear forecaster
2. Fits an exponential growth model (y = a * e^(b*t))
3. Projects when the metric will reach capacity
4. Returns the projected exhaustion date and days remaining

<details>
<summary>Hint</summary>
Exponential growth: daily_growth_rate = ln(last_value / first_value) / days_elapsed. Days to exhaustion = ln(capacity / current_value) / daily_growth_rate. Exponential models are more accurate for rapidly growing systems.
</details>

### Part C: Compare the Models
Run both forecasters on the data above. Compare:
1. When does each model predict capacity exhaustion?
2. Which model is more conservative (predicts earlier exhaustion)?
3. Which model is more appropriate for this data? Why?

<details>
<summary>Hint</summary>
Look at the growth pattern: early weeks grow slowly (~100 RPS/week), later weeks grow faster (~400 RPS/week). This acceleration suggests exponential growth. The exponential model will predict earlier exhaustion because it accounts for the increasing growth rate.
</details>

### Part D: Generate Action Recommendations
Write a function that, given the forecast results, recommends:
1. When to start capacity expansion (30 days before exhaustion for linear, 60 days for exponential)
2. What capacity to target (20% headroom above projected need at expansion time)
3. A cost estimate (assume $0.10/hour per vCPU, and each 1,000 RPS requires 4 vCPUs)

<details>
<summary>Hint</summary>
Action date = exhaustion date - buffer days. Target capacity = projected_need * 1.2. Monthly cost = (target_capacity / 1000) * 4 vCPUs * $0.10/hr * 730 hours/month.
</details>

## Success Criteria
- [ ] Linear forecasting correctly projects exhaustion date from historical data.
- [ ] Exponential forecasting correctly projects exhaustion date using growth rate.
- [ ] You can explain when to use linear vs exponential forecasting.
- [ ] Action recommendations include specific dates and capacity targets.
- [ ] Cost estimates are calculated based on projected capacity needs.

## What You Should Understand After This Exercise
Forecasting transforms raw metrics into actionable planning. Linear forecasting works for steady, predictable growth. Exponential forecasting is essential for accelerating growth (common in early-stage products). The key output is not the exact date but the "action by" date -- when you need to start capacity expansion to avoid running out.
