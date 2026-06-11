# Exercise 04: Graceful Degradation Under Extreme Load

**Type:** Challenge
**Time:** 45 minutes
**Difficulty:** Medium-Hard

## Objective

Design a graceful degradation system that maintains core functionality when your system is under extreme load, rather than failing completely.

## Scenario

You run an e-commerce platform with these features:

| Feature | Criticality | Resource Cost | Cacheable |
|---------|-------------|---------------|-----------|
| Product listing | Critical | Low | Yes (CDN) |
| Search | High | High (Elasticsearch) | Partially |
| Add to cart | Critical | Low | No |
| Checkout | Critical | Medium (payment API) | No |
| Recommendations | Low | High (ML model) | Yes (5 min) |
| User reviews | Low | Medium (DB read) | Yes (1 hour) |
| Inventory check | High | Medium (DB read) | Yes (30 sec) |

During a Black Friday sale, traffic hits 100x normal. Your infrastructure can handle 10x. You need to design a degradation strategy that keeps the site functional for the remaining 90x traffic.

## Tasks

### Part A: Define Degradation Levels

Define four degradation levels (NORMAL, ELEVATED, HIGH, CRITICAL) with specific thresholds for each. For each level, specify:

1. System metrics that trigger the level (CPU, error rate, queue depth, latency)
2. Which features are disabled or degraded
3. What the user sees (full experience vs. reduced experience)
4. How the system recovers when load decreases

<details>
<summary>Hint 1</summary>

Map features to degradation levels: disable recommendations at ELEVATED, disable search at HIGH, serve only cached/static content at CRITICAL. Use queue depth and error rate as primary triggers, not just CPU (CPU can be high without impact if the app is doing useful work).

</details>

### Part B: Implement Degradation Decision Logic

Write a Python class that implements the degradation decision logic. It should:

1. Accept current system metrics as input
2. Return the appropriate degradation level
3. Return a list of features to disable/enable
4. Implement hysteresis (do not oscillate between levels)
5. Log level transitions for monitoring

<details>
<summary>Hint 2</summary>

Use different thresholds for entering and exiting a level (hysteresis). For example, enter HIGH at error_rate > 10%, exit HIGH at error_rate < 5%. This prevents rapid oscillation between levels when metrics hover near the threshold.

</details>

### Part C: Design the CDN Fallback Strategy

When the system reaches CRITICAL degradation, you want to serve as much as possible from the CDN cache. Design a CDN configuration that:

1. Caches product pages aggressively (1 hour)
2. Serves stale content when the origin is down (`stale-if-error`)
3. Returns a static "high traffic" page for dynamic endpoints that cannot be cached
4. Maintains cart functionality through client-side storage (localStorage)

Write the CDN cache rules and the static fallback page content.

<details>
<summary>Hint 3</summary>

Use `stale-if-error` and `stale-while-revalidate` cache-control directives. For the cart, store items in localStorage and sync to the server when the system recovers. The static fallback page should include: product images (cached), prices (cached), and a message explaining the degraded state.

</details>

## Success Criteria

- [ ] Four degradation levels are defined with specific metric thresholds and hysteresis
- [ ] Each level maps features to enable/disable decisions with clear user-visible impact
- [ ] Degradation logic includes hysteresis to prevent oscillation
- [ ] CDN fallback strategy keeps the site browsable even when origin is down
- [ ] Cart functionality survives through client-side storage during CRITICAL degradation

## What You Should Understand After This Exercise

Graceful degradation is about serving something rather than nothing. When you cannot handle all traffic, prioritize critical features (product listing, cart, checkout), cache aggressively, and use client-side storage for state. The user should see a reduced experience, not an error page. Recovery should be automatic as load decreases.
