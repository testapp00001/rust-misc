# Exercise 04: Query Optimization Audit

**Type:** Challenge
**Time:** 45 minutes
**Difficulty:** Medium-Hard

## Objective
Analyze slow queries, interpret execution plans, and apply optimization techniques to improve database performance.

## Scenario
You have inherited a PostgreSQL database with several slow queries that are degrading application performance. The `pg_stat_statements` extension shows the following problematic queries:

```sql
-- Query 1: Product search (avg 850ms, called 10,000x/hour)
SELECT p.*, c.name as category_name, AVG(r.rating) as avg_rating
FROM products p
JOIN categories c ON p.category_id = c.id
LEFT JOIN reviews r ON r.product_id = p.id
WHERE p.name ILIKE '%wireless%'
  AND p.price BETWEEN 10 AND 100
GROUP BY p.id, c.name
ORDER BY avg_rating DESC NULLS LAST
LIMIT 20;

-- Query 2: User order history (avg 1200ms, called 5,000x/hour)
SELECT o.id, o.created_at, o.total,
       array_agg(oi.product_name) as items
FROM orders o
JOIN order_items oi ON oi.order_id = o.id
WHERE o.user_id = 12345
  AND o.created_at > NOW() - INTERVAL '1 year'
GROUP BY o.id
ORDER BY o.created_at DESC;

-- Query 3: Inventory check (avg 200ms, called 50,000x/hour)
SELECT warehouse_id, quantity
FROM inventory
WHERE product_id = 67890
  AND quantity > 0;
```

## Tasks

### Part A: Analyze Execution Plans
For each query above, explain what you would look for in the `EXPLAIN ANALYZE` output. Identify at least two potential performance problems per query.

<details>
<summary>Hint</summary>
Look for: sequential scans on large tables, missing index usage, high row count estimates vs actual, expensive sorts, and nested loop joins on large datasets. Query 1 has an ILIKE with leading wildcard that prevents index usage.
</details>

### Part B: Design Indexes
Create indexes that would improve each query. For each index:
1. Write the `CREATE INDEX` statement
2. Explain why this index helps
3. Identify any trade-offs (write overhead, storage)

<details>
<summary>Hint</summary>
Consider GIN indexes for text search (ILIKE), composite indexes for multi-column WHERE clauses, and covering indexes to avoid table lookups. For Query 1, the ILIKE with leading '%' cannot use a B-tree index -- you need `pg_trgm` with a GIN index.
</details>

### Part C: Rewrite Queries
For each query, rewrite it to be more efficient. You may:
- Restructure JOINs
- Use CTEs or subqueries strategically
- Add query hints or optimizations
- Suggest schema changes

Explain why each rewrite improves performance.

<details>
<summary>Hint</summary>
For Query 2, consider whether `array_agg` forces a full scan. For Query 3, an index-only scan is possible if the index covers both columns. Consider materialized views for Query 1 if the results do not need to be real-time.
</details>

## Success Criteria
- [ ] Each query has at least two identified performance problems
- [ ] Indexes are syntactically correct and well-justified
- [ ] Query rewrites produce equivalent results
- [ ] Trade-offs for each optimization are discussed
- [ ] At least one query suggests a schema-level change

## What You Should Understand After This Exercise
Query optimization requires reading execution plans, understanding index types, and knowing when to rewrite queries versus add indexes. Not all queries can be fixed with indexes alone -- some need schema changes, materialized views, or application-level caching. Always measure before and after with `EXPLAIN ANALYZE`.
