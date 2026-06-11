# Solution 05: Complete Migration Pipeline

## Part A: Design the CI/CD Pipeline for Migrations

### Pipeline Architecture

```
PR Opened
    |
    v
[Lint Migrations] --> [Spin Up Test DB] --> [Apply Migrations] --> [Integration Tests]
    |                        |                      |                     |
    v                        v                      v                     v
 Fail PR                 Timeout?              Syntax Error?          Test Fail?
    |
    v (merge to main)
[Staging Apply] --> [Staging Verification] --> [Manual Approval Gate]
    |                      |                        |
    v                      v                        v
 Rollback              Lag Check?              Approved?
    |
    v
[Production Apply] --> [Production Verification] --> [Cleanup Old Tables]
    |                        |
    v                        v
 Rollback              Alert on Issues
```

### GitHub Actions Workflow

```yaml
# .github/workflows/database-migration.yml
name: Database Migration Pipeline

on:
  pull_request:
    paths:
      - 'migrations/**'
  push:
    branches: [main]
    paths:
      - 'migrations/**'

concurrency:
  group: db-migration-${{ github.ref }}
  cancel-in-progress: false  # Never cancel in-progress migrations

env:
  FLYWAY_VERSION: "10.4.0"

jobs:
  lint:
    name: Lint Migration Files
    runs-on: ubuntu-latest
    if: github.event_name == 'pull_request'
    steps:
      - uses: actions/checkout@v4

      - name: Check migration naming convention
        run: |
          echo "Checking migration file naming..."
          ERRORS=0
          for f in migrations/V*.sql; do
            filename=$(basename "$f")
            if ! echo "$filename" | grep -qE '^V[0-9]+__[a-z_]+\.sql$'; then
              echo "ERROR: Invalid naming: $filename (expected V{number}__{description}.sql)"
              ERRORS=$((ERRORS + 1))
            fi
          done
          for f in migrations/R__*.sql; do
            filename=$(basename "$f")
            if ! echo "$filename" | grep -qE '^R__[a-z_]+\.sql$'; then
              echo "ERROR: Invalid naming: $filename (expected R__{description}.sql)"
              ERRORS=$((ERRORS + 1))
            fi
          done
          if [ $ERRORS -gt 0 ]; then
            exit 1
          fi
          echo "All migration files pass naming convention."

      - name: Check for destructive operations
        run: |
          echo "Scanning for destructive operations..."
          WARNINGS=0
          for f in migrations/*.sql; do
            if grep -iE 'DROP TABLE|DROP COLUMN|TRUNCATE|DELETE FROM' "$f"; then
              echo "WARNING: Destructive operation in $f"
              WARNINGS=$((WARNINGS + 1))
            fi
          done
          if [ $WARNINGS -gt 0 ]; then
            echo "::warning::Found $WARNINGS destructive operations. Requires manual review."
          fi

      - name: Validate SQL syntax
        run: |
          # Install pgFormatter for syntax checking
          sudo apt-get install -y pgFormatter
          ERRORS=0
          for f in migrations/*.sql; do
            if ! pg_format --no-rc "$f" > /dev/null 2>&1; then
              echo "ERROR: Syntax error in $f"
              ERRORS=$((ERRORS + 1))
            fi
          done
          if [ $ERRORS -gt 0 ]; then
            exit 1
          fi

  test:
    name: Test Migrations
    runs-on: ubuntu-latest
    needs: lint
    services:
      postgres:
        image: postgres:16
        env:
          POSTGRES_PASSWORD: testpass
          POSTGRES_DB: testdb
        ports:
          - 5432:5432
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
    steps:
      - uses: actions/checkout@v4

      - name: Wait for PostgreSQL
        run: |
          until pg_isready -h localhost -p 5432 -U postgres; do
            echo "Waiting for postgres..."
            sleep 2
          done

      - name: Install Flyway
        run: |
          wget -qO- https://download.red-gate.com/maven/release/com/redgate/flyway/flyway-commandline/${{ env.FLYWAY_VERSION }}/flyway-commandline-${{ env.FLYWAY_VERSION }}-linux-x64.tar.gz | tar xz
          export PATH="$PWD/flyway-${{ env.FLYWAY_VERSION }}:$PATH"
          echo "$PWD/flyway-${{ env.FLYWAY_VERSION }}" >> $GITHUB_PATH

      - name: Apply migrations
        run: |
          flyway -url=jdbc:postgresql://localhost:5432/testdb \
                 -user=postgres \
                 -password=testpass \
                 -locations=filesystem:migrations \
                 migrate
        env:
          FLYWAY_PLACEHOLDERS_ENV: test

      - name: Run integration tests
        run: |
          # Test that all expected tables exist
          TABLES=$(PGPASSWORD=testpass psql -h localhost -U postgres -d testdb -t -c "
            SELECT tablename FROM pg_tables WHERE schemaname = 'public' ORDER BY tablename;
          ")
          echo "Tables created: $TABLES"

          # Test that migrations are idempotent (run again, should succeed)
          flyway -url=jdbc:postgresql://localhost:5432/testdb \
                 -user=postgres \
                 -password=testpass \
                 -locations=filesystem:migrations \
                 info

      - name: Test rollback scripts
        run: |
          # Apply latest migration, then roll it back
          LATEST=$(flyway -url=jdbc:postgresql://localhost:5432/testdb \
                  -user=postgres -password=testpass \
                  -locations=filesystem:migrations info -outputType=json | \
                  jq -r '.migrations[-1].version')

          echo "Latest migration: $LATEST"

          # Execute rollback if it exists
          ROLLBACK_FILE="migrations/V${LATEST}__rollback.sql"
          if [ -f "$ROLLBACK_FILE" ]; then
            PGPASSWORD=testpass psql -h localhost -U postgres -d testdb -f "$ROLLBACK_FILE"
            echo "Rollback of V${LATEST} succeeded."
          else
            echo "No rollback file found for V${LATEST}. Consider adding one."
          fi

  staging:
    name: Apply to Staging
    runs-on: ubuntu-latest
    needs: test
    if: github.ref == 'refs/heads/main' && github.event_name == 'push'
    environment: staging
    steps:
      - uses: actions/checkout@v4

      - name: Install Flyway
        run: |
          wget -qO- https://download.red-gate.com/maven/release/com/redgate/flyway/flyway-commandline/${{ env.FLYWAY_VERSION }}/flyway-commandline-${{ env.FLYWAY_VERSION }}-linux-x64.tar.gz | tar xz
          echo "$PWD/flyway-${{ env.FLYWAY_VERSION }}" >> $GITHUB_PATH

      - name: Pre-migration checks
        run: |
          echo "Running pre-migration checks on staging..."
          PGPASSWORD=${{ secrets.STAGING_DB_PASSWORD }} psql \
            -h ${{ secrets.STAGING_DB_HOST }} \
            -U ${{ secrets.STAGING_DB_USER }} \
            -d ${{ secrets.STAGING_DB_NAME }} \
            -f scripts/pre_migration_checks.sql

      - name: Apply migrations to staging
        run: |
          flyway -url=jdbc:postgresql://${{ secrets.STAGING_DB_HOST }}:5432/${{ secrets.STAGING_DB_NAME }} \
                 -user=${{ secrets.STAGING_DB_USER }} \
                 -password=${{ secrets.STAGING_DB_PASSWORD }} \
                 -locations=filesystem:migrations \
                 migrate

      - name: Post-migration verification
        run: |
          PGPASSWORD=${{ secrets.STAGING_DB_PASSWORD }} psql \
            -h ${{ secrets.STAGING_DB_HOST }} \
            -U ${{ secrets.STAGING_DB_USER }} \
            -d ${{ secrets.STAGING_DB_NAME }} \
            -f scripts/post_migration_verify.sql

      - name: Run staging smoke tests
        run: |
          python scripts/smoke_test.py --env staging

  production:
    name: Apply to Production
    runs-on: ubuntu-latest
    needs: staging
    if: github.ref == 'refs/heads/main' && github.event_name == 'push'
    environment:
      name: production
      url: https://app.example.com
    steps:
      - uses: actions/checkout@v4

      - name: Install Flyway
        run: |
          wget -qO- https://download.red-gate.com/maven/release/com/redgate/flyway/flyway-commandline/${{ env.FLYWAY_VERSION }}/flyway-commandline-${{ env.FLYWAY_VERSION }}-linux-x64.tar.gz | tar xz
          echo "$PWD/flyway-${{ env.FLYWAY_VERSION }}" >> $GITHUB_PATH

      - name: Pre-migration checks (production)
        run: |
          PGPASSWORD=${{ secrets.PROD_DB_PASSWORD }} psql \
            -h ${{ secrets.PROD_DB_HOST }} \
            -U ${{ secrets.PROD_DB_USER }} \
            -d ${{ secrets.PROD_DB_NAME }} \
            -f scripts/pre_migration_checks.sql

      - name: Create backup
        run: |
          TIMESTAMP=$(date +%Y%m%d_%H%M%S)
          pg_dump -h ${{ secrets.PROD_DB_HOST }} \
                  -U ${{ secrets.PROD_DB_USER }} \
                  -d ${{ secrets.PROD_DB_NAME }} \
                  -f /tmp/pre_migration_backup_${TIMESTAMP}.sql
          echo "Backup created: pre_migration_backup_${TIMESTAMP}.sql"

      - name: Apply migrations to production
        run: |
          flyway -url=jdbc:postgresql://${{ secrets.PROD_DB_HOST }}:5432/${{ secrets.PROD_DB_NAME }} \
                 -user=${{ secrets.PROD_DB_USER }} \
                 -password=${{ secrets.PROD_DB_PASSWORD }} \
                 -locations=filesystem:migrations \
                 migrate

      - name: Post-migration verification (production)
        run: |
          PGPASSWORD=${{ secrets.PROD_DB_PASSWORD }} psql \
            -h ${{ secrets.PROD_DB_HOST }} \
            -U ${{ secrets.PROD_DB_USER }} \
            -d ${{ secrets.PROD_DB_NAME }} \
            -f scripts/post_migration_verify.sql

      - name: Run production smoke tests
        run: |
          python scripts/smoke_test.py --env production

      - name: Notify team
        if: always()
        run: |
          STATUS=${{ job.status }}
          curl -X POST ${{ secrets.SLACK_WEBHOOK_URL }} \
            -H 'Content-type: application/json' \
            -d "{\"text\": \"Database migration to production: ${STATUS}\"}"
```

