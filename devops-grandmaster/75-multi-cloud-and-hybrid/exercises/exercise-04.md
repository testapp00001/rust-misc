# Exercise 04: Design a Multi-Cloud Disaster Recovery Strategy

**Type:** Challenge
**Difficulty:** Advanced
**Estimated Time:** 4-6 hours

## Objective

Design and document a comprehensive disaster recovery (DR) strategy for a
multi-cloud application. Your strategy must handle regional outages within a
single cloud AND complete cloud provider failures. You will create an
architecture document, Terraform infrastructure for the DR environment, and
runbook procedures for failover and failback.

## Background

A multi-cloud DR strategy goes beyond having backups. It requires:

- **Data replication** across clouds with defined RPO (Recovery Point Objective)
- **Infrastructure readiness** with defined RTO (Recovery Time Objective)
- **Automated failover** that can activate without manual intervention
- **Failback procedures** to return to the primary cloud after recovery
- **Regular testing** to validate the strategy actually works

This exercise simulates a scenario where a SaaS application runs primarily on
AWS with a warm standby on GCP.

## Scenario

You are the platform architect for **DataVault**, a SaaS analytics platform
with these characteristics:

- **Primary region**: AWS us-east-1
- **DR region**: GCP us-central1
- **Application**: A three-tier web application
  - Web tier: Containerized Node.js frontend
  - API tier: Containerized Go backend services
  - Data tier: PostgreSQL database with 500GB of data
- **RPO**: 15 minutes (maximum data loss acceptable)
- **RTO**: 30 minutes (maximum downtime acceptable)
- **Traffic**: 10,000 requests/second at peak
- **Compliance**: Must maintain audit logs during failover

## Instructions

### Part 1: Architecture Document (90 minutes)

Create `dr-architecture.md` that covers:

1. **Component inventory**: List every component in the primary (AWS) stack
   and its DR equivalent on GCP. Include compute, database, storage, DNS,
   CDN, secrets management, and monitoring.

2. **Data replication strategy**: For each data store, specify:
   - Replication method (streaming, snapshot, log shipping)
   - Replication lag target
   - Conflict resolution strategy
   - Cost estimate for replication

3. **Failover decision tree**: Create a decision tree that answers:
   - How do you detect a failure? (health checks, synthetic monitoring)
   - How do you determine it is a regional vs cloud-wide failure?
   - Who authorizes failover? (automated vs manual)
   - What are the go/no-go criteria?

4. **Network architecture**: Show how DNS failover works:
   - Route 53 health checks pointing to AWS
   - Failover to GCP load balancer
   - Certificate management across both clouds

5. **Security considerations**: How do you manage:
   - Secrets synchronization across clouds
   - IAM equivalent mapping (AWS IAM to GCP IAM)
   - Network security during failover

### Part 2: Infrastructure as Code (90 minutes)

Create Terraform configurations for the DR environment. You do NOT need to
implement the full primary stack -- only the DR components on GCP that would
be activated during failover.

Create these files:

```
exercise-04/
  dr-architecture.md
  terraform/
    main.tf          -- Provider config for GCP
    variables.tf     -- All input variables
    compute.tf       -- GKE cluster or Compute Engine for DR
    database.tf      -- Cloud SQL PostgreSQL with cross-cloud replication
    storage.tf       -- GCS buckets synced from S3
    networking.tf    -- VPC, load balancer, firewall rules
    dns.tf           -- Cloud DNS or Route 53 failover records
    monitoring.tf    -- Uptime checks and alerting for DR readiness
    outputs.tf       -- Connection endpoints and failover URLs
```

Key requirements for the Terraform code:

1. **Database**: Configure Cloud SQL PostgreSQL with point-in-time recovery
   enabled. Document how you would set up logical replication from RDS to
   Cloud SQL.

2. **Compute**: Create a GKE cluster (or managed instance group) with
   auto-scaling that can scale from 0 (warm standby) to full capacity on
   demand.

3. **Storage**: Create GCS buckets and document the cross-cloud sync
   strategy (e.g., using Cloud Storage Transfer Service or a custom
   replication job).

