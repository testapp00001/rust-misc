# Module 50: Database Migration -- Exercises

Database migrations are one of the most dangerous operations in production systems. A poorly planned migration can cause hours of downtime, data loss, or cascading failures. This module teaches you to perform schema changes safely on live systems with millions of rows and zero downtime.

## Why This Matters

Every application evolves. Columns are added, types change, indexes are created, and tables are restructured. In a development environment these changes are trivial. In production with active traffic, they become one of the highest-risk operations a database administrator or platform engineer performs. Understanding safe migration patterns is the difference between a 2 AM incident and a routine deployment.

## Exercise List

| # | Name | Type | Time | Difficulty |
|---|------|------|------|------------|
| 01 | Migration Risk Assessment | Conceptual | 15 min | Easy |
| 02 | Online Schema Change with gh-ost | Guided | 30 min | Easy-Medium |
| 03 | Zero-Downtime Column Migration | Independent | 30 min | Medium |
| 04 | Large Table Migration Strategy | Challenge | 45 min | Medium-Hard |
| 05 | Complete Migration Pipeline | Integration | 45 min | Hard |

## How to Use These Exercises

1. Start with Exercise 01 to build foundational understanding of migration risk levels.
2. Work through each exercise in order; later exercises build on concepts from earlier ones.
3. Attempt each exercise before checking the solutions.
4. The solutions directory contains detailed walkthroughs with real commands and SQL.

## Prerequisites

- Familiarity with SQL (DDL and DML)
- Basic understanding of database transactions and locking
- Command-line comfort (bash/terminal)
- Access to a MySQL or PostgreSQL instance for hands-on exercises (Docker is fine)

## Environment Setup

```bash
# Start a MySQL instance for exercises
docker run -d \
  --name mysql-migration-lab \
  -e MYSQL_ROOT_PASSWORD=rootpass \
  -e MYSQL_DATABASE=testdb \
  -p 3306:3306 \
  mysql:8.0

# Or start a PostgreSQL instance
docker run -d \
  --name postgres-migration-lab \
  -e POSTGRES_PASSWORD=rootpass \
  -e POSTGRES_DB=testdb \
  -p 5432:5432 \
  postgres:16

# Connect to MySQL
docker exec -it mysql-migration-lab mysql -uroot -prootpass testdb

# Connect to PostgreSQL
docker exec -it postgres-migration-lab psql -U postgres testdb
```