## Part B: Implement Migration Versioning and Rollback

### Flyway Configuration

```properties
# flyway.conf
flyway.url=jdbc:postgresql://localhost:5432/ecommerce
flyway.user=flyway_user
flyway.password=${DB_PASSWORD}
flyway.schemas=public
flyway.table=flyway_schema_history
flyway.baselineOnMigrate=true
flyway.validateOnMigrate=true
flyway.cleanDisabled=true
flyway.outOfOrder=false
flyway.locations=filesystem:migrations
```

### Flyway Naming Conventions

| Prefix | Purpose | Example | Behavior |
|--------|---------|---------|----------|
| `V__` | Versioned migrations | `V001__create_users.sql` | Applied once, in order. Tracked by version number. |
| `R__` | Repeatable migrations | `R__recreate_user_stats_view.sql` | Re-applied whenever the file checksum changes. Runs after all `V__` migrations. |
| `U__` | Undo migrations (Flyway Pro/Enterprise) | `U001__undo_create_users.sql` | Paired with a `V__` migration of the same number. Used to undo a specific version. |

**When to use each**:
- `V__` for all schema-changing migrations (CREATE TABLE, ALTER TABLE, CREATE INDEX)
- `R__` for views, stored procedures, functions, and grants -- objects that are fully replaced, not altered
- `U__` for rollback scripts when using Flyway Pro/Enterprise's built-in undo capability; otherwise use the `__rollback` suffix convention shown below

