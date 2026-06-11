# Exercise 01: Feature Flags and Deployment vs Release

## Objective

Understand the fundamental concepts of feature flags, the critical distinction between deployment and release, and how feature flags enable safer, more controlled software delivery.

## Background

Traditionally, teams treated "deploy" and "release" as the same event: code goes live, users see the new feature. This coupling creates risk -- a bad feature affects all users immediately, rollouts are all-or-nothing, and rollback means redeploying old code.

Feature flags break this coupling. You deploy code to production with new features hidden behind flags, then release features to users by toggling flags independently of deployments.

## Instructions

### Part A: Conceptual Understanding

Answer the following questions in writing. Be specific and provide concrete examples.

1. **Define feature flag.** Write a clear definition of what a feature flag is. Include:
   - What it is (a mechanism, not a UI element)
   - How it works at a high level
   - Why it exists

2. **Deployment vs Release.** Explain the difference between deployment and release. For each scenario below, identify whether it is a deployment, a release, or both:
   - Running `kubectl apply` to update a production deployment
   - Enabling a new checkout flow for 10% of users via a dashboard
   - Rolling back a broken feature by setting a flag to false
   - Pushing a new container image to the production registry
   - Announcing a product launch on social media

3. **Flag types.** Match each scenario to the correct flag type (Release, Experiment, Ops, Permission):
   - (a) Hiding unfinished code behind a toggle so it can be merged into `main`
   - (b) Showing a new search algorithm to 50% of users to compare conversion rates
   - (c) Disabling a third-party integration when its API becomes unreliable
   - (d) Enabling an admin dashboard only for users with the `admin` role

4. **Flag lifecycle.** Describe the typical lifecycle of a feature flag from creation to removal. Why is removing stale flags important?

### Part B: Scenario Analysis

Read the following scenario and answer the questions that follow.

**Scenario:** You work on an e-commerce platform. The team has built a new recommendation engine. Product management wants to roll it out gradually. The engine is significantly different from the current one and changes the layout of the product detail page.

5. **Flag strategy.** How many feature flags would you create for this scenario? Would you use one flag or multiple? Justify your answer.

6. **Rollout plan.** Design a rollout plan for the new recommendation engine. Include:
   - Stages of the rollout
   - Percentage of users at each stage
   - What metrics you would monitor at each stage
   - Criteria for advancing to the next stage
   - Criteria for rolling back

7. **Risk analysis.** What are the risks of deploying the new recommendation engine without a feature flag? List at least four specific risks.

### Part C: Trade-offs and Anti-patterns

8. **Costs of feature flags.** Feature flags are not free. List at least four costs or downsides of maintaining feature flags in a codebase. For each, suggest a mitigation strategy.

9. **Anti-patterns.** For each anti-pattern below, explain why it is problematic and how to fix it:
   - (a) A feature flag that has been in the codebase for 2 years, is always-on, but nobody removed it
   - (b) A feature flag whose name is `flag_3` with no documentation
   - (c) A feature flag that controls three unrelated features at once
   - (d) Checking a feature flag's value by reading a database on every request

10. **When NOT to use feature flags.** Describe at least two situations where feature flags would be unnecessary or harmful. Justify your reasoning.

## Success Criteria

- [ ] You can clearly articulate the difference between deployment and release
- [ ] You can identify and categorize all four types of feature flags
- [ ] You can design a rollout plan with appropriate stages and metrics
- [ ] You understand the costs and anti-patterns associated with feature flags
- [ ] You can reason about when feature flags are and are not appropriate

## Hints

<details>
<summary>Hint 1: Deployment vs Release</summary>

Deployment is about moving code to an environment. Release is about making a feature available to users. A deployment without a release means the code is live but hidden. A release without a deployment means existing code is being activated (e.g., via a flag toggle). Think about which scenarios involve moving code versus activating functionality.

</details>

<details>
<summary>Hint 2: Flag Types</summary>

- **Release toggles** hide work-in-progress from users.
- **Experiment toggles** split traffic for A/B testing.
- **Ops toggles** control system behavior in production (circuit breakers, maintenance modes).
- **Permission toggles** gate features based on user attributes or entitlements.

</details>

<details>
<summary>Hint 3: Rollout Stages</summary>

A common pattern is: internal dogfooding, then 1%, 5%, 25%, 50%, 100%. At each stage, watch error rates, latency, and business metrics (conversion, engagement). If any metric degrades beyond a threshold, halt the rollout.

</details>

<details>
<summary>Hint 4: Flag Costs</summary>

Consider: code complexity from conditional branches, testing burden (every flag doubles the test matrix), risk of stale flags accumulating, performance overhead of flag evaluation, and the operational burden of managing flag state.

</details>
