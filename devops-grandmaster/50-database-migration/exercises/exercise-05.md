# Exercise 05: Complete Migration Pipeline

**Type:** Integration | **Time:** 45 min | **Difficulty:** Hard

## Objective

Design and implement a complete CI/CD pipeline for database migrations that handles versioning, validation, testing, rollback, and deployment. This integrates everything from the previous exercises into a production-ready system.

## Scenario

Your team manages a microservices platform with 12 services, each with its own PostgreSQL database. Migrations are currently run manually by a senior engineer, which has caused three incidents in the past month:

1. A migration was applied to production but not to staging, causing a deployment failure
2. A migration locked a table for 10 minutes, causing a service outage
3. A migration with a bug corrupted data, and there was no tested rollback procedure

You need to build an automated migration pipeline that prevents these incidents.

Technical context:
- 12 microservices, each with its own database
- PostgreSQL 16 on AWS RDS
- CI/CD: GitHub Actions
- Infrastructure: Kubernetes on EKS
- Migration tool: Flyway (already adopted, but poorly configured)

## Tasks

### Part A: Design the CI/CD Pipeline for Migrations

Design a GitHub Actions workflow that automates database migrations. The pipeline must:

1. Lint and validate migration files on every pull request
2. Apply migrations to a test database and run integration tests
3. Apply migrations to staging automatically on merge to main
4. Apply migrations to production with manual approval
5. Handle the case where multiple services need migrations in a coordinated order

Draw the pipeline flow and write the GitHub Actions YAML for the key stages.

```
[PR] --> [Lint] --> [Test DB Apply] --> [Integration Tests]
  |
  v (merge)
[Staging Apply] --> [Staging Tests] --> [Manual Approval]
  |
  v
[Production Apply] --> [Verification] --> [Rollback if needed]
```

<details>
<summary>Hint</summary>

Your GitHub Actions workflow needs:
- Separate jobs for lint, test, staging, and production
- `environment` protection rules for production
- `concurrency` groups to prevent parallel migrations on the same database
- Service-specific paths so only affected migrations run

```yaml
on:
  pull_request:
    paths:
      - 'services/*/migrations/**'
```

</details>

### Part B: Implement Migration Versioning and Rollback

Design a migration versioning strategy that supports:

1. Forward migrations (schema changes)
2. Backward migrations (rollback scripts)
3. Repeatable migrations (views, functions that are always re-applied)
4. Hotfix migrations (emergency changes that skip the normal pipeline)

Write the migration file naming convention and the Flyway configuration. Also write a rollback script for a typical migration (adding a column).

<details>
<summary>Hint</summary>

Flyway naming convention:
```
V1__create_users_table.sql
V2__add_email_index.sql
V3__add_priority_column.sql
R__recreate_user_view.sql
U1__hotfix_add_missing_index.sql
```

Rollback scripts should be stored alongside migrations:
```
V3__add_priority_column.sql
V3__add_priority_column__rollback.sql
```

The rollback for adding a column:
```sql
ALTER TABLE orders DROP COLUMN IF EXISTS priority;
```

But be careful: if other migrations depend on the column, you need to roll back in reverse order.

</details>

### Part C: Add Pre-Migration Validation Checks

Write the pre-migration validation checks that run before every production migration. The checks must verify:

1. **Disk space**: Sufficient space for table copies and WAL growth
2. **Locks**: No long-running transactions that would block the migration
3. **Replication lag**: Replicas are within acceptable lag before starting
4. **Migration duration estimate**: Warn if the migration is expected to take more than N seconds
5. **Table size check**: Flag migrations on tables larger than a threshold for manual review

Write the SQL queries and bash scripts for each check. Each check should return pass/fail with a descriptive message.

<details>
<summary>Hint</summary>

Disk space check (PostgreSQL):
```sql
SELECT
    pg_database_size(current_database()) as db_size,
    pg_size_pretty(pg_database_size(current_database())) as db_size_pretty;
```

Long-running transactions:
```sql
SELECT pid, now() - xact_start AS duration, query
FROM pg_stat_activity
WHERE state = 'active'
AND xact_start IS NOT NULL
AND now() - xact_start > interval '5 minutes'
ORDER BY duration DESC;
```

Replication lag:
```sql
SELECT
    client_addr,
    state,
    sent_lsn,
    write_lsn,
    flush_lsn,
    replay_lsn,
    pg_wal_lsn_diff(sent_lsn, replay_lsn) as lag_bytes
FROM pg_stat_replication;
```

</details>

### Part D: Design the Migration Testing Strategy

Design a comprehensive testing strategy for database migrations:

1. **Unit tests**: Test migration SQL syntax and basic correctness
2. **Integration tests**: Apply migration to a test database and verify schema
3. **Data integrity tests**: Verify existing data is preserved after migration
4. **Performance tests**: Measure migration duration on a production-sized dataset
5. **Rollback tests**: Verify every migration can be rolled back cleanly

Write the test cases for a migration that adds a NOT NULL column with a default value.

<details>
<summary>Hint</summary>

Testing a NOT NULL column addition:

```sql
-- Integration test: Verify column exists after migration
SELECT column_name, is_nullable, column_default
FROM information_schema.columns
WHERE table_name = 'orders' AND column_name = 'priority';

-- Expected: priority | NO | 0

-- Data integrity test: Verify existing rows have the default
SELECT COUNT(*) FROM orders WHERE priority IS NULL;
-- Expected: 0

-- Performance test: Time the migration on a copy of production data
\timing on
ALTER TABLE orders_copy ADD COLUMN priority INTEGER NOT NULL DEFAULT 0;
```

</details>

## Success Criteria

- [ ] GitHub Actions pipeline has separate stages with appropriate gates
- [ ] Migration naming convention supports forward, backward, and repeatable migrations
- [ ] Rollback scripts exist for every migration and are tested
- [ ] Pre-migration checks cover disk, locks, lag, and table size
- [ ] Testing strategy covers unit, integration, data integrity, performance, and rollback
- [ ] The pipeline prevents the three incidents described in the scenario

## What You Should Understand

After completing this exercise, you should be able to:

1. Build a CI/CD pipeline that makes database migrations as safe as code deployments
2. Design migration versioning that supports rollbacks and hotfixes
3. Write pre-flight checks that catch problems before they become incidents
4. Test migrations thoroughly, including rollback scenarios
5. Coordinate migrations across multiple services safely
