# Module 14: Database & Storage

SQLx with compile-time checked queries, connection pooling, migrations, caching strategies, and embedded databases.

## Lessons

| # | File | Topic |
|---|------|-------|
| 01 | `p01_sqlx_fundamentals.rs` | Query, fetch, execute, compile-time checked queries, sqlx::query! |
| 02 | `p02_connection_pooling.rs` | Pool configuration, PgPool, SqlitePool, pool sizing, health checks |
| 03 | `p03_migrations.rs` | sqlx migrate, migration files, running migrations, rollback strategies |
| 04 | `p04_transaction_patterns.rs` | BEGIN/COMMIT/ROLLBACK, savepoints, isolation levels, retry on conflict |
| 05 | `p05_repository_pattern.rs` | Repository trait, CRUD operations, pagination, filtering, sorting |
| 06 | `p06_embedded_databases.rs` | SQLite, sled, redb, embedded use cases, zero-config databases |
| 07 | `p07_caching_strategies.rs` | In-memory caching, cache-aside, TTL, cache invalidation patterns |
| 08 | `p08_database_testing.rs` | Test databases, fixtures, cleanup, transactional tests |
| 09 | `p09_query_optimization.rs` | EXPLAIN, indexes, N+1 queries, batch loading, cursor pagination |
| 10 | `p10_storage_engines.rs` | Key-value stores, LSM trees, B-trees, storage abstraction traits |

## Running Tests

```bash
cargo test -p database_storage
```
