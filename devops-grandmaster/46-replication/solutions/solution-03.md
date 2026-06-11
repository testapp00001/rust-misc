# Solution 03: Multi-Master Conflict Resolution

---

## Part A: Conflict Classification

| Scenario | Conflict Type | Database Detectable? | Potential Corruption |
|----------|--------------|---------------------|---------------------|
| 1: Phone vs email update | Update-update (different columns) | No -- different columns, no constraint violation | Silent inconsistency: phone and email may be from different "versions" of the record |
| 2: Duplicate contact insert | Insert-insert (unique constraint) | Yes -- unique constraint on email | Duplicate records with same email, breaking application logic |
| 3: Delete vs update | Delete-update | Partially -- FK constraints can detect | Lost update: changes made in London are silently discarded |
| 4: Counter increment | Counter conflict | No -- both are valid increments | Lead score is wrong: one increment is lost |
| 5: Bulk update vs single update | Write skew | No -- both updates are valid individually | London customers may have incorrect addresses if the bulk update overwrites the individual update |

### Detailed Analysis

**Scenario 1 (Update-Update, Different Columns):** This is the trickiest conflict because the database sees no error -- both updates are on different columns. New York updates `phone`, London updates `email`. Both succeed. The result is a row with New York's phone and London's email, which may be inconsistent if the user intended both changes to be part of the same "profile update."

**Scenario 2 (Insert-Insert):** The database detects this via the unique constraint on email. Both inserts cannot succeed. The resolution must choose one insert and discard the other, or merge the data.

**Scenario 3 (Delete-Update):** The delete removes the row that London is updating. When London's update arrives at New York, the row does not exist. The update is silently ignored, and London's changes are lost.

**Scenario 4 (Counter):** Both offices increment the lead score. If New York adds 5 and London adds 3, the expected result is +8. With last-writer-wins, the result is either +5 or +3, losing one increment.

**Scenario 5 (Write Skew):** London updates customer A's address to "123 Main St." New York runs `UPDATE customers SET region = 'US' WHERE region = 'UK'`. Both are valid individually, but together they produce an inconsistent result: customer A has region = 'US' but a London address.

## Part B: Resolution Strategies

### Strategy 1: Last-Writer-Wins (LWW)

**How it works:** Each write has a timestamp. When a conflict is detected, the write with the later timestamp wins.

```sql
-- Add timestamp column to all replicated tables
ALTER TABLE customers ADD COLUMN updated_at TIMESTAMP DEFAULT now();
ALTER TABLE customers ADD COLUMN updated_by VARCHAR(50);

-- Trigger to update timestamp on every write
CREATE OR REPLACE FUNCTION update_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    NEW.updated_by = current_setting('app.node_id');
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_update_timestamp
    BEFORE UPDATE ON customers
    FOR EACH ROW EXECUTE FUNCTION update_timestamp();
```

**When to use:** Scenario 1 (different column updates) -- LWW is acceptable because the updates are independent and the "latest" version is usually the most correct.

**What is lost:** The earlier write is completely discarded. For scenario 4 (counters), this loses an increment.

### Strategy 2: Application-Level Merge

**How it works:** When a conflict is detected, the application merges both writes into a single result.

```python
def merge_customer_updates(local_row, remote_row):
    """Merge two conflicting customer updates."""
    merged = {}

    # For each column, prefer the more recent update
    for column in ['phone', 'email', 'address', 'lead_score']:
        local_ts = local_row.get(f'{column}_updated_at')
        remote_ts = remote_row.get(f'{column}_updated_at')

        if local_ts and remote_ts:
            if local_ts > remote_ts:
                merged[column] = local_row[column]
            else:
                merged[column] = remote_row[column]
        elif local_ts:
            merged[column] = local_row[column]
        elif remote_ts:
            merged[column] = remote_row[column]
        else:
            merged[column] = local_row[column]

    # Special handling for counters: ADD, not replace
    merged['lead_score'] = local_row['lead_score_delta'] + remote_row['lead_score_delta']

    return merged
```

**When to use:** Scenario 4 (counters) and scenario 1 (different columns). Merge preserves both changes.

