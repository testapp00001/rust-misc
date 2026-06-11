# Solution 05: Cross-Region Replication Architecture

---

## Part A: Replication Topology Design

### Architecture Decision: Hybrid Partitioned

The GDPR requirement (EU data stays in EU) prevents a single global primary. The architecture uses:
- **Regional primaries** for user data (partitioned by region)
- **One global primary** for shared data (product catalog, system config)
- **Read replicas** in each region for local read latency

```
                        GLOBAL SHARED DATA
                        ==================

              US-East                  EU-West              AP-Southeast
         +-----------+           +-----------+           +-----------+
         | GLOBAL    |  -------> | GLOBAL    |  -------> | GLOBAL    |
         | PRIMARY   |  async    | REPLICA   |  async    | REPLICA   |
         +-----------+           +-----------+           +-----------+


                        REGIONAL USER DATA
                        ==================

              US-East                  EU-West              AP-Southeast
         +-----------+           +-----------+           +-----------+
         | REGIONAL  |           | REGIONAL  |           | REGIONAL  |
         | PRIMARY US|           | PRIMARY EU|           | PRIMARY   |
         | (user     |           | (user     |           | APAC      |
         |  data)    |           |  data)    |           | (user     |
         +-----+-----+           +-----+-----+           |  data)    |
               |                       |                 +-----+-----+
         +-----+-----+           +-----+-----+               |
         | REPLICA US|           | REPLICA EU|          +-----+-----+
         | (US-East  |           | (EU-West  |          | REPLICA   |
         |  1b)      |           |  1b)      |          | APAC      |
         +-----------+           +-----------+          | (AP-SE-1b)|
                                                        +-----------+

              +-------------------------------------------+
              |              APPLICATION LAYER             |
              |                                           |
              |  US users  --> US-East regional primary    |
              |  EU users  --> EU-West regional primary    |
              |  APAC users -> AP-SE regional primary      |
              |                                           |
              |  All users -> nearest global replica       |
              |  (for product catalog, system config)      |
              +-------------------------------------------+
```

### Data Classification

| Data Type | Storage | Replication | Rationale |
|-----------|---------|-------------|-----------|
| User profiles, orders, sessions | Regional primary | None (isolated per region) | GDPR: EU user data stays in EU |
| Product catalog | Global primary (US-East) | Async to EU, APAC | Shared read data, low write rate |
| System configuration | Global primary (US-East) | Async to EU, APAC | Shared read data, very low write rate |
| Analytics events | Regional primary | Async to US-East warehouse | GDPR: process in region, aggregate centrally |

## Part B: Data Residency Rules

### Table Partitioning by Region

```sql
-- User table partitioned by region
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    region VARCHAR(10) NOT NULL CHECK (region IN ('US', 'EU', 'APAC')),
    email VARCHAR(255) NOT NULL,
    name VARCHAR(255),
    created_at TIMESTAMP DEFAULT now()
) PARTITION BY LIST (region);

-- Regional partitions (on respective regional primaries)
CREATE TABLE users_us PARTITION OF users FOR VALUES IN ('US');
CREATE TABLE users_eu PARTITION OF users FOR VALUES IN ('EU');
CREATE TABLE users_apac PARTITION OF users FOR VALUES IN ('APAC');
```

### Application-Level Routing

```python
import hashlib

REGION_MAP = {
    'US': 'us-east-1.db.internal',
    'EU': 'eu-west-1.db.internal',
    'APAC': 'ap-southeast-1.db.internal',
}

def get_user_region(user_id: str) -> str:
    """Determine which region owns this user."""
    # Users are assigned to regions at creation time
    # and their region never changes
    return get_region_from_db(user_id)

def route_write(user_id: str, query: str, params: tuple):
    """Route a write to the correct regional primary."""
    region = get_user_region(user_id)
    host = REGION_MAP[region]
    return execute_query(host, query, params)

def route_read(user_id: str, query: str, params: tuple):
    """Route a read to the nearest regional replica."""
    region = get_user_region(user_id)
    # Read from nearest replica in the user's region
    host = REGION_MAP[region].replace('.db.internal', '-replica.db.internal')
    return execute_query(host, query, params)

def route_global_read(query: str, params: tuple):
    """Route a read of global data to the nearest replica."""
    # Use the nearest global replica
    host = get_nearest_global_replica()
    return execute_query(host, query, params)
```

