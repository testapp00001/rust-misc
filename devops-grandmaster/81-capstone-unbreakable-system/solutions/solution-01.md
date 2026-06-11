# Solution 01: Architect the Unbreakable System

## Part A: Target Architecture

```
                        Users Worldwide
                             |
                             v
                 +-----------------------+
                 |   Global DNS (Route53)|
                 |   Latency-based       |
                 |   routing + failover  |
                 +-----------------------+
                    /        |        \
                   v         v         v
          +----------+ +----------+ +----------+
          | us-east-1| | eu-west-1| |ap-south-1|
          | CloudFront| |CloudFront| |CloudFront|
          | WAF + DDoS| | WAF+DDoS| | WAF+DDoS|
          +----------+ +----------+ +----------+
               |            |            |
               v            v            v
          +----------+ +----------+ +----------+
          | K8s      | | K8s      | | K8s      |
          | Cluster  | | Cluster  | | Cluster  |
          | (3 AZ)   | | (3 AZ)   | | (3 AZ)   |
          |          | |          | |          |
          | App Pods | | App Pods | | App Pods |
          | (HPA)    | | (HPA)    | | (HPA)    |
          +----------+ +----------+ +----------+
               |            |            |
               v            v            v
          +----------+ +----------+ +----------+
          |PostgreSQL| |PostgreSQL| |PostgreSQL|
          | Primary  | | Read     | | Read     |
          | + Sync   | | Replica  | | Replica  |
          | Replica  | | (async)  | | (async)  |
          +----------+ +----------+ +----------+
               |            |            |
               +------+-----+------+-----+
                      |            |
                      v            v
               +----------+ +----------+
               |Cross-Reg | | Cross-Reg|
               | Async    | | Cache    |
               | Replicas | | (Redis)  |
               +----------+ +----------+
```

Key design decisions:

1. **Edge layer:** Route53 with latency-based routing sends users to the
   nearest region. CloudFront provides CDN caching and AWS Shield + WAF
   provides DDoS mitigation at the edge.

2. **Application layer:** Each region has its own Kubernetes cluster spread
   across 3 Availability Zones. HPA scales pods automatically. Pod
   Disruption Budgets ensure zero-downtime during node drains.

3. **Database layer:** Each region has a PostgreSQL primary with a
   synchronous replica in a different AZ (zero data loss on AZ failure).
   Cross-region replicas are async (eventual consistency, but the primary
   region handles all writes for strong consistency on payments).

4. **Deployment:** Blue-green deployments via Argo Rollouts. New version
   receives traffic only after health checks pass. Instant rollback on
   error rate spike.

5. **DDoS mitigation:** Three layers -- CloudFront rate limiting (Layer 7),
   AWS Shield Advanced (Layer 3/4), and application-level rate limiting
   with token buckets per user/IP.

## Part B: Single Points of Failure in the Current Architecture

| Component | Failure Impact | Downtime | Data Loss |
|-----------|---------------|----------|-----------|
| Single ALB | All traffic stops | Until ALB is replaced (~5 min) | None (in-flight requests lost) |
| Single K8s cluster | All pods unreachable | Until cluster recovers (~10-30 min) | None if pods were stateless |
| Single PostgreSQL instance | No reads or writes possible | Until DB is restored from backup (~30-120 min) | All data since last backup |
| Single region (us-east-1) | Total system outage | Until region recovers or manual failover (~30-60 min) | All in-flight data |
| Single deployment pipeline | Cannot deploy fixes or rollbacks | Manual intervention required | None |
| Single K8s control plane | Cannot schedule new pods, cannot scale | Until control plane recovers (~5-15 min) | None |

The most dangerous SPOF is the single PostgreSQL instance. If the disk
corrupts, recovery depends on the last backup. With hourly backups, you
lose up to one hour of payment data -- potentially millions of dollars
in transactions.

## Part C: Failure Domain Hierarchy

```
Level              Detection     Failover       Mechanism
---------------------------------------------------------------------------
Process            < 1 second    < 1 second     K8s liveness probe restarts
                                                the container
Machine            < 30 seconds  < 2 minutes    K8s reschedules pods to other
                                                nodes. Node auto-replacement
                                                via cloud provider.
Rack               < 1 minute    < 2 minutes    Multi-AZ deployment. K8s
                                                topology spread constraints
                                                ensure pods are on different
                                                racks/AZs.
Availability Zone  < 1 minute    < 5 minutes    Multi-AZ K8s cluster. DB
                                                synchronous replica promotes
                                                to primary. Traffic routes
                                                to surviving AZs.
Region             < 1 minute    < 5 minutes    Global DNS failover (Route53
                                                health checks). Standby
                                                region promotes its DB
                                                replica to primary.
Cloud Provider     Manual        30-60 minutes  Multi-cloud hot standby.
                                                Requires manual DNS switch
                                                or external global LB.
```

