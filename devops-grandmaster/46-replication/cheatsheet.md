# Cheatsheet: Database Replication

## Replication Types

| Type | Consistency | Performance | Use Case |
|------|-------------|-------------|----------|
| Async | Eventual | High | Most applications |
| Sync | Strong | Lower | Financial data |
| Semi-sync | Balance | Medium | Critical reads |

## PostgreSQL Streaming Replication
```bash
# On primary
wal_level = replica
max_wal_senders = 5
wal_keep_size = '1GB'

# Create replication user
CREATE USER replicator WITH REPLICATION ENCRYPTED 'password';

# On replica
primary_conninfo = 'host=primary port=5432 user=replicator password=password'
hot_standby = on
```

## MySQL Replication
```sql
-- On primary
server-id = 1
log_bin = mysql-bin
binlog_format = ROW

-- Create replication user
CREATE USER 'repl'@'%' IDENTIFIED BY 'password';
GRANT REPLICATION SLAVE ON *.* TO 'repl'@'%';

-- On replica
server-id = 2
relay_log = relay-bin
read_only = ON
```

## Monitoring Replication
```sql
-- PostgreSQL
SELECT * FROM pg_stat_replication;

-- MySQL
SHOW SLAVE STATUS;
```
