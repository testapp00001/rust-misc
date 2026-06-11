# Exercise 04: Feature Flag Management System

## Objective

Build a feature flag management system with a REST API, supporting CRUD operations, audit logging, flag lifecycle management, and environment-specific configurations.

## Background

Real-world feature flag systems need more than just an evaluation engine. Teams need:
- A central service to manage flags across environments
- An audit trail of who changed what and when
- The ability to have different flag states per environment (dev, staging, prod)
- Flag lifecycle management (create, activate, deactivate, archive, delete)
- Access control to prevent unauthorized changes

## Instructions

### Part A: Data Model

1. **Design the flag model.** Extend the feature flag model from Exercise 02 to include:

```
FeatureFlag:
  - id: unique identifier (UUID)
  - key: string (e.g., "new-search-algorithm")
  - name: human-readable name
  - description: text
  - flag_type: enum (boolean, multivariate, percentage_rollout)
  - created_at: timestamp
  - updated_at: timestamp
  - created_by: user identifier
  - status: enum (active, inactive, archived)
  - tags: list of strings (e.g., ["search", "experiment", "team-growth"])
  - environments: map of environment name -> EnvironmentConfig

EnvironmentConfig:
  - enabled: boolean
  - default_value: any (bool, string, number)
  - rollout_percentage: integer (0-100)
  - targeting_rules: list of TargetingRule
  - overrides: map of user_id -> value

TargetingRule:
  - attribute: string (e.g., "country", "role", "plan")
  - operator: enum (equals, not_equals, in, not_in, contains, gt, lt)
  - values: list of strings
  - value: the value to return if the rule matches
```

2. **Design the audit log model.** Create an `AuditEntry` that captures:
   - Timestamp
   - Actor (who made the change)
   - Action (created, updated, toggled, archived, deleted)
   - Flag key
   - Environment
   - Previous value
   - New value
   - Reason/note (optional free text)

### Part B: REST API

3. **Implement CRUD endpoints.** Design (or implement) the following API endpoints:

   - `POST /flags` -- Create a new feature flag
   - `GET /flags` -- List all flags (with filtering by status, tag, environment)
   - `GET /flags/:key` -- Get a specific flag with its full configuration
   - `PUT /flags/:key` -- Update a flag's configuration
   - `DELETE /flags/:key` -- Delete a flag (with safety check)
   - `POST /flags/:key/environments/:env/toggle` -- Toggle a flag on/off in a specific environment
   - `GET /flags/:key/environments/:env/evaluate` -- Evaluate a flag for a given context
   - `GET /audit-log` -- Get the audit log (with filtering by flag, actor, date range)

4. **Implement validation.** Add input validation for:
   - Flag key must be lowercase, alphanumeric with hyphens, 3-64 characters
   - Flag key must be unique
   - Rollout percentage must be 0-100
   - Targeting rules must reference valid operators
   - Cannot delete an active flag without a force parameter
   - Cannot create a flag with the same key as an archived flag

5. **Implement filtering and pagination.** The `GET /flags` endpoint should support:
   - Filter by status (active, inactive, archived)
   - Filter by tag
   - Filter by environment
   - Search by name or description
   - Pagination with `page` and `per_page` parameters

### Part C: Lifecycle Management

6. **Implement flag lifecycle.** Enforce the following state machine:

```
  [created] -> active -> inactive -> active (toggle)
       |         |
       v         v
    archived  archived
       |
       v
    deleted
```

   Rules:
   - A flag can only be archived if it is inactive
   - A flag can only be deleted if it is archived
   - Archiving requires a reason (audit log entry)
   - Deleting requires a confirmation parameter
   - Archived flags are preserved in the audit log but removed from the active list

7. **Implement stale flag detection.** Create a function that identifies stale flags:
   - A flag that has been inactive for more than 30 days
   - A flag that has been at 100% rollout for more than 90 days (likely should be permanent code)
   - Return a report of stale flags with recommendations (archive, remove, or keep)

### Part D: Environment Management

8. **Implement environment-specific behavior.** Ensure that:
   - Each flag can have different settings per environment
   - Evaluation uses the configuration for the requested environment
   - If no environment-specific config exists, fall back to a default
   - Changes to one environment do not affect others

9. **Implement promotion.** Create an endpoint `POST /flags/:key/promote` that copies a flag's configuration from one environment to another (e.g., staging -> production). Include:
   - A diff showing what will change
   - A confirmation parameter
   - An audit log entry

### Part E: Safety Features

10. **Implement kill switch.** Create an endpoint `POST /emergency/disable-all` that:
    - Immediately disables all flags in a specified environment
    - Returns the list of affected flags and their previous states
    - Logs the emergency action
    - Can be reversed by `POST /emergency/restore`

## Success Criteria

- [ ] CRUD operations work correctly for feature flags
- [ ] Audit log captures all changes with actor, timestamp, and diff
- [ ] Validation prevents invalid flag configurations
- [ ] Lifecycle state machine is enforced correctly
- [ ] Stale flag detection identifies candidates for cleanup
- [ ] Environment-specific configurations are isolated
- [ ] Emergency disable/restore works correctly

## Hints

<details>
<summary>Hint 1: Audit Log Strategy</summary>

Implement audit logging as middleware or a decorator around your mutation endpoints. Every time a flag is created, updated, toggled, archived, or deleted, automatically generate an audit entry. Never allow a mutation without an audit entry.

</details>

<details>
<summary>Hint 2: Safe Deletion</summary>

Never delete a flag outright from the database immediately. Instead:
1. Mark it as `archived` (soft delete)
2. After a configurable retention period (e.g., 90 days), allow permanent deletion
3. Always keep the audit log entries even after permanent deletion

</details>

<details>
<summary>Hint 3: Environment Fallback</summary>

When evaluating a flag for environment `X`:
1. Check if `environments[X]` exists
2. If yes, use that configuration
3. If no, check if `environments["default"]` exists
4. If neither exists, return the flag's global default value

</details>

<details>
<summary>Hint 4: API Response Format</summary>

Use a consistent response format:

```json
{
  "success": true,
  "data": { ... },
  "meta": {
    "page": 1,
    "per_page": 20,
    "total": 150
  }
}
```

For errors:

```json
{
  "success": false,
  "error": {
    "code": "FLAG_NOT_FOUND",
    "message": "No flag found with key 'unknown-flag'"
  }
}
```

</details>