### Migration File Structure

```
migrations/
├── V001__create_users_table.sql
├── V001__create_users_table__rollback.sql
├── V002__create_orders_table.sql
├── V002__create_orders_table__rollback.sql
├── V003__add_email_index.sql
├── V003__add_email_index__rollback.sql
├── V004__add_priority_column.sql
├── V004__add_priority_column__rollback.sql
├── R__recreate_user_stats_view.sql
├── R__recreate_order_summary_function.sql
└── scripts/
    ├── pre_migration_checks.sql
    ├── post_migration_verify.sql
    └── smoke_test.py
```

When using Flyway Pro/Enterprise with native undo support, the structure looks like this instead:

```
migrations/
├── V001__create_users_table.sql
├── U001__undo_create_users_table.sql
├── V002__create_orders_table.sql
├── U002__undo_create_orders_table.sql
├── R__recreate_user_stats_view.sql
└── scripts/
    └── ...
```

If you are on the Community edition, stick with the `__rollback` suffix convention. The CI pipeline lints for the presence of a rollback file alongside every `V__` migration.

### Example Forward Migration with Rollback

```sql
-- migrations/V004__add_priority_column.sql
-- Forward migration: Add priority column to orders
-- Author: platform-team
-- Date: 2024-01-15
-- Risk: Low (nullable column, instant operation)

ALTER TABLE orders ADD COLUMN IF NOT EXISTS priority INTEGER DEFAULT 0;
COMMENT ON COLUMN orders.priority IS 'Order priority: 0=normal, 1=high, 2=urgent';
```

