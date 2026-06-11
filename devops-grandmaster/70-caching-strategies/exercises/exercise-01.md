# Exercise 01: Cache Pattern Selection

**Type:** Conceptual
**Time:** 15 minutes
**Difficulty:** Easy

## Objective
Understand the differences between cache-aside, write-through, and write-behind patterns. Select the appropriate pattern for different use cases and explain the trade-offs.

## Scenario
You are designing the caching strategy for three different applications:

1. **E-commerce product catalog**: Products are read frequently (10,000 reads/sec) but updated rarely (10 writes/day). Stale data is acceptable for up to 5 minutes.

2. **Banking transaction system**: Transactions must be immediately consistent. Every write must be visible to subsequent reads. Write latency must be minimized.

3. **Social media feed**: Posts are written frequently (1,000 writes/sec) and read even more frequently (50,000 reads/sec). Users can tolerate a few seconds of staleness.

## Tasks

### Part A: Match Patterns to Use Cases
For each application above, select the most appropriate cache pattern (cache-aside, write-through, or write-behind) and explain why.

<details>
<summary>Hint</summary>
Cache-aside works well when reads dominate writes and staleness is acceptable. Write-through ensures consistency but adds write latency. Write-behind minimizes write latency but risks data loss.
</details>

### Part B: Explain Trade-offs
For the e-commerce product catalog (cache-aside), explain:
1. What happens when a product is updated?
2. How long might stale data be served?
3. What happens if the cache is cold (empty)?

<details>
<summary>Hint</summary>
With cache-aside, the application manages the cache. On update, the application must invalidate the cache entry. The next read will miss the cache, query the database, and repopulate the cache. If the application forgets to invalidate, stale data persists until TTL expires.
</details>

### Part C: Design Invalidation Strategy
For each application, design a cache invalidation strategy:
1. E-commerce: TTL-based, event-based, or version-based?
2. Banking: How to ensure immediate consistency?
3. Social media: How to handle the thundering herd on popular posts?

<details>
<summary>Hint</summary>
E-commerce uses TTL (5-minute expiry) plus event-based invalidation on product update. Banking uses write-through (cache always has latest data). Social media uses TTL plus lock-based stampede prevention.
</details>

## Success Criteria
- [ ] You can explain when to use cache-aside, write-through, and write-behind patterns.
- [ ] You can match each pattern to the appropriate use case with justification.
- [ ] You can describe the trade-offs of each pattern (latency, consistency, complexity).
- [ ] You can design an invalidation strategy for different consistency requirements.
- [ ] You can explain what happens when the cache is cold or unavailable.

## What You Should Understand After This Exercise
There is no "best" cache pattern -- only the best pattern for a given use case. Cache-aside is the most common and flexible. Write-through guarantees consistency at the cost of write latency. Write-behind minimizes write latency at the cost of potential data loss. The choice depends on your consistency requirements, read/write ratio, and tolerance for staleness.
