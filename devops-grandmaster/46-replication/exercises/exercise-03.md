# Exercise 03: Multi-Master Conflict Resolution

**Type:** Independent | **Time:** 30 min | **Difficulty:** Medium

## Objective
Design conflict resolution strategies for multi-master database setups where concurrent writes to the same data can occur in different locations.

## Scenario
Your company runs a customer relationship management (CRM) system used by sales teams in New York and London. Both offices need read-write access to the same customer records. The system uses two PostgreSQL instances with bi-directional logical replication. During a recent week, the following conflicts were reported:

1. A sales rep in New York updated a customer's phone number while a London rep updated the same customer's email
2. Both offices created a new contact for the same person (same email address)
3. New York deleted a customer record that London was still updating
4. Both offices incremented the same customer's lead score by different amounts
5. London updated a customer's address while New York was running a bulk update on all London customers

## Tasks

### Part A: Classify Conflict Types
For each of the 5 scenarios above, identify:
1. The type of conflict (update-update, insert-insert, delete-update, counter, write skew)
2. Whether the conflict is detectable at the database level or requires application logic
3. The potential data corruption if the conflict is not resolved

<details>
<summary>Hint</summary>
Conflicts fall into categories: same-row update conflicts (both update the same column), unique constraint violations (both insert the same key), delete-update conflicts (one deletes while the other updates), and semantic conflicts (application-level inconsistencies that the database cannot detect).
</details>

### Part B: Design Resolution Strategies
For each conflict type, propose a resolution strategy:
1. Last-writer-wins (LWW) -- timestamp-based
2. First-writer-wins
3. Application-level merge
4. Partitioning to avoid conflicts

For each strategy, explain when it is appropriate and what data is lost.

<details>
<summary>Hint</summary>
LWW is simple but loses one write. Application-level merge preserves both writes but is complex to implement. Partitioning (e.g., by region) avoids conflicts entirely but limits collaboration. Counter conflicts require CRDTs (Conflict-free Replicated Data Types) to merge correctly.
</details>

### Part C: Implement a Version Vector Mechanism
Write Python pseudocode for a version vector system that:
1. Tracks the version of each record per node
2. Detects conflicts when two nodes modify the same record
3. Applies the chosen resolution strategy

```python
class VersionVector:
    # Implement this class
    pass

class ConflictResolver:
    # Implement this class
    pass
```

<details>
<summary>Hint</summary>
A version vector is a map of `{node_id: version_counter}`. When a node writes, it increments its own counter. A conflict occurs when two version vectors are concurrent (neither dominates the other). The resolution strategy decides which write wins.
</details>

## Success Criteria

- [ ] All 5 conflict types are correctly classified
- [ ] Resolution strategies are appropriate for each conflict type
- [ ] Version vector implementation correctly detects concurrent writes
- [ ] Code handles at least 3 of the 5 conflict scenarios

## What You Should Understand After This Exercise

Multi-master replication introduces conflicts that single-master setups never face. Not all conflicts can be resolved at the database level -- some require application logic. The key insight is that you must choose between consistency and availability: avoiding conflicts (partitioning) maintains consistency but limits collaboration, while accepting conflicts (LWW, merge) maintains availability but may lose data.
