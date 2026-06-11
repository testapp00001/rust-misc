# Solution 01: HA Pattern Selection

## Part A: Allowed Downtime Calculations

The formula for downtime is: `Total Time x (1 - Availability)`

### Calculation Reference

| Duration | Total Minutes |
|----------|---------------|
| 1 Year (365.25 days) | 525,960 minutes |
| 1 Month (30 days) | 43,200 minutes |
| 1 Week (7 days) | 10,080 minutes |

### Completed Table

| System | Availability | Downtime/Year | Downtime/Month | Downtime/Week |
|--------|--------------|---------------|----------------|---------------|
| A | 99.9% | 525.96 min (~8.77 hrs) | 43.20 min | 10.08 min |
| B | 99.99% | 52.60 min | 4.32 min | 1.01 min |
| C | 99.999% | 5.26 min | 0.43 min (~26 sec) | 0.10 min (~6 sec) |
| D | 99.0% | 5,259.6 min (~87.66 hrs) | 432.00 min (~7.2 hrs) | 100.80 min (~1.68 hrs) |
| E | 99.99% | 52.60 min | 4.32 min | 1.01 min |

### What the Numbers Tell You

- **System C (99.999%)** has only 26 seconds of downtime per month. A single failover that takes 30 seconds would blow the entire monthly budget. This system cannot use any pattern that involves a failover event visible to clients.
- **System B and E (99.99%)** have about 4 minutes per month. A 10-second failover is acceptable, but you only get ~24 such events per month before exceeding the budget.
- **System A (99.9%)** gives you nearly 9 hours per year -- enough for planned maintenance windows plus a few unplanned incidents.
- **System D (99.0%)** gives you over 7 hours per month, making it suitable for batch workloads where recovery from backup is acceptable.

## Part B: Completed Pattern Mapping

| System | Recommended Pattern | Justification |
|--------|---------------------|---------------|
| A | Active-Passive (Cold Standby) | Internal portal with business-hours usage can tolerate minutes of failover time; 99.9% allows ~43 min/month of downtime, making cold standby cost-effective. |
| B | Active-Active (Single-Region) | E-commerce checkout requires sub-second failover and handles burst traffic; multiple active nodes provide both HA and load distribution for peak shopping events. |
| C | Active-Active (Multi-Region) | Payment processing at 99.999% needs geographic redundancy so that a regional failure does not exhaust the 26-second monthly budget; multi-region also reduces latency for global merchants. |
| D | No HA (Single Instance) | Batch analytics runs nightly and has 7+ hours of allowed downtime per month; a single instance with S3-backed backups and infrastructure-as-code for fast rebuild is the most cost-effective approach. |
| E | Active-Active (Single-Region) | Real-time bidding has extreme latency requirements (sub-10ms) and cannot tolerate any failover delay; multiple active nodes with client-side failover ensure continuous operation. |

### Why Active-Passive Would Not Work for B and E

Active-passive failover involves detecting the failure, promoting the standby, and redirecting traffic. Even with automation, this takes 5-30 seconds. For System E (real-time bidding), a 5-second outage means lost auctions. For System B (checkout), a 30-second outage during Black Friday could mean thousands of abandoned carts. Active-active eliminates the failover delay entirely because traffic is already flowing to multiple nodes.

### Why Multi-Region Is Required for C

A single-region active-active cluster can survive a node failure but not a regional failure (data center fire, network partition, power outage). At 99.999%, you have 5.26 minutes per year. A regional incident that takes 2 hours to resolve would consume 23 years' worth of downtime budget. Multi-region ensures that a regional failure is handled by the surviving region.

## Part C: Single Points of Failure

| System | SPOF 1 | SPOF 2 | Mitigation for Each |
|--------|--------|--------|---------------------|
| A | Single DNS provider (if DNS fails, users cannot resolve the portal URL) | Single backup storage location (if backups are only on local disk, disk failure destroys recovery capability) | Use two DNS providers with low TTL; replicate backups to an off-site location (S3 or another data center) |
| B | Single HAProxy instance (if the load balancer fails, all backend servers are unreachable) | Single database primary (if the DB primary fails, writes stop until failover) | Deploy HAProxy in active-passive with keepalived VIP; use Patroni with etcd for database HA with automatic failover |
| C | Single payment processor integration (if the processor's API is down, transactions cannot complete) | Single TLS certificate authority (if the CA revokes your cert or has an outage, all HTTPS connections fail) | Integrate with 2+ payment processors with automatic fallback; use certificates from two different CAs or implement certificate pinning with manual backup |
| D | Single ETL job scheduler (if the scheduler crashes, no jobs run) | Single data warehouse connection (if the warehouse is down, all queries fail) | Use a distributed scheduler (Airflow with PostgreSQL metadata store); implement retry logic with exponential backoff for warehouse connections |
| E | Single Redis cache layer (if Redis fails, the bidding engine slows down dramatically) | Single time-source server (if NTP drifts, bid timestamps become unreliable, causing auction disputes) | Deploy Redis Sentinel or Redis Cluster for cache HA; use multiple NTP servers with local GPS-based time source as backup |

### How to Find SPOFs Systematically

Walk the request path from the client to the data store and back. For each component, ask: "If this single instance fails, does the request fail?" If yes, it is a SPOF. Common categories:

1. **Network:** Single ISP, single switch, single load balancer
2. **Compute:** Single application server, single database node
3. **Storage:** Single disk, single storage controller, single backup location
4. **Services:** Single DNS provider, single CA, single time source
5. **Human:** Single on-call engineer who knows the system, single deployment key

## Common Mistakes to Avoid

1. **Confusing availability target with actual availability.** A system designed for 99.99% will only achieve it if every component in the chain meets or exceeds that target. One component at 99.9% becomes the ceiling for the entire system.

2. **Ignoring planned maintenance in downtime budgets.** The 52.6 minutes per year for 99.99% includes planned maintenance. If you need 30 minutes for a quarterly upgrade, that is 120 minutes per year -- already over budget. Plan for online maintenance or rolling upgrades.

3. **Over-engineering low-impact systems.** System D (batch analytics) does not need active-active. Spending money on HA for a system that can tolerate 7 hours of downtime per month is waste that could be better spent on the systems that actually need it.

4. **Forgetting about dependent services.** Your application may be active-active, but if it depends on a single-region payment processor, you have not achieved multi-region HA. Map the entire dependency chain.

5. **Treating 99.999% as "just one more nine."** Each additional nine of availability is roughly 10x harder and more expensive than the previous one. 99.999% is not an incremental improvement over 99.99% -- it is a fundamentally different architecture.

## Key Takeaway

Availability is a mathematical contract, not a wish. The number of nines you commit to dictates every architectural decision: how many replicas you need, how fast failover must be, whether you can tolerate a single-region failure, and whether planned maintenance counts against your budget. Start with the math, then choose the pattern -- never the other way around.