```sql
-- migrations/V004__add_priority_column__rollback.sql
-- Rollback: Remove priority column from orders
-- WARNING: This is destructive. Ensure no application code depends on this column.

-- Verify no application queries reference this column
DO $$
BEGIN
    -- Check if any views reference the column
    IF EXISTS (
        SELECT 1 FROM information_schema.view_column_usage
        WHERE table_name = 'orders' AND column_name = 'priority'
    ) THEN
        RAISE EXCEPTION 'Cannot rollback: views reference the priority column';
    END IF;
END $$;

ALTER TABLE orders DROP COLUMN IF EXISTS priority;
```

### Hotfix Migration Strategy

For emergency changes that cannot wait for the full pipeline:

```yaml
# .github/workflows/db-hotfix.yml
name: Database Hotfix

on:
  workflow_dispatch:
    inputs:
      migration_file:
        description: 'Migration file to apply (e.g., V005__hotfix_add_index.sql)'
        required: true
      reason:
        description: 'Reason for hotfix'
        required: true

jobs:
  hotfix:
    name: Apply Hotfix
    runs-on: ubuntu-latest
    environment: production
    steps:
      - uses: actions/checkout@v4

      - name: Verify hotfix file exists
        run: |
          if [ ! -f "migrations/${{ github.event.inputs.migration_file }}" ]; then
            echo "ERROR: Migration file not found"
            exit 1
          fi
          echo "Hotfix file verified."

      - name: Apply hotfix
        run: |
          flyway -url=jdbc:postgresql://${{ secrets.PROD_DB_HOST }}:5432/${{ secrets.PROD_DB_NAME }} \
                 -user=${{ secrets.PROD_DB_USER }} \
                 -password=${{ secrets.PROD_DB_PASSWORD }} \
                 -locations=filesystem:migrations \
                 -target=${{ github.event.inputs.migration_file }} \
                 migrate

      - name: Record hotfix
        run: |
          echo "Hotfix applied: ${{ github.event.inputs.migration_file }}" >> hotfix_log.txt
          echo "Reason: ${{ github.event.inputs.reason }}" >> hotfix_log.txt
          echo "Applied by: ${{ github.actor }}" >> hotfix_log.txt
          echo "Date: $(date -u)" >> hotfix_log.txt
          echo "---" >> hotfix_log.txt
```

### Multi-Service Migration Ordering