### Cross-Region Access

```sql
-- For admin access to EU data from US:
-- Use a read-only federated query through dblink or postgres_fdw

-- On the US admin database:
CREATE EXTENSION postgres_fdw;

CREATE SERVER eu_primary
    FOREIGN DATA WRAPPER postgres_fdw
    OPTIONS (host 'eu-west-1.db.internal', dbname 'users_eu', port '5432');

CREATE USER MAPPING FOR admin_user
    SERVER eu_primary
    OPTIONS (user 'readonly', password '...');

-- Create a foreign table that mirrors the EU users table
CREATE FOREIGN TABLE users_eu_view (
    id UUID,
    email VARCHAR(255),
    name VARCHAR(255)
) SERVER eu_primary OPTIONS (table_name 'users');

-- Query EU data from US (read-only, with audit logging)
SELECT * FROM users_eu_view WHERE email = 'alice@example.eu';
```

### Data That Moves Between Regions

```python
def handle_user_relocation(user_id: str, new_region: str):
    """Handle a user moving to a new region (e.g., EU user travels to US).

    GDPR note: The user's data AT REST stays in their home region.
    Only the access routing changes. The user's data is NOT migrated.
    """
    current_region = get_user_region(user_id)

    # Do NOT migrate data -- it stays in the home region
    # Only update the user's "current access region" for latency optimization
    update_access_region(user_id, new_region)

    # All reads and writes still go to the home region's primary
    # This ensures GDPR compliance (data at rest stays in EU)
    return {
        'data_region': current_region,  # never changes
        'access_region': new_region,    # changes with travel
        'routing': REGION_MAP[current_region]  # always route to home
    }
```

## Part C: Failover Scenarios

### Scenario 1: US-East Primary Fails

**Impact:** US users cannot write. Global product catalog is unavailable for writes.

**Procedure:**
1. Patroni detects primary failure (30-second TTL)
2. US-East replica is promoted to primary (automatic, ~60 seconds)
3. EU and APAC continue operating independently (regional primaries unaffected)
4. Global product catalog writes are unavailable until US-East recovers or failover completes
5. Update DNS to point to the new US-East primary

**RTO:** 60-90 seconds (automatic Patroni failover)
**RPO:** Zero (synchronous replication to the US-East replica)

### Scenario 2: Network Partition Between US and EU

**Impact:** US and EU regions operate independently. Cross-region replication stops.

**Procedure:**
1. Each region continues serving its own users (no user-facing impact)
2. Global product catalog updates queue in each region
3. When the partition heals, apply conflict resolution for global data
4. Regional user data is unaffected (partitioned by region)

**Conflict resolution for global data:**
- Product catalog changes: last-writer-wins (product updates are idempotent)
- System config changes: manual merge required

**RTO:** Zero (no failover needed -- each region is independent)
**RPO:** Zero for regional data. For global data, RPO equals partition duration.

### Scenario 3: EU Primary Fails (4-Hour Recovery)

**Impact:** EU users cannot write. EU user data is unavailable.

**Procedure:**
1. Patroni promotes EU replica (if available in same zone) -- 60 seconds
2. If no EU replica is available, activate the DR replica in EU-West-2
3. If no EU replicas exist, fail over to US-East (temporary, GDPR concern)
4. Begin recovery of the EU primary in EU-West-1

**GDPR consideration:** If failing over to US-East, EU user data temporarily resides outside the EU. This is acceptable under GDPR Article 49 (necessary for performance of contract) but must be reverted within a reasonable time.

**RTO:** 60 seconds (with replica) or 5-10 minutes (without replica, requires backup restore)
**RPO:** Zero (with sync replica) or up to 1 minute (with async replica)

### Scenario 4: APAC Replica Falls 5 Minutes Behind

**Impact:** APAC users see stale data (up to 5 minutes old) when reading from the replica.

