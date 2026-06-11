# Exercise 03: Gradual Rollout with Feature Flags

## Objective

Implement a gradual rollout system that allows you to progressively expose a new feature to increasing percentages of users, with monitoring, automatic health checks, and the ability to halt or rollback the rollout.

## Background

Gradual rollouts (also called canary releases or progressive delivery) reduce risk by exposing changes to a small fraction of users first. If metrics look good, the percentage increases. If metrics degrade, the rollout halts or rolls back automatically.

This exercise combines feature flags with monitoring to create a controlled rollout pipeline.

## Instructions

### Part A: Rollout Configuration

1. **Design the rollout model.** Create a data structure for a rollout plan that includes:
   - The flag key being rolled out
   - A list of stages, each with:
     - Target percentage (e.g., 1%, 5%, 25%, 50%, 100%)
     - Duration to hold at that stage (e.g., 2 hours, 24 hours)
     - Health check thresholds (max error rate, max latency p99)
   - Current stage index
   - Status (pending, in_progress, paused, completed, rolled_back)

2. **Write a sample rollout plan** for a new "Search v2" feature with at least four stages. Define realistic health check thresholds for each stage.

### Part B: Rollout Controller

3. **Implement the rollout controller.** Create a `RolloutController` that:
   - Reads the rollout plan
   - Sets the flag's rollout percentage to the current stage's target
   - Waits for the stage duration
   - Checks health metrics against the stage thresholds
   - Advances to the next stage if healthy, or pauses/rolls back if unhealthy
   - Logs every state transition with a timestamp and reason

4. **Implement health checks.** Create a `HealthChecker` that monitors:
   - Error rate (percentage of requests returning 5xx)
   - Latency (p99 response time)
   - Business metric (e.g., conversion rate or click-through rate)
   The checker should accept thresholds and return a pass/fail verdict with details.

5. **Implement automatic rollback.** When a health check fails:
   - Immediately set the rollout percentage to 0% (disable the flag)
   - Log the rollback event with the failing metric and its value
   - Notify the team (simulate this with a log message)
   - Do NOT advance to the next stage

### Part C: Simulation

6. **Build a simulator.** Create a simulation that:
   - Has a pool of 1000 simulated users (with IDs)
   - Routes users through the feature flag based on the current rollout percentage
   - Simulates metrics for both the old and new versions:
     - Old version: 0.5% error rate, 200ms p99 latency, 3.2% conversion
     - New version: 0.6% error rate, 180ms p99 latency, 3.5% conversion
   - Runs the rollout controller through all stages
   - Reports whether the rollout completed successfully

7. **Simulate a failure.** Modify the simulator so the new version has a 5% error rate at the 25% stage. Verify that:
   - The rollout controller detects the failure
   - The flag is rolled back to 0%
   - A rollback event is logged
   - The system returns to the old version for all users

8. **Simulate a pause and resume.** Demonstrate:
   - A manual pause of the rollout at a specific stage
   - Investigation (simulated by a log message)
   - A manual resume that continues from the paused stage

### Part D: Analysis

9. **Stage duration trade-offs.** Discuss the trade-offs between short stage durations (e.g., 10 minutes) and long stage durations (e.g., 24 hours). When would you choose each?

10. **Metric sensitivity.** If your error rate threshold is 1% and the new version has a 1.05% error rate, should the rollout proceed? Discuss the implications of setting thresholds too tight versus too loose.

## Success Criteria

- [ ] Your rollout plan model supports multiple stages with thresholds
- [ ] The rollout controller correctly advances through stages
- [ ] Health checks correctly evaluate metrics against thresholds
- [ ] Automatic rollback triggers when thresholds are exceeded
- [ ] The simulator demonstrates a successful multi-stage rollout
- [ ] The simulator correctly detects and handles a failure scenario
- [ ] All state transitions are logged with timestamps and reasons

## Hints

<details>
<summary>Hint 1: Consistent Percentage Rollout</summary>

Use the same hashing strategy from Exercise 02. To determine if a user is in the rollout, compute `hash(user_id + ":" + flag_key) % 100 < rollout_percentage`. As the percentage increases from stage to stage, more users will pass this check. Users who were included at 5% will still be included at 25%.

</details>

<details>
<summary>Hint 2: Health Check Aggregation</summary>

Aggregate metrics over the duration of each stage. For example, if the stage duration is 2 hours, collect all error counts and request counts over that window, then compute the error rate as `errors / total_requests`. Do not evaluate on a per-request basis -- you need volume to get reliable numbers.

</details>

<details>
<summary>Hint 3: Rollout State Machine</summary>

```
pending -> in_progress -> completed
                  |
                  v
               paused -> in_progress (on resume)
                  |
                  v
              rolled_back
```

</details>

<details>
<summary>Hint 4: Simulation Metrics</summary>

To simulate metrics, use a random number generator. For a version with a 0.5% error rate, generate a random number between 0 and 1; if it is less than 0.005, count it as an error. Aggregate these over the simulated requests in each stage window.

</details>