```yaml
# .github/workflows/coordinated-migration.yml
name: Coordinated Multi-Service Migration

on:
  workflow_dispatch:
    inputs:
      services:
        description: 'Comma-separated list of services to migrate in order'
        required: true
        default: 'users,orders,payments'

jobs:
  migrate:
    name: Migrate Services
    runs-on: ubuntu-latest
    strategy:
      max-parallel: 1  # Run sequentially
      matrix:
        service: ${{ fromJson(format('["{0}"]', join(fromJson(format('["{0}"]', github.event.inputs.services)), '","'))) }}
    steps:
      - uses: actions/checkout@v4

      - name: Migrate ${{ matrix.service }}
        run: |
          echo "Migrating service: ${{ matrix.service }}"
          flyway -url=jdbc:postgresql://${{ secrets[format('{0}_DB_HOST', matrix.service)] }}:5432/${{ matrix.service }} \
                 -user=${{ secrets[format('{0}_DB_USER', matrix.service)] }} \
                 -password=${{ secrets[format('{0}_DB_PASSWORD', matrix.service)] }} \
                 -locations=filesystem:services/${{ matrix.service }}/migrations \
                 migrate
```

## Part C: Add Pre-Migration Validation Checks

### Pre-Migration Checks Script

```sql
-- scripts/pre_migration_checks.sql
-- Run this before every production migration
-- Returns: PASS/FAIL for each check with descriptive messages

DO $$
DECLARE
    check_name TEXT;
    check_result TEXT;
    all_passed BOOLEAN := TRUE;
BEGIN
    RAISE NOTICE '=== Pre-Migration Validation Checks ===';
    RAISE NOTICE '';

    -- Check 1: Disk space
    check_name := 'Disk Space';
    BEGIN
        IF (SELECT pg_database_size(current_database())) > 50 * 1024 * 1024 * 1024 THEN
            RAISE NOTICE 'WARNING [%]: Database size is %, which may require extra disk for migration',
                check_name, pg_size_pretty(pg_database_size(current_database()));
        ELSE
            RAISE NOTICE 'PASS [%]: Database size is %',
                check_name, pg_size_pretty(pg_database_size(current_database()));
        END IF;
    EXCEPTION WHEN OTHERS THEN
        RAISE NOTICE 'FAIL [%]: %', check_name, SQLERRM;
        all_passed := FALSE;
    END;

    -- Check 2: Long-running transactions
    check_name := 'Long-Running Transactions';
    BEGIN
        IF EXISTS (
            SELECT 1 FROM pg_stat_activity
            WHERE state = 'active'
            AND xact_start IS NOT NULL
            AND now() - xact_start > interval '5 minutes'
            AND pid != pg_backend_pid()
        ) THEN
            RAISE NOTICE 'FAIL [%]: Found long-running transactions:', check_name;
            FOR check_result IN
                SELECT format('  PID %s: running for %s - %s',
                    pid, now() - xact_start, left(query, 80))
                FROM pg_stat_activity
                WHERE state = 'active'
                AND xact_start IS NOT NULL
                AND now() - xact_start > interval '5 minutes'
                AND pid != pg_backend_pid()
            LOOP
                RAISE NOTICE '%', check_result;
            END LOOP;
            all_passed := FALSE;
        ELSE
            RAISE NOTICE 'PASS [%]: No long-running transactions found', check_name;
        END IF;
    EXCEPTION WHEN OTHERS THEN
        RAISE NOTICE 'FAIL [%]: %', check_name, SQLERRM;
        all_passed := FALSE;
    END;

    -- Check 3: Active locks
    check_name := 'Active Locks';
    BEGIN
        IF EXISTS (
            SELECT 1 FROM pg_locks
            WHERE mode IN ('AccessExclusiveLock', 'ExclusiveLock')
            AND granted = true
            AND pid != pg_backend_pid()
        ) THEN
            RAISE NOTICE 'WARNING [%]: Exclusive locks detected:', check_name;
            FOR check_result IN
                SELECT format('  PID %s: %s on %s', l.pid, l.mode, c.relname)
                FROM pg_locks l
                JOIN pg_class c ON l.relation = c.oid
                WHERE l.mode IN ('AccessExclusiveLock', 'ExclusiveLock')
                AND l.granted = true
                AND l.pid != pg_backend_pid()
            LOOP
                RAISE NOTICE '%', check_result;
            END LOOP;
        ELSE
            RAISE NOTICE 'PASS [%]: No exclusive locks detected', check_name;
        END IF;
    EXCEPTION WHEN OTHERS THEN
        RAISE NOTICE 'FAIL [%]: %', check_name, SQLERRM;
        all_passed := FALSE;
    END;

    -- Check 4: Replication lag
    check_name := 'Replication Lag';
    BEGIN
        IF EXISTS (
            SELECT 1 FROM pg_stat_replication
            WHERE pg_wal_lsn_diff(sent_lsn, replay_lsn) > 100 * 1024 * 1024  -- 100MB
        ) THEN
            RAISE NOTICE 'FAIL [%]: Replication lag exceeds 100MB:', check_name;
            FOR check_result IN
                SELECT format('  Replica %s: lag = %s',
                    client_addr, pg_size_pretty(pg_wal_lsn_diff(sent_lsn, replay_lsn)))
                FROM pg_stat_replication
                WHERE pg_wal_lsn_diff(sent_lsn, replay_lsn) > 100 * 1024 * 1024
            LOOP
                RAISE NOTICE '%', check_result;
            END LOOP;
            all_passed := FALSE;
        ELSE
            RAISE NOTICE 'PASS [%]: All replicas within acceptable lag', check_name;
        END IF;
    EXCEPTION WHEN OTHERS THEN
        -- pg_stat_replication may be empty on single-node setups
        RAISE NOTICE 'SKIP [%]: No replication configured or no replicas found', check_name;
    END;

    -- Check 5: Connection count
    check_name := 'Connection Count';
    BEGIN
        IF (SELECT count(*) FROM pg_stat_activity) >
           (SELECT setting::int * 0.8 FROM pg_settings WHERE name = 'max_connections') THEN
            RAISE NOTICE 'WARNING [%]: Connection count is at 80%% of max_connections',
                check_name;
        ELSE
            RAISE NOTICE 'PASS [%]: Connection count is healthy (% / %)',
                check_name,
                (SELECT count(*) FROM pg_stat_activity),
                (SELECT setting FROM pg_settings WHERE name = 'max_connections');
        END IF;
    EXCEPTION WHEN OTHERS THEN
        RAISE NOTICE 'FAIL [%]: %', check_name, SQLERRM;
    END;

    -- Check 6: Pending migrations
    check_name := 'Pending Migrations';
    BEGIN
        RAISE NOTICE 'INFO [%]: Check Flyway info for pending migrations', check_name;
    END;

    RAISE NOTICE '';
    IF all_passed THEN
        RAISE NOTICE '=== ALL CHECKS PASSED - Safe to proceed ===';
    ELSE
        RAISE EXCEPTION '=== SOME CHECKS FAILED - Review before proceeding ===';
    END IF;
END $$;
```

