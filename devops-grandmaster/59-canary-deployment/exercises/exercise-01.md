# Exercise 01: Canary vs Blue-Green Analysis

**Type:** Conceptual
**Time:** 15 minutes
**Difficulty:** Easy

## Objective

Compare canary deployment with blue-green deployment by analyzing risk
exposure, feedback speed, and rollback characteristics. This exercise
trains you to choose the right progressive delivery strategy.

## Scenario

Your team runs a recommendation engine that serves personalized product
suggestions. The current version (v1) has a 2% error rate and 150ms p99
latency. You are deploying v2, which changes the recommendation algorithm.

The service handles 10,000 requests per second across 50 pods.

## Tasks

### Part A: Risk Exposure Comparison

Calculate the maximum number of users affected by a bug in v2 for:
1. Blue-green deployment (all-or-nothing switch)
2. Canary deployment at 5% traffic
3. Canary deployment at 25% traffic

Express as requests per second and percentage of total traffic.

<details>
<summary>Hint</summary>

At 10,000 req/s:
- Blue-green: 100% = 10,000 req/s
- Canary at 5%: 5% = 500 req/s
- Canary at 25%: 25% = 2,500 req/s

</details>

### Part B: Feedback Speed Comparison

Which strategy gives you faster feedback about v2's behavior under real
traffic? Compare the time it takes to detect a 10% error rate increase
in each strategy.

<details>
<summary>Hint</summary>

With blue-green, you detect the error after the switch -- 100% of users
are affected. With canary at 5%, you detect the error from the 5% slice
within minutes, before promoting further. Consider how many requests you
need to observe to detect a 10% error rate increase with statistical
confidence.

</details>

### Part C: Rollback Characteristics

Compare the rollback process for each strategy when a bug is detected:
1. How long does rollback take?
2. How many users are affected during rollback?
3. What happens to in-flight requests during rollback?

<details>
<summary>Hint</summary>

Blue-green rollback: instant (switch Service selector), but all users
were on the bad version. Canary rollback: instant (set weight to 0),
and only the canary slice was affected. In-flight requests to the canary
pods complete normally; new requests go to stable.

</details>

### Part D: When to Use Each Strategy

For each scenario, choose canary or blue-green and justify:

1. A stateless API where any error rate increase is unacceptable
2. A new feature that might have subtle performance regressions
3. A database schema migration that requires atomic switchover
4. A UI redesign where you want to test user engagement metrics

<details>
<summary>Hint</summary>

Canary is better when you want to observe behavior under real traffic
before full promotion. Blue-green is better when you need an atomic
switchover or when the deployment involves infrastructure changes
(like database migrations) that cannot be done incrementally.

</details>

## Success Criteria

- [ ] You can calculate risk exposure for both strategies at different percentages
- [ ] You can explain why canary provides faster feedback with less risk
- [ ] You can compare rollback speed, scope, and in-flight request handling
- [ ] You can choose the right strategy for different scenarios with justification
- [ ] You understand that canary is a superset of blue-green (canary at 100% = blue-green)

## What You Should Understand After This Exercise

Canary deployment trades deployment speed (it takes longer to reach 100%)
for risk reduction (only a small percentage of users are affected by bugs).
Blue-green is faster but riskier. The right choice depends on how much
risk you can tolerate and how much you trust the new version.
