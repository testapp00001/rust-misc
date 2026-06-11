# Cheatsheet: Backup & Restore

## Backup Types

| Type | Speed | Size | Use Case |
|------|-------|------|----------|
| Full | Slow | Large | Weekly baseline |
| Incremental | Fast | Small | Daily backups |
| Differential | Medium | Medium | Between full backups |

## PostgreSQL Backup
```bash
# Full backup
pg_dump -U postgres mydb > backup.sql

# Custom format (parallel restore)
pg_dump -U postgres -Fc mydb > backup.dump

# Base backup for PITR
pg_basebackup -D /backup/base -Fp -Xs -P

# Restore
psql -U postgres mydb < backup.sql
pg_restore -U postgres -d mydb backup.dump
```

## Automated Backup Script
```bash
#!/bin/bash
BACKUP_DIR="/backups"
DATE=$(date +%Y%m%d_%H%M%S)

# PostgreSQL
pg_dump -U postgres mydb | gzip > "$BACKUP_DIR/postgres_$DATE.sql.gz"

# Keep last 7 daily, 4 weekly, 12 monthly
find $BACKUP_DIR -name "postgres_*.sql.gz" -mtime +7 -delete
```

## Point-in-Time Recovery (PITR)
```bash
# Restore base backup
pg_basebackup -D /var/lib/postgresql/data -Xs -P

# Configure recovery
restore_command = 'cp /backups/wal/%f %p'
recovery_target_time = '2024-01-15 10:30:00'
```

## Backup Verification
```bash
# ALWAYS test your backups!
pg_restore -U postgres -d test_db backup.dump
psql -U postgres test_db -c "SELECT count(*) FROM users;"
```
