# Cheatsheet: Database Migration

## Migration Tools

| Tool | Language | Best For |
|------|----------|----------|
| Flyway | Java/SQL | Enterprise, SQL-first |
| Alembic | Python | SQLAlchemy projects |
| golang-migrate | Go | Go projects |
| Prisma Migrate | TypeScript | Prisma ORM |
| Rails ActiveRecord | Ruby | Rails projects |

## Zero-Downtime Migration Pattern

### Expand and Contract
```
Phase 1: Expand (add new, keep old)
  - Add new column (nullable)
  - Deploy code that writes to both
  - Backfill existing data

Phase 2: Contract (remove old)
  - Deploy code that reads from new only
  - Drop old column
```

## Example: Add Column Safely
```sql
-- Phase 1: Add column (nullable)
ALTER TABLE users ADD COLUMN email_normalized VARCHAR(255);

-- Phase 2: Backfill
UPDATE users SET email_normalized = LOWER(email);

-- Phase 3: Add NOT NULL constraint
ALTER TABLE users ALTER COLUMN email_normalized SET NOT NULL;

-- Phase 4: Drop old column (after code updated)
ALTER TABLE users DROP COLUMN email;
```

## Best Practices
- Always test migrations on staging first
- Backup before migration
- Make migrations idempotent
- Never modify existing migrations
- Use transactions when possible
- Monitor performance during migration