**Procedure:**
1. Check the cause: network congestion, WAL generation spike, or replica I/O saturation
2. If network: route APAC reads to the APAC primary temporarily (increases primary load)
3. If I/O: reduce replica query load by routing heavy analytics queries to a different replica
4. If WAL spike: wait for the spike to pass; lag will recover naturally
5. Alert if lag exceeds 10 minutes

**RTO:** N/A (no failover needed)
**RPO:** N/A (reads are stale, but writes are not affected)

## Part D: Conflict Resolution Strategy

### Data Partitioning Eliminates Most Conflicts

Because user data is partitioned by region, most conflicts are impossible:
- US users only write to the US primary
- EU users only write to the EU primary
- APAC users only write to the APAC primary

Conflicts only occur for **global shared data** (product catalog, system config).

### Global Data Conflict Resolution

| Data Type | Conflict Type | Resolution | Rationale |
|-----------|--------------|------------|-----------|
| Product catalog | Update-update | Last-writer-wins | Product updates are idempotent; latest price/description is correct |
| Product catalog | Update-delete | Update wins | Products should not be deleted while referenced by orders |
| System config | Update-update | First-writer-wins + manual review | Config changes need validation |
| System config | Concurrent creation | Unique constraint + retry | Config keys are unique; second creator gets error |

### Split-Brain Handling

```python
class GlobalDataResolver:
    """Resolves conflicts for global shared data."""

    def __init__(self, primary_region: str):
        self.primary_region = primary_region  # US-East is source of truth

    def resolve_write_conflict(self, local_data: dict,
                                remote_data: dict) -> dict:
        """Resolve a conflict between two writes to global data."""
        local_ts = local_data['updated_at']
        remote_ts = remote_data['updated_at']

        # For product catalog: last-writer-wins
        if local_data['table'] == 'products':
            if local_ts >= remote_ts:
                return local_data
            return remote_data

        # For system config: primary region wins (source of truth)
        if local_data['table'] == 'system_config':
            if local_data['region'] == self.primary_region:
                return local_data
            return remote_data

    def handle_split_brain(self, region_a_data: dict,
                           region_b_data: dict) -> dict:
        """Handle split-brain: two regions accepted conflicting writes."""
        # The primary region is always the source of truth
        if region_a_data['region'] == self.primary_region:
            winner = region_a_data
            loser = region_b_data
        else:
            winner = region_b_data
            loser = region_a_data

        # Log the conflict for post-incident review
        log_conflict(winner, loser)

        # Notify the losing region that their write was overridden
        notify_region(loser['region'], loser['id'], 'overridden')

        return winner
```

### Version Vectors for Global Data

```python
# Global data uses version vectors to detect conflicts
# Regional data does not need them (no conflicts possible)

class GlobalRecord:
    def __init__(self, data: dict):
        self.data = data
        self.vector = {}  # {region: counter}

    def write(self, region: str, new_data: dict):
        self.vector[region] = self.vector.get(region, 0) + 1
        self.data.update(new_data)
        self.data['updated_by'] = region
        self.data['updated_at'] = time.time()

    def has_conflict(self, other: 'GlobalRecord') -> bool:
        """Check if two versions are concurrent."""
        all_regions = set(self.vector.keys()) | set(other.vector.keys())
        self_dominates = True
        other_dominates = True

        for region in all_regions:
            s = self.vector.get(region, 0)
            o = other.vector.get(region, 0)
            if s < o:
                self_dominates = False
            if o < s:
                other_dominates = False

        return not self_dominates and not other_dominates
```

## Common Mistakes to Avoid

1. **Replicating user data across regions without partitioning** -- this creates GDPR compliance issues and unnecessary conflict resolution
2. **Treating all data types the same** -- user data and global data need different replication strategies
3. **Ignoring the "data at rest" vs. "data in transit" distinction in GDPR** -- user data can transit through other regions but must be stored in the home region
4. **Not having a designated source of truth for global data** -- without one, split-brain resolution is arbitrary

## Key Takeaway

Cross-region replication must be designed around data residency requirements first. Partitioning user data by region eliminates most conflicts and ensures GDPR compliance. Global shared data requires a separate replication strategy with conflict resolution. The key architectural decision is what is regional (user data) vs. what is global (product catalog), because this determines whether you need conflict resolution at all.