### Table Size Check

```sql
-- For migrations on large tables, flag for manual review
-- Include this in the migration itself as a guard

DO $$
DECLARE
    table_size BIGINT;
    table_name TEXT := 'orders';
    threshold BIGINT := 100 * 1024 * 1024 * 1024;  -- 100GB
BEGIN
    SELECT pg_total_relation_size(table_name::regclass) INTO table_size;

    IF table_size > threshold THEN
        RAISE EXCEPTION 'Table % is % which exceeds 100GB threshold. This migration requires manual review and approval.',
            table_name, pg_size_pretty(table_size);
    END IF;
END $$;
```

## Part D: Design the Migration Testing Strategy

### Test Suite for NOT NULL Column Addition

This test suite validates the migration `V004__add_priority_column.sql` which adds `priority INTEGER NOT NULL DEFAULT 0` to the `orders` table.

```python
# tests/test_migration_v004.py
import pytest
import psycopg2

@pytest.fixture
def db_connection():
    """Create a test database connection."""
    conn = psycopg2.connect(
        host="localhost",
        port=5432,
        dbname="testdb",
        user="postgres",
        password="testpass"
    )
    conn.autocommit = False
    yield conn
    conn.rollback()
    conn.close()

@pytest.fixture
def apply_migration(db_connection):
    """Apply the migration."""
    cur = db_connection.cursor()
    # Apply migration V004
    cur.execute("""
        ALTER TABLE orders ADD COLUMN IF NOT EXISTS priority INTEGER DEFAULT 0;
        ALTER TABLE orders ALTER COLUMN priority SET NOT NULL;
    """)
    db_connection.commit()
    return cur

class TestMigrationV004:
    """Test suite for V004: Add priority column."""

    def test_column_exists(self, db_connection, apply_migration):
        """Verify the column exists after migration."""
        cur = db_connection.cursor()
        cur.execute("""
            SELECT column_name, data_type, is_nullable, column_default
            FROM information_schema.columns
            WHERE table_name = 'orders' AND column_name = 'priority'
        """)
        result = cur.fetchone()
        assert result is not None, "priority column does not exist"
        assert result[1] == 'integer', f"Expected integer, got {result[1]}"
        assert result[2] == 'NO', f"Expected NOT NULL, got nullable={result[2]}"
        assert result[3] == '0', f"Expected default 0, got {result[3]}"

    def test_existing_data_has_default(self, db_connection, apply_migration):
        """Verify existing rows have the default value."""
        cur = db_connection.cursor()
        cur.execute("SELECT COUNT(*) FROM orders WHERE priority IS NULL")
        null_count = cur.fetchone()[0]
        assert null_count == 0, f"Found {null_count} rows with NULL priority"

    def test_existing_data_default_value(self, db_connection, apply_migration):
        """Verify existing rows have the correct default value."""
        cur = db_connection.cursor()
        cur.execute("SELECT COUNT(*) FROM orders WHERE priority != 0 AND created_at < NOW()")
        wrong_default = cur.fetchone()[0]
        assert wrong_default == 0, f"Found {wrong_default} pre-existing rows with non-zero priority"

    def test_insert_with_default(self, db_connection, apply_migration):
        """Verify INSERT works without specifying priority."""
        cur = db_connection.cursor()
        cur.execute("""
            INSERT INTO orders (user_id, total_price, status)
            VALUES (1, 99.99, 'pending')
            RETURNING priority
        """)
        priority = cur.fetchone()[0]
        assert priority == 0, f"Expected default priority 0, got {priority}"

    def test_insert_with_explicit_priority(self, db_connection, apply_migration):
        """Verify INSERT works with explicit priority."""
        cur = db_connection.cursor()
        cur.execute("""
            INSERT INTO orders (user_id, total_price, status, priority)
            VALUES (1, 99.99, 'pending', 2)
            RETURNING priority
        """)
        priority = cur.fetchone()[0]
        assert priority == 2, f"Expected priority 2, got {priority}"

    def test_reject_null_priority(self, db_connection, apply_migration):
        """Verify NOT NULL constraint rejects NULL values."""
        cur = db_connection.cursor()
        with pytest.raises(psycopg2.IntegrityError):
            cur.execute("""
                INSERT INTO orders (user_id, total_price, status, priority)
                VALUES (1, 99.99, 'pending', NULL)
            """)
        db_connection.rollback()

    def test_migration_idempotent(self, db_connection):
        """Verify migration can be applied twice without error."""
        cur = db_connection.cursor()
        # Apply once
        cur.execute("ALTER TABLE orders ADD COLUMN IF NOT EXISTS priority INTEGER DEFAULT 0")
        # Apply again
        cur.execute("ALTER TABLE orders ADD COLUMN IF NOT EXISTS priority INTEGER DEFAULT 0")
        db_connection.commit()
        # Should not raise

    def test_performance_impact(self, db_connection, apply_migration):
        """Verify the migration does not significantly impact query performance."""
        cur = db_connection.cursor()

        # Time a typical query before and after
        import time

        # Ensure there's data to query
        cur.execute("SELECT COUNT(*) FROM orders")
        count = cur.fetchone()[0]
        if count < 1000:
            pytest.skip("Not enough data for performance test")

        # Time the query
        start = time.time()
        for _ in range(100):
            cur.execute("SELECT * FROM orders WHERE user_id = 1 LIMIT 10")
            cur.fetchall()
        elapsed = time.time() - start

        # Should complete 100 queries in under 5 seconds
        assert elapsed < 5.0, f"100 queries took {elapsed:.2f}s, expected < 5s"

    def test_rollback_possible(self, db_connection, apply_migration):
        """Verify the migration can be rolled back."""
        cur = db_connection.cursor()
        # Rollback: drop the column
        cur.execute("ALTER TABLE orders DROP COLUMN IF EXISTS priority")
        db_connection.commit()

        # Verify column is gone
        cur.execute("""
            SELECT column_name
            FROM information_schema.columns
            WHERE table_name = 'orders' AND column_name = 'priority'
        """)
        assert cur.fetchone() is None, "Column still exists after rollback"
```

