# Exercise 02: Implement Feature Flags in an Application

## Objective

Build a working feature flag system from scratch. You will implement the core components of a feature flag evaluation engine, a configuration store, and integrate it into a simple application.

## Background

At its core, a feature flag system needs:
1. A **configuration store** that holds flag definitions and their current state
2. An **evaluation engine** that determines whether a flag is on or off for a given context
3. An **API** that application code uses to check flag values
4. A **context** object representing the current user/request that the engine uses for targeting rules

In this exercise, you will build all four components.

## Instructions

You may use any programming language you prefer. Pseudocode is acceptable if you want to focus on the design rather than syntax.

### Part A: Configuration Store

1. **Define the data model.** Design a data structure to represent a feature flag. It should support:
   - A unique key (e.g., `new-checkout-flow`)
   - A human-readable name and description
   - A boolean on/off state
   - A default value (what to return if the flag is not found)
   - Optional targeting rules (percentage rollout, specific user IDs)

2. **Implement the store.** Create a `FeatureFlagStore` that can:
   - Load flag configurations from a JSON file
   - Look up a flag by key
   - List all flags
   - Update a flag's state at runtime (without restarting the application)

Provide a sample JSON configuration with at least three flags of different types.

### Part B: Evaluation Engine

3. **Implement basic evaluation.** Create a function `evaluate(flag_key, context) -> value` that:
   - Looks up the flag in the store
   - Returns the default value if the flag is not found
   - Returns the flag's state if no targeting rules are defined

4. **Add percentage-based rollout.** Extend the evaluation to support percentage rollouts:
   - A flag can specify `rollout_percentage: 25` meaning 25% of users see it as enabled
   - The assignment must be **consistent**: the same user always gets the same result for the same flag
   - Use a hashing strategy (hint: hash the user ID + flag key together)

5. **Add user targeting.** Extend the evaluation to support targeting specific users:
   - A flag can specify a list of `enabled_user_ids` or `disabled_user_ids`
   - Disabled users always get `false`, even if they fall within the rollout percentage
   - Enabled users always get `true`, even if they fall outside the rollout percentage
   - User targeting takes priority over percentage rollout

### Part C: Application Integration

6. **Define the context.** Create a `Context` structure that carries:
   - User ID
   - User attributes (role, plan, country, etc.)
   - Request metadata (timestamp, environment)

7. **Wire it together.** Write a simple application (or function) that:
   - Loads flags from the store
   - Creates a context for the current request
   - Evaluates a flag and branches behavior accordingly
   - Logs which flags were evaluated and their results

8. **Write tests.** Write tests that verify:
   - A flag set to `true` returns `true` for all users
   - A flag set to `false` returns `false` for all users
   - Percentage rollout is consistent for the same user
   - User targeting overrides percentage rollout correctly
   - A missing flag key returns the default value

### Part D: Extension (Optional)

9. **Variation support.** Extend the system so a flag can return more than just `true`/`false`. For example, a flag `checkout-experiment` might return `"control"`, `"variant-a"`, or `"variant-b"`. Design the data model and evaluation logic for this.

## Success Criteria

- [ ] Your configuration store can load, look up, list, and update flags
- [ ] Your evaluation engine handles boolean flags, percentage rollouts, and user targeting
- [ ] Percentage rollout produces consistent results for the same user
- [ ] User targeting correctly overrides percentage rollout
- [ ] Your application branches behavior based on flag evaluation
- [ ] All specified tests pass

## Hints

<details>
<summary>Hint 1: Consistent Hashing for Rollout</summary>

To ensure consistent results, hash the concatenation of `user_id + ":" + flag_key`. Use a deterministic hash function (e.g., MD5, SHA-256, or CRC32). Convert the hash to an integer, take it modulo 100, and compare against the rollout percentage. If `hash % 100 < rollout_percentage`, the flag is enabled for that user.

</details>

<details>
<summary>Hint 2: Data Model Example</summary>

```json
{
  "key": "new-checkout-flow",
  "name": "New Checkout Flow",
  "description": "Redesigned checkout experience",
  "enabled": true,
  "default_value": false,
  "rollout_percentage": 25,
  "enabled_user_ids": ["user-beta-001", "user-beta-002"],
  "disabled_user_ids": ["user-excluded-001"]
}
```

</details>

<details>
<summary>Hint 3: Evaluation Order</summary>

The evaluation order should be:
1. Flag exists? If not, return default value.
2. Flag is globally disabled (`enabled: false`)? Return default value.
3. User is in `disabled_user_ids`? Return `false`.
4. User is in `enabled_user_ids`? Return `true`.
5. Percentage rollout check? Return based on consistent hash.
6. Otherwise, return default value.

</details>

<details>
<summary>Hint 4: Runtime Updates</summary>

For runtime updates without restart, consider a pattern like:
- An in-memory store protected by a read-write lock
- A periodic refresh from the configuration file
- Or an API endpoint that accepts flag updates and modifies the in-memory state

</details>
