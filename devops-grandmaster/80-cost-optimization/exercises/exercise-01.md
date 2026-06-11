# Exercise 01: Cloud Cost Analysis

**Type:** Conceptual | **Time:** 15 min | **Difficulty:** Easy

## Objective

Analyze a monthly cloud bill breakdown to identify waste, categorize costs by team, and
propose optimization strategies across on-demand, reserved, and spot pricing models.

## Background

You are a DevOps engineer at **Acme Corp**. The CFO has flagged a 40% increase in the
monthly cloud bill over the past quarter. You have been asked to review the bill and
recommend cost-saving measures. The company runs workloads on AWS across three teams:
**Platform**, **Data**, and **ML**.

## Monthly Cloud Bill Summary (April 2026)

| Service | Resource | Pricing Model | Monthly Cost | Team |
|---------|----------|---------------|-------------|------|
| EC2 | 8x m5.2xlarge (us-east-1) | On-Demand | $3,340.80 | Platform |
| EC2 | 4x m5.xlarge (us-east-1) | On-Demand | $835.20 | Platform |
| EC2 | 6x r5.4xlarge (us-east-1) | On-Demand | $4,924.80 | Data |
| EC2 | 12x p3.2xlarge (us-east-1) | On-Demand | $22,032.00 | ML |
| EC2 | 2x t3.medium (us-east-1) | On-Demand | $60.48 | Platform |
| RDS | db.r5.2xlarge Multi-AZ | On-Demand | $1,392.00 | Data |
| RDS | db.t3.medium Single-AZ | On-Demand | $87.60 | Platform |
| S3 | 15 TB Standard | - | $345.00 | Shared |
| S3 | 8 TB Infrequent Access | - | $100.00 | Shared |
| EBS | 5 TB gp3 | - | $400.00 | Platform |
| EBS | 12 TB gp3 (attached) | - | $960.00 | Data |
| EBS | 3 TB gp3 (unattached) | - | $240.00 | Unknown |
| NAT Gateway | 2 gateways, 2 TB processed | - | $135.04 | Platform |
| ELB | 4 Application Load Balancers | - | $72.00 | Platform |
| CloudWatch | Logs + Metrics | - | $180.00 | Shared |
| Elasticache | cache.r5.xlarge (3 nodes) | On-Demand | $1,170.00 | Data |
| EKS | 3 clusters (control plane) | - | $219.60 | Platform |

**Total Monthly Bill: $36,594.52**

## Tasks

### Task 1: Identify Waste

Review the bill and identify resources that are clearly wasteful or underutilized. For
each item, explain *why* it is wasteful and what you would do about it.

Record your findings in this format:

```
Waste Item #:
  Resource:
  Monthly Cost:
  Why It Is Waste:
  Recommended Action:
  Estimated Savings:
```

### Task 2: Categorize Costs by Team

Calculate the total cost attributed to each team. Note that some costs are "Shared" --
propose a fair allocation method for shared costs.

| Team | Direct Cost | Shared Cost (Method) | Total |
|------|-------------|----------------------|-------|
| Platform | ? | ? | ? |
| Data | ? | ? | ? |
| ML | ? | ? | ? |

### Task 3: Pricing Model Optimization

For each EC2 and RDS resource, evaluate whether it should move to Reserved Instances (RI)
or Spot Instances. Use these assumptions:

- **1-Year Reserved (No Upfront):** ~36% savings vs On-Demand
- **3-Year Reserved (All Upfront):** ~60% savings vs On-Demand
- **Spot Instances:** ~65-70% savings vs On-Demand (with interruption risk)
- The ML team runs training jobs that are interruptible (checkpointing is implemented)
- The Platform team runs web servers that must maintain 99.9% availability
- The Data team runs batch ETL jobs nightly (6 hours) and a persistent analytics cluster

Complete this table:

| Resource | Current Model | Recommended Model | Justification | Monthly Savings |
|----------|--------------|-------------------|---------------|-----------------|
| 8x m5.2xlarge (Platform) | On-Demand | ? | ? | ? |
| 4x m5.xlarge (Platform) | On-Demand | ? | ? | ? |
| 6x r5.4xlarge (Data) | On-Demand | ? | ? | ? |
| 12x p3.2xlarge (ML) | On-Demand | ? | ? | ? |
| 2x t3.medium (Platform) | On-Demand | ? | ? | ? |
| RDS r5.2xlarge (Data) | On-Demand | ? | ? | ? |
| RDS t3.medium (Platform) | On-Demand | ? | ? | ? |

### Task 4: Quick Wins

List at least 3 "quick win" optimizations that require minimal effort but yield immediate
savings. These should be things you can implement today.

```
Quick Win #1:
  Action:
  Estimated Monthly Savings:
  Implementation Effort:

Quick Win #2:
  Action:
  Estimated Monthly Savings:
  Implementation Effort:

Quick Win #3:
  Action:
  Estimated Monthly Savings:
  Implementation Effort:
```

<details>
<summary>Hint 1: Identifying Waste</summary>

Look for resources with the word "unattached," unused NAT gateways, idle load balancers,
and instances that could be smaller. Also check if any resources belong to "Unknown" teams
-- those are often orphaned.

</details>

<details>
<summary>Hint 2: Shared Cost Allocation</summary>

Common allocation methods:
- **Even split:** Divide equally (simplest, but unfair if usage differs)
- **Proportional to compute:** Split by each team's percentage of total compute cost
- **Proportional to headcount:** Split by team size
- **Tag-based:** Use resource tagging to assign exact costs

</details>

<details>
<summary>Hint 3: Reserved vs Spot Decision</summary>

Use this decision framework:
- **Must be always-on, no interruption tolerance** -> Reserved Instance (1yr or 3yr)
- **Interruptible, stateless, or checkpointed** -> Spot Instance
- **Variable baseline, predictable peak** -> Reserved for baseline + On-Demand/Spot for peak
- **Short-lived or experimental** -> On-Demand (keep flexibility)

</details>

## Verification

After completing this exercise, you should have:
- Identified at least 3 clear sources of waste
- A cost breakdown by team with shared cost allocation
- A pricing model recommendation for every EC2 and RDS resource
- At least 3 quick wins with estimated savings
- A total estimated savings figure (target: 30-50% of current bill)

## Reflection Questions

1. Why is it important to tag all resources with team ownership?
2. What risks come with moving everything to Reserved Instances?
3. How would you build a culture of cost awareness across engineering teams?