**What is lost:** Nothing, if implemented correctly. But merge logic is complex and error-prone.

### Strategy 3: Partitioning (Conflict Avoidance)

**How it works:** Assign data ownership by region. Each region only writes to its own data.

```python
# Application-level routing
def get_write_region(customer_id):
    """Determine which region owns this customer."""
    # Hash-based partitioning
    return hash(customer_id) % 2  # 0 = New York, 1 = London

def route_write(customer_id, operation):
    region = get_write_region(customer_id)
    if region == CURRENT_REGION:
        return execute_local(operation)
    else:
        return execute_remote(region, operation)
```

**When to use:** Scenario 5 (write skew). If London customers are owned by London and US customers by US, the bulk update in New York only affects New York-owned customers.

**What is lost:** Cross-region collaboration. A London sales rep cannot update a New York customer without routing through New York.

### Strategy 4: First-Writer-Wins

**How it works:** The first write to arrive at all nodes wins. Subsequent conflicting writes are rejected.

**When to use:** Scenario 2 (duplicate inserts). The first contact creation wins; the second is rejected with a unique constraint violation.

**What is lost:** The second write is lost, but the user is notified (unlike LWW, which silently discards).

## Part C: Version Vector Implementation

```python
from typing import Dict, Optional
from dataclasses import dataclass, field
import json


@dataclass
class VersionVector:
    """Tracks the version of a record across multiple nodes."""
    versions: Dict[str, int] = field(default_factory=dict)

    def increment(self, node_id: str) -> None:
        """Increment the version counter for a node."""
        self.versions[node_id] = self.versions.get(node_id, 0) + 1

    def dominates(self, other: 'VersionVector') -> bool:
        """Check if this version vector dominates another.

        A dominates B if A is >= B in all dimensions and > B in at least one.
        """
        all_keys = set(self.versions.keys()) | set(other.versions.keys())
        has_greater = False

        for key in all_keys:
            self_val = self.versions.get(key, 0)
            other_val = other.versions.get(key, 0)

            if self_val < other_val:
                return False
            if self_val > other_val:
                has_greater = True

        return has_greater

    def is_concurrent(self, other: 'VersionVector') -> bool:
        """Check if two version vectors are concurrent (conflict)."""
        return not self.dominates(other) and not other.dominates(self)

    def merge(self, other: 'VersionVector') -> 'VersionVector':
        """Merge two version vectors (take max of each dimension)."""
        all_keys = set(self.versions.keys()) | set(other.versions.keys())
        merged = {}
        for key in all_keys:
            merged[key] = max(self.versions.get(key, 0), other.versions.get(key, 0))
        return VersionVector(versions=merged)

    def to_dict(self) -> dict:
        return self.versions

    @classmethod
    def from_dict(cls, d: dict) -> 'VersionVector':
        return cls(versions=d)


@dataclass
class VersionedRecord:
    """A record with a version vector and data."""
    record_id: str
    data: dict
    vector: VersionVector
    deleted: bool = False


class ConflictResolver:
    """Detects and resolves conflicts using version vectors."""

    def __init__(self, node_id: str, strategy: str = 'lww'):
        self.node_id = node_id
        self.strategy = strategy

    def write(self, record: VersionedRecord, new_data: dict) -> VersionedRecord:
        """Apply a write to a record, incrementing the local version."""
        record.vector.increment(self.node_id)
        record.data.update(new_data)
        record.data['updated_at'] = time.time()
        record.data['updated_by'] = self.node_id
        return record

    def detect_conflict(self, local: VersionedRecord,
                        remote: VersionedRecord) -> bool:
        """Check if two versions of the same record are in conflict."""
        return local.vector.is_concurrent(remote.vector)

    def resolve(self, local: VersionedRecord,
                remote: VersionedRecord) -> VersionedRecord:
        """Resolve a conflict between local and remote versions."""
        if not self.detect_conflict(local, remote):
            # No conflict -- one dominates the other
            if local.vector.dominates(remote.vector):
                return local
            else:
                return remote

        # Conflict detected -- apply resolution strategy
        if self.strategy == 'lww':
            return self._resolve_lww(local, remote)
        elif self.strategy == 'merge':
            return self._resolve_merge(local, remote)
        else:
            raise ValueError(f"Unknown strategy: {self.strategy}")

    def _resolve_lww(self, local: VersionedRecord,
                     remote: VersionedRecord) -> VersionedRecord:
        """Last-writer-wins: the write with the later timestamp wins."""
        local_ts = local.data.get('updated_at', 0)
        remote_ts = remote.data.get('updated_at', 0)

        if local_ts >= remote_ts:
            winner = local
        else:
            winner = remote

        # Merge version vectors (always merge, even if one write wins)
        winner.vector = local.vector.merge(remote.vector)
        return winner

    def _resolve_merge(self, local: VersionedRecord,
                       remote: VersionedRecord) -> VersionedRecord:
        """Application-level merge: combine both writes."""
        merged_data = {}

        # For each field, take the value from the more recent write
        all_keys = set(local.data.keys()) | set(remote.data.keys())
        for key in all_keys:
            if key in ('updated_at', 'updated_by'):
                continue

            local_val = local.data.get(key)
            remote_val = remote.data.get(key)

            # Special handling for numeric fields (add deltas)
            if isinstance(local_val, (int, float)) and isinstance(remote_val, (int, float)):
                # If both modified the same number, take the max (or sum for counters)
                merged_data[key] = max(local_val, remote_val)
            else:
                # For strings, take the more recent value
                local_ts = local.data.get('updated_at', 0)
                remote_ts = remote.data.get('updated_at', 0)
                merged_data[key] = local_val if local_ts >= remote_ts else remote_val

        merged_record = VersionedRecord(
            record_id=local.record_id,
            data=merged_data,
            vector=local.vector.merge(remote.vector)
        )
        merged_record.data['updated_at'] = max(
            local.data.get('updated_at', 0),
            remote.data.get('updated_at', 0)
        )
        merged_record.data['resolved_by'] = 'merge'
        return merged_record


# Example usage
import time

# New York node
ny_resolver = ConflictResolver(node_id='new_york', strategy='merge')

# Create a record
record = VersionedRecord(
    record_id='customer_123',
    data={'name': 'Alice', 'phone': '212-555-0100', 'email': 'alice@old.com'},
    vector=VersionVector()
)

# New York updates phone
ny_resolver.write(record, {'phone': '212-555-0200'})

# Simulate London's concurrent update (arrives later)
london_record = VersionedRecord(
    record_id='customer_123',
    data={'name': 'Alice', 'phone': '212-555-0100', 'email': 'alice@new.co.uk',
          'updated_at': time.time() - 1, 'updated_by': 'london'},
    vector=VersionVector(versions={'london': 1})
)

# Detect and resolve conflict
if ny_resolver.detect_conflict(record, london_record):
    resolved = ny_resolver.resolve(record, london_record)
    print(f"Conflict detected and resolved: {resolved.data}")
else:
    print("No conflict -- one version dominates the other")
```

### How the Version Vector Detects Conflicts

A version vector is a map `{node_id: counter}`. When node A writes, it increments its own counter. When two nodes modify the same record independently, their version vectors become concurrent:

```
After New York writes:  {new_york: 1}
After London writes:    {london: 1}

Neither dominates the other (new_york has 1 but london has 0; london has 1 but new_york has 0).
This is a conflict.
```

After resolution, the merged vector is `{new_york: 1, london: 1}`, which dominates both originals.

## Common Mistakes to Avoid

1. **Using only timestamps for conflict detection** -- clock skew between regions causes incorrect ordering
2. **Not merging version vectors after resolution** -- future conflict detection will be wrong
3. **Treating all conflicts the same way** -- counter conflicts need different resolution than string conflicts
4. **Ignoring delete-delete conflicts** -- two nodes deleting the same record is still a conflict that needs handling

## Key Takeaway

Multi-master conflict resolution requires two layers: detection (version vectors) and resolution (strategy). Version vectors correctly identify concurrent writes regardless of clock skew. The resolution strategy depends on the data semantics: LWW for simple fields, merge for counters, partitioning to avoid conflicts entirely. The key insight is that you cannot have both multi-master writes and strong consistency -- you must choose your trade-off.