4. **DNS**: Configure health checks and failover routing so that DNS
   automatically points to GCP when AWS health checks fail.

### Part 3: Runbook (60 minutes)

Create `dr-runbook.md` with step-by-step procedures for:

1. **Failover procedure**: Exact commands and steps to:
   - Confirm the failure is not a false alarm
   - Scale up the DR environment from warm standby to full capacity
   - Promote the GCP database replica to primary
   - Switch DNS to point to GCP
   - Verify application health on GCP
   - Notify stakeholders

2. **Failback procedure**: Exact commands and steps to:
   - Rebuild the AWS environment
   - Set up reverse replication from GCP to AWS
   - Validate data consistency
   - Switch DNS back to AWS
   - Scale down GCP to warm standby

3. **Testing schedule**: Define a DR testing cadence:
   - Monthly: Automated health check validation
   - Quarterly: Partial failover test (database only)
   - Annually: Full failover test with production traffic

### Part 4: RPO/RTO Validation (30 minutes)

Create a table or script that calculates:

| Component | Replication Method | Actual RPO | Target RPO | Actual RTO | Target RTO |
|-----------|-------------------|------------|------------|------------|------------|
| PostgreSQL | ? | ? | 15 min | ? | 30 min |
| Object Storage | ? | ? | 1 hr | ? | 30 min |
| Application State | ? | ? | N/A | ? | 30 min |
| DNS | ? | ? | 5 min | ? | 5 min |

Identify any gaps where actual RPO/RTO exceeds the target and propose
solutions.

## Success Criteria

- [ ] Architecture document covers all components with clear primary-to-DR
      mapping.
- [ ] Data replication strategy addresses each data store with a specific
      technology choice and rationale.
- [ ] Failover decision tree is actionable and distinguishes automated vs
      manual steps.
- [ ] Terraform configurations are syntactically valid and create a viable
      DR environment.
- [ ] Runbook contains exact commands (not vague descriptions) for both
      failover and failback.
- [ ] RPO/RTO table identifies gaps and proposes mitigations.
- [ ] You can articulate the cost of maintaining this DR strategy and
      justify it against the business impact of downtime.

## Hints

<details>
<summary>Hint 1: Cross-cloud database replication</summary>

Direct streaming replication between AWS RDS and GCP Cloud SQL is not
natively supported. Common approaches:

1. **Logical replication via PostgreSQL native**: Configure RDS as a
   publisher and Cloud SQL as a subscriber using PostgreSQL logical
   replication. Both support this with some configuration.

2. **Application-level dual-write**: The application writes to both databases
   asynchronously. Simpler but higher risk of inconsistency.

3. **Third-party tools**: Use tools like Debezium (CDC) with Kafka to stream
   changes between databases.

For this exercise, option 1 is the recommended approach. Document the
limitations (schema changes require manual coordination, DDL is not
replicated).

</details>

<details>
<summary>Hint 2: Warm standby cost optimization</summary>

A warm standby that runs at full capacity 24/7 is expensive. Optimize by:

1. Running the GKE node pool at minimum size (1-2 nodes) during normal
   operations.
2. Using Horizontal Pod Autoscaler and Cluster Autoscaler to scale up on
   demand during failover.
3. Keeping the Cloud SQL replica at a smaller instance class and using
   `gcloud sql instances patch` to resize during failover (takes 5-10
   minutes).
4. Pre-building container images and pushing to both ECR and GCR so there
   is no build step during failover.

</details>

<details>
<summary>Hint 3: DNS failover with health checks</summary>

Use AWS Route 53 with failover routing policy:

```
Primary record: app.datavault.com -> AWS ALB (primary)
Secondary record: app.datavault.com -> GCP LB (DR)
Health check: HTTPS GET to /health every 10 seconds, failure threshold 3
```

When the primary health check fails for 30+ seconds, Route 53 automatically
routes to the secondary. Document the DNS TTL impact on failover time --
lower TTL means faster failover but higher DNS query costs.

</details>
