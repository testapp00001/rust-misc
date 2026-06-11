# Exercise 05: Feature Flags in CI/CD Pipeline

## Objective

Integrate feature flags into a CI/CD pipeline so that deployments are decoupled from releases, flags are managed as part of the deployment process, and rollouts can be controlled and rolled back through the pipeline.

## Background

Feature flags reach their full potential when integrated into the CI/CD pipeline. Instead of asking "should we deploy this code?", the pipeline always deploys, and the question becomes "should we release this feature?". This exercise ties together everything from the previous exercises into a pipeline integration.

## Instructions

### Part A: Pipeline Design

1. **Design the pipeline stages.** A feature-flag-aware CI/CD pipeline should include:

   Design a pipeline with the following stages. For each stage, describe what happens, what gates it, and what can go wrong:

   - **Build**: Compile and package the application
   - **Test**: Run unit, integration, and flag-specific tests
   - **Deploy**: Push to staging, then production
   - **Flag Configure**: Set initial flag state for the new deployment
   - **Canary Release**: Enable the flag for a small percentage
   - **Monitor**: Watch metrics during canary
   - **Progressive Rollout**: Increase percentage through stages
   - **Full Release**: Set flag to 100%
   - **Cleanup**: Remove the flag from code (future step)

2. **Flag-aware testing strategy.** Design a testing strategy that accounts for feature flags:

   - How do you test code behind a disabled flag?
   - How do you test code behind an enabled flag?
   - How do you test the transition from disabled to enabled?
   - How many test combinations exist if you have N flags?
   - How do you reduce the test matrix while maintaining coverage?

3. **Write a pipeline configuration.** Using YAML (GitHub Actions, GitLab CI, or Jenkins syntax of your choice), write a pipeline configuration that:
   - Builds and tests the application
   - Deploys to a staging environment
   - Enables a feature flag in staging with a test user
   - Runs integration tests against staging with the flag enabled
   - Deploys to production with the flag disabled
   - Outputs instructions for manual flag rollout (or triggers an automated rollout)

### Part B: Deployment Patterns

4. **Branch-by-abstraction.** Explain the "branch by abstraction" pattern and how it relates to feature flags. Write code that demonstrates:
   - An interface `SearchProvider` with two implementations (`LegacySearch`, `NewSearch`)
   - A flag-based factory that selects the implementation at runtime
   - The ability to switch implementations without code changes

5. **Database migration with feature flags.** Describe how to safely handle database schema changes alongside feature flags. Specifically:
   - A new column is being added to a table
   - The new feature reads/writes the new column
   - The old feature must continue to work during the rollout
   - Write the migration steps and the flag-aware code that handles both states

6. **Feature flag in a microservices architecture.** Consider a system with three services:

   ```
   [API Gateway] -> [Product Service] -> [Recommendation Service]
   ```

   The new recommendation engine is a new version of the Recommendation Service. Design the deployment strategy:
   - Should the feature flag live in the gateway, the product service, or the recommendation service?
   - How do you ensure consistency if the flag state differs between services?
   - How do you handle the case where the new service is deployed but the flag is still off?

### Part C: Rollback Scenarios

7. **Automated rollback.** Design an automated rollback mechanism that:
   - Monitors error rates after a flag is enabled in production
   - If error rate exceeds a threshold within a time window, automatically disables the flag
   - Sends an alert with the rollback reason
   - Logs the event for post-mortem analysis
   - Write the pseudocode or actual code for this mechanism

8. **Rollback without redeployment.** Demonstrate how feature flags enable rollback without redeployment:
   - Show the steps to roll back a feature using only flag operations
   - Compare this to a traditional rollback (redeploying the previous version)
   - Quantify the time difference (conceptually) between the two approaches

### Part D: Advanced Integration

9. **GitOps and feature flags.** Describe how to manage feature flags in a GitOps workflow:
   - Where should flag configurations be stored in the repository?
   - How does a flag change propagate through the pipeline?
   - How do you handle environment-specific flag states in GitOps?
   - What are the trade-offs of managing flags via Git commits versus a flag management UI?

10. **Observability.** Design an observability strategy for feature flags:
    - What metrics should you emit about flag evaluations? (count, latency, distribution)
    - How do you trace a request through flag evaluation?
    - How do you correlate flag changes with metric changes in your dashboards?
    - Write example log lines and metric names for a flag evaluation event

## Success Criteria

- [ ] You can design a CI/CD pipeline that integrates feature flags at every stage
- [ ] You can implement branch-by-abstraction with flag-based selection
- [ ] You can handle database migrations alongside feature flags
- [ ] You can design a microservices deployment strategy with feature flags
- [ ] You can implement automated rollback based on health metrics
- [ ] You can articulate how feature flags fit into a GitOps workflow
- [ ] You can design observability for feature flag systems

## Hints

<details>
<summary>Hint 1: Pipeline YAML Structure</summary>

```yaml
stages:
  - build
  - test
  - deploy-staging
  - verify-staging
  - deploy-production
  - canary-release
  - monitor
  - progressive-rollout

# Each stage should have:
# - Conditions for when it runs
# - Steps it executes
# - Artifacts it produces
# - Failure handling
```

</details>

<details>
<summary>Hint 2: Test Matrix Reduction</summary>

With N boolean flags, you theoretically have 2^N combinations. In practice:
- Test each flag independently (on/off while others are default)
- Test the "all new flags on" combination
- Test the "all flags off" combination (baseline)
- If flags interact, test those specific interactions
- This reduces from 2^N to roughly 2N + 2 test cases

</details>

<details>
<summary>Hint 3: Database Migration Pattern</summary>

For a new column `preference_data`:
1. Deploy code that reads `preference_data` if the flag is on, otherwise reads the old column
2. Run migration to add `preference_data` column (nullable)
3. Backfill `preference_data` from the old column
4. Enable flag for new writes to use `preference_data`
5. Once 100% on new code, remove old column reads
6. Drop old column (much later)

</details>

<details>
<summary>Hint 4: Observability Signals</summary>

For each flag evaluation, emit:
- `flag.evaluation.count` (counter, tagged by flag_key, result, environment)
- `flag.evaluation.duration` (histogram, in microseconds)
- `flag.change.count` (counter, tagged by flag_key, actor, environment)
- In logs: `{"event": "flag_evaluated", "flag": "new-search", "result": true, "user_id": "u123", "duration_us": 45}`
</details>