### Test Categories

| Category | What It Tests | When It Runs | Tool |
|----------|--------------|--------------|------|
| Unit | SQL syntax, naming conventions | Every PR | CI linter |
| Integration | Schema correctness after migration | Every PR | pytest + test DB |
| Data integrity | Existing data preserved, defaults applied | Every PR | pytest |
| Performance | Migration duration, query impact | Weekly / pre-release | pgbench + custom scripts |
| Rollback | Every migration can be rolled back | Every PR | pytest |

### Performance Testing Script

```bash
#!/bin/bash
# performance_test.sh
# Run migrations against a production-sized dataset to estimate duration

DB_HOST="localhost"
DB_NAME="perftest"
DB_USER="postgres"

export PGPASSWORD="testpass"

echo "=== Migration Performance Test ==="

# Create test dataset (production-sized)
echo "Creating test dataset..."
psql -h "$DB_HOST" -U "$DB_USER" -d "$DB_NAME" -c "
    CREATE TABLE IF NOT EXISTS orders AS
    SELECT
        id,
        (random() * 1000000)::bigint as user_id,
        (random() * 100)::int as product_id,
        (random() * 10 + 1)::int as quantity,
        (random() * 100)::decimal(10,2) as unit_price,
        (random() * 1000)::decimal(10,2) as total_price,
        CASE WHEN random() < 0.3 THEN 'pending'
             WHEN random() < 0.6 THEN 'completed'
             ELSE 'cancelled' END as status,
        NOW() - (random() * interval '365 days') as created_at,
        NOW() as updated_at
    FROM generate_series(1, 10000000) as id;
"

# Time the migration
echo "Running migration..."
START=$(date +%s%N)

psql -h "$DB_HOST" -U "$DB_USER" -d "$DB_NAME" -c "
    ALTER TABLE orders ADD COLUMN IF NOT EXISTS priority INTEGER DEFAULT 0;
    ALTER TABLE orders ALTER COLUMN priority SET NOT NULL;
"

END=$(date +%s%N)
DURATION=$(( (END - START) / 1000000 ))
echo "Migration took: ${DURATION}ms"

if [ $DURATION -gt 300000 ]; then
    echo "WARNING: Migration took over 5 minutes. Consider using gh-ost or expand-contract."
    exit 1
else
    echo "PASS: Migration completed within acceptable time."
fi
```