Detection times assume health check intervals. Failover times assume
automated tooling (not manual intervention).

## Part D: CAP Theorem Trade-off

**1. How does your architecture handle the CAP theorem trade-off?**

The system uses a hybrid approach:

- **Within a region:** CP (Consistency + Partition tolerance). PostgreSQL
  synchronous replication ensures every write is confirmed by at least
  two nodes before returning success. If the sync replica is unreachable,
  writes block until it recovers (availability sacrificed for consistency).

- **Across regions:** AP with eventual consistency for reads, but all
  writes are routed to the primary region. This avoids multi-region
  write conflicts entirely. The trade-off is that write latency for
  users far from the primary region is higher (~100-200ms RTT).

**2. What happens to availability during a network partition between regions?**

During a partition, the primary region continues processing writes
normally. Secondary regions serve stale reads from their local replicas.
If the primary region becomes unreachable, Route53 health checks detect
this and fail over to the secondary region, which promotes its replica
to primary. There is a brief window (seconds) where writes are
unavailable during the promotion.

**3. What is your consistency model for payment data vs. analytics data?**

- **Payment data:** Strong consistency. All writes go to the primary
  region's PostgreSQL primary. Reads that must see the latest write
  (e.g., balance checks) also go to the primary. This guarantees
  no double-spending.

- **Analytics data:** Eventual consistency. Analytics are written to a
  separate event stream (Kafka) and consumed by analytics services in
  each region independently. A few seconds of delay is acceptable for
  dashboards and reports.

**4. How do you handle split-brain in the database layer?**

Split-brain is prevented by using a quorum-based leader election. The
database uses a distributed consensus protocol (Patroni with etcd) that
requires a majority of nodes to agree on the primary. If a network
partition splits the cluster, only the partition with a majority (2 of 3
nodes) can elect a primary. The minority partition becomes read-only.

For cross-region failover, an external arbiter (Route53 health checks)
determines which region is the primary. Only one region can be primary
at a time. The promotion script checks that the old primary is truly
down (via health checks from multiple locations) before promoting the
secondary.

## Part E: Cost of "Unbreakable"

| Setup | Monthly Cost (Estimate) | Availability | Downtime/Year |
|-------|------------------------|--------------|---------------|
| Single instance | ~$500 | 99.9% (~8.7 hrs) | 8.7 hours |
| Multi-AZ (1 region) | ~$3,000 | 99.99% (~52 min) | 52 minutes |
| Multi-region active-active | ~$15,000 | 99.999% (~5 min) | 5 minutes |
| Multi-cloud active-active | ~$50,000+ | 99.9999% (~31 sec) | 31 seconds |

**Formula:**

```
Break-even point:
  Cost_of_downtime = Revenue_per_hour * Downtime_hours_per_year

  If Revenue = $1,000,000/hour:
    Single instance:  $1M * 8.7   = $8,700,000/year downtime cost
    Multi-AZ:         $1M * 0.87  = $870,000/year downtime cost
    Multi-region:     $1M * 0.087 = $87,000/year downtime cost

  Infrastructure cost difference:
    Single to Multi-AZ:       +$2,500/month  = $30,000/year
    Multi-AZ to Multi-region: +$12,000/month = $144,000/year

  For a $1M/hour revenue company:
    Multi-AZ saves:    $8.7M - $0.87M = $8.2M  >> $30K cost  --> WORTH IT
    Multi-region saves: $0.87M - $87K  = $783K  >> $144K cost --> WORTH IT
    Multi-cloud saves:  $87K - $8.7K   = $78K   << $420K cost --> NOT WORTH IT
```

For most companies, multi-region is the sweet spot. Multi-cloud only
makes sense when the cost of downtime exceeds millions per hour (stock
exchanges, payment processors at scale).

## Common Mistakes

1. **Designing for five nines when the business needs three.** Most
   startups waste months building multi-region infrastructure when a
   single multi-AZ setup would suffice. Match your availability target
   to your revenue impact, not to engineering ambition.

2. **Forgetting the deployment pipeline as a SPOF.** If your CI/CD system
   goes down, you cannot deploy fixes during an incident. The deployment
   pipeline should be as redundant as the production system.

3. **Ignoring the human element.** A system that requires a human to
   make a failover decision will have 10+ minute downtime no matter how
   fast the infrastructure can recover. Automation is not optional for
   four nines and above.

4. **Treating all data the same.** Payment data needs strong consistency.
   Analytics data can be eventually consistent. Session data can be
   lost and re-created. Using the same consistency model for everything
   either wastes money (everything synchronous) or risks correctness
   (everything eventually consistent).

5. **Overcomplicating the architecture.** Every additional component
   (service mesh, sidecar, proxy) adds a failure mode. The most
   reliable system is the simplest one that meets the requirements.
   Do not add multi-cloud if multi-region is sufficient.
