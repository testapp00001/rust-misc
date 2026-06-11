# Solution 04: Query Optimization Audit

## Part A: Execution Plan Analysis

### Query 1: Product Search (850ms)

Expected problems:
1. **Sequential scan on products table**: The `ILIKE '%wireless%'` with a leading wildcard cannot use a standard B-tree index. PostgreSQL will scan every row in the products table.
2. **Expensive sort on computed column**: `ORDER BY avg_rating DESC` requires sorting the entire result set after aggregation. The `avg_rating` is computed from a JOIN, so no index can help with this sort.
3. **Large intermediate result set**: The LEFT JOIN with reviews may produce many rows per product before aggregation, increasing memory and CPU usage.

### Query 2: User Order History (1200ms)

Expected problems:
1. **Missing index on orders(user_id, created_at)**: Without this composite index, PostgreSQL may scan the entire orders table to find orders for user 12345.
2. **array_agg forcing full materialization**: The `array_agg(oi.product_name)` forces PostgreSQL to collect all order items before returning, preventing any streaming or early termination.
3. **Large time range**: `INTERVAL '1 year'` may include many orders for active users, producing a large result set.

### Query 3: Inventory Check (200ms)

Expected problems:
1. **Potential index miss on (product_id)**: If there is only an index on `product_id` alone, PostgreSQL must still look up `quantity` in the table (heap access).
2. **High call frequency (50,000/hour)**: Even at 200ms, this query consumes significant resources. At 50,000 calls/hour, that is ~14 calls/second, each taking 200ms = ~2.8 concurrent connections just for this query.

## Part B: Index Design

### Query 1: Product Search Indexes

```sql
-- Enable pg_trgm extension for trigram matching
CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- GIN index for ILIKE pattern matching
CREATE INDEX idx_products_name_trgm ON products USING GIN (name gin_trgm_ops);

-- Composite index for price range filtering
CREATE INDEX idx_products_price_category ON products (category_id, price);

-- Index for reviews aggregation
CREATE INDEX idx_reviews_product_id ON reviews (product_id, rating);
```

**Why this helps**: The `gin_trgm_ops` index allows PostgreSQL to use the GIN index for `ILIKE` patterns, even with leading wildcards. The composite index on `(category_id, price)` supports the `BETWEEN` filter and the JOIN. The reviews index supports the aggregation with a covering index.

**Trade-offs**: GIN indexes are larger than B-tree indexes and slower to update. Each INSERT/UPDATE on the products table must also update the GIN index, adding ~10-20% write overhead.

### Query 2: Order History Indexes

```sql
-- Composite index for user order lookup with time range
CREATE INDEX idx_orders_user_created ON orders (user_id, created_at DESC);

-- Covering index for order items
CREATE INDEX idx_order_items_order_covering ON order_items (order_id) INCLUDE (product_name);
```

**Why this helps**: The composite index `(user_id, created_at DESC)` allows PostgreSQL to seek directly to user 12345's orders and scan them in reverse chronological order. The covering index on `order_items` avoids a table lookup for `product_name`.

**Trade-offs**: The covering index includes the `product_name` column, making it larger. If `product_name` is updated frequently, the index maintenance cost increases.

### Query 3: Inventory Index

```sql
-- Covering index for index-only scan
CREATE INDEX idx_inventory_product_quantity ON inventory (product_id, quantity);
```

**Why this helps**: This covering index allows PostgreSQL to answer the query entirely from the index without accessing the table (index-only scan). This reduces I/O from 2 reads (index + table) to 1 read (index only).

**Trade-offs**: Minimal -- this index is small and the write overhead is negligible since `inventory` updates are expected.

## Part C: Query Rewrites

### Query 1 Rewrite: Product Search

```sql
-- Use a materialized view for expensive aggregations
CREATE MATERIALIZED VIEW product_ratings AS
SELECT
    p.id,
    p.name,
    p.price,
    p.category_id,
    c.name AS category_name,
    COALESCE(AVG(r.rating), 0) AS avg_rating,
    COUNT(r.id) AS review_count
FROM products p
JOIN categories c ON p.category_id = c.id
LEFT JOIN reviews r ON r.product_id = p.id
GROUP BY p.id, p.name, p.price, p.category_id, c.name;

CREATE UNIQUE INDEX idx_product_ratings_id ON product_ratings (id);
CREATE INDEX idx_product_ratings_search ON product_ratings USING GIN (name gin_trgm_ops);
CREATE INDEX idx_product_ratings_price ON product_ratings (price, avg_rating DESC);

-- Refresh periodically (e.g., every 15 minutes)
REFRESH MATERIALIZED VIEW CONCURRENTLY product_ratings;

-- Optimized query uses the materialized view
SELECT id, name, category_name, avg_rating
FROM product_ratings
WHERE name ILIKE '%wireless%'
  AND price BETWEEN 10 AND 100
ORDER BY avg_rating DESC NULLS LAST
LIMIT 20;
```

**Why this improves performance**: The expensive JOIN and aggregation are pre-computed. The materialized view has its own indexes optimized for this query pattern. `REFRESH MATERIALIZED VIEW CONCURRENTLY` updates the view without locking reads.

### Query 2 Rewrite: Order History

```sql
-- Rewrite to use a lateral join for efficient streaming
SELECT
    o.id,
    o.created_at,
    o.total,
    (
        SELECT array_agg(oi.product_name ORDER BY oi.id)
        FROM order_items oi
        WHERE oi.order_id = o.id
    ) AS items
FROM orders o
WHERE o.user_id = 12345
  AND o.created_at > NOW() - INTERVAL '1 year'
ORDER BY o.created_at DESC
LIMIT 50;
```

**Why this improves performance**: Adding `LIMIT 50` prevents PostgreSQL from processing all orders for the user. The correlated subquery for `array_agg` only runs for the 50 returned orders, not all orders. With the composite index `(user_id, created_at DESC)`, PostgreSQL can efficiently scan just the most recent orders.

### Query 3 Rewrite: Inventory Check

```sql
-- Add a partial index for the most common query pattern
CREATE INDEX idx_inventory_in_stock ON inventory (product_id, quantity)
WHERE quantity > 0;

-- The query itself is already efficient with the covering index.
-- For extreme optimization, add a cached inventory layer:
-- Use Redis to cache inventory counts with 5-second TTL
-- Only hit the database on cache miss
```

**Why this improves performance**: The partial index only includes rows where `quantity > 0`, making it smaller and faster to scan. For the 50,000 calls/hour frequency, adding a Redis cache with a 5-second TTL reduces database load by 99% while keeping inventory data reasonably fresh.

## Common Mistakes to Avoid
- Adding indexes without checking existing indexes -- you may create duplicates
- Using `CREATE INDEX` without `CONCURRENTLY` on a production table -- it locks the table
- Assuming all queries need the same optimization strategy -- some benefit from indexes, others from rewrites, others from caching
- Not running `ANALYZE` after adding indexes -- PostgreSQL statistics must be updated for the query planner to use new indexes

## Key Takeaway
Query optimization is a multi-layered problem. Execution plans reveal bottlenecks, indexes provide fast lookups, query rewrites reduce work, and materialized views pre-compute expensive operations. Always measure with `EXPLAIN ANALYZE` before and after changes, and consider the trade-off between read performance and write overhead for each index.