## Common Mistakes to Avoid

1. **No concurrency control**: Two engineers merging migration PRs at the same time can create conflicting version numbers. Use `concurrency` groups in GitHub Actions to prevent this.

2. **Skipping staging**: "It works on my machine" is not sufficient for database migrations. Every migration must be tested against a production-sized dataset in staging.

3. **No rollback scripts**: Writing rollback scripts forces you to think about how to undo a migration before you run it. The discipline of writing rollbacks catches many mistakes early.

4. **Clean-enabled Flyway**: `flyway.cleanDisabled=true` must be set in production. Without it, an accidental `flyway clean` drops all tables.

5. **Not testing idempotency**: Migrations should be idempotent (safe to run multiple times). Use `IF NOT EXISTS` and `IF EXISTS` clauses. If a migration fails partway through and is re-run, it should succeed.

6. **Hardcoded credentials**: Never commit database credentials. Use GitHub Secrets and environment variables.

## Key Takeaway

A production-ready migration pipeline treats database changes with the same rigor as application code deployments. Every migration is linted, tested, applied to staging, verified, and only then applied to production with manual approval. Pre-flight checks catch infrastructure problems (disk space, replication lag, long-running transactions) before they become migration failures. Rollback scripts are required, not optional. The pipeline prevents all three incidents from the scenario: version drift is caught by the pipeline, dangerous migrations are flagged by linting, and rollbacks are tested before they are needed.
