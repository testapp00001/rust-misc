# Solution 01: Cloud Cost Analysis

## Problem Statement

Perform a complete cloud cost analysis for a mid-size infrastructure. Identify
waste, break down costs by team, compare pricing models, and produce
prioritized optimization recommendations with estimated savings.

## Complete Solution

### Step 1: Gather Raw Cost Data

Pull billing data from your cloud provider. Below is a representative monthly
snapshot for a company running 120 EC2 instances, 15 RDS databases, and
supporting services.

```
Infrastructure Inventory (Monthly)
===================================

Compute (EC2)
  40x  m5.2xlarge   (8 vCPU, 32 GB)   -- Production web tier
  20x  m5.xlarge    (4 vCPU, 16 GB)    -- Staging environments
  15x  c5.4xlarge   (16 vCPU, 32 GB)   -- Batch processing
  10x  r5.2xlarge   (8 vCPU, 64 GB)    -- In-memory caching
  20x  t3.medium    (2 vCPU, 4 GB)     -- Dev/test instances
  15x  m5.large     (2 vCPU, 8 GB)     -- Monitoring/logging

Databases (RDS)
  5x   db.r5.2xlarge  -- Production PostgreSQL
  3x   db.r5.xlarge   -- Production MySQL
  4x   db.t3.medium   -- Dev/test databases
  3x   db.r5.large    -- Staging databases

Storage
  50 TB  S3 Standard
  20 TB  S3 Infrequent Access
  30 TB  EBS gp3
  10 TB  EBS io2

Networking
  2x    NAT Gateways
  3 TB   Data transfer out
  1x    CloudFront distribution
```

### Step 2: Cost Breakdown by Service

```
Monthly Cost by Service
========================

Service                   Monthly Cost    % of Total
----------------------------------------------------+
EC2 Compute               $48,200          42.1%
RDS Databases             $22,800          19.9%
EBS Storage               $ 3,600           3.1%
S3 Storage                $ 1,450           1.3%
Data Transfer             $ 9,200           8.0%
NAT Gateways              $ 1,800           1.6%
Load Balancers            $   900           0.8%
CloudFront                $ 4,500           3.9%
ElastiCache               $ 6,200           5.4%
Other Services            $ 5,850           5.1%
Support                   $ 9,000           7.9%
                                                  |
                                  TOTAL  $113,500  |
-------------------------------------------+
```

### Step 3: Cost Allocation by Team

Tag-based cost allocation using AWS Cost Explorer or equivalent:

```
Monthly Cost by Team
=====================

Team              Instances  RDS    Storage  Network  Other   Total      %
--------------------------------------------------------------------------------+
Platform           15        3      12 TB    1.2 TB   $3,200  $28,400   25.0%
Backend            35        5      18 TB    0.8 TB   $4,100  $32,100   28.3%
Data Engineering   20        2      15 TB    0.5 TB   $2,800  $18,900   16.6%
Frontend           18        2      8 TB     0.3 TB   $1,500  $14,200   12.5%
ML/AI              12        1      10 TB    0.1 TB   $2,200  $12,600   11.1%
QA                 20        2      5 TB     0.1 TB   $  800  $ 7,300    6.4%
                                                                                
Untagged           --        --     --       --       --      $   --     0.0%
--------------------------------------------------------------------------------+
                                  TOTAL                          $113,500
```

### Step 4: Identify Waste

```
Waste Identification Report
=============================

Category                Detail                           Monthly Waste
--------------------------------------------------------------------+
Idle Instances          8 EC2 instances running 24/7      $ 3,200
                        with <5% CPU utilization
                        
Oversized Instances     22 instances using <30% of        $ 8,400
                        allocated CPU and memory
                        
Unattached EBS Volumes  45 volumes (2.3 TB) detached      $   230
                        after instance termination
                        
Old Snapshots           180 EBS snapshots older than      $   180
                        90 days, no retention policy
                        
Unused Elastic IPs      12 unattached Elastic IPs         $   108
                        (charged when not associated)
                        
Stale Load Balancers    4 ALBs with zero traffic in       $   320
                        the last 30 days
                        
Oversized RDS           3 databases using <20% of         $ 2,100
                        provisioned IOPS and storage
                        
Dev/Test Always-On      20 dev instances running          $ 4,800
                        nights and weekends
                        
Over-provisioned S3     12 TB in S3 Standard that        $   540
                        has not been accessed in 90+ days
--------------------------------------------------------------------+
                    TOTAL IDENTIFIED WASTE:               $19,878/yr
                                                          = $238,536/yr
```

### Step 5: On-Demand vs Reserved vs Spot Comparison

```
Pricing Model Comparison (m5.2xlarge example)
===============================================

Model         Hourly    Monthly     Annual      Savings vs
              Rate      Cost        Cost        On-Demand
-----------------------------------------------------------+
On-Demand     $0.384    $278.40     $3,340.80   --
1yr RI (No)   $0.242    $176.66     $2,119.92   36.6%
1yr RI (All)  $0.209    $152.57     $1,830.79   45.2%
3yr RI (No)   $0.167    $121.91     $1,462.92   56.2%
3yr RI (All)  $0.145    $105.85     $1,270.20   62.0%
Savings Plan  $0.230    $167.90     $2,014.80   39.7%
Spot          $0.115    $ 83.95     $1,007.40   69.8%
-----------------------------------------------------------+

* RI (No) = No Upfront    * RI (All) = All Upfront
* Spot price varies; shown is typical 30-day average
```

**Tier-level comparison across workload categories:**

```
Workload Category    Current Model   Recommended Model   Annual Savings
-----------------------------------------------------------------------+
Production Web       On-Demand       1yr RI (All Up)     $ 48,200
Production DB        On-Demand       3yr RI (All Up)     $ 36,500
Batch Processing     On-Demand       Spot (with fallback)$ 22,800
Staging              On-Demand       Savings Plans       $ 12,400
Dev/Test             On-Demand       Spot + Scheduling   $ 18,600
-----------------------------------------------------------------------+
                              TOTAL POTENTIAL SAVINGS:   $138,500/yr
```

### Step 6: Prioritized Optimization Recommendations

```
Optimization Roadmap (Sorted by Impact)
=========================================

Priority  Action                              Effort   Annual Savings  Risk
-----------------------------------------------------------------------------+
P1        Convert prod compute to 1yr RI       Low     $ 48,200        Low
P2        Convert prod DB to 3yr RI            Low     $ 36,500        Low
P3        Move batch processing to Spot        Medium  $ 22,800        Medium
P4        Schedule dev/test off-hours          Low     $ 18,600        Low
P5        Right-size 22 oversized instances    Medium  $ 15,200        Low
P6        Adopt Compute Savings Plans          Low     $ 12,400        Low
P7        Terminate 8 idle instances           Low     $  3,200        Low
P8        Migrate cold S3 to IA tier           Low     $    540        Low
P9        Delete unattached EBS + old snaps    Low     $    410        Low
P10       Remove stale ALBs + unused EIPs      Low     $    428        Low
-----------------------------------------------------------------------------+
                              TOTAL POTENTIAL:          $158,278/yr
                              Current Spend:            $113,500/month
                              Annual Spend:           $1,362,000/yr
                              Savings:                    11.6% of spend
```

### Step 7: Implementation Timeline

```
Quarter 1 (Quick Wins)
  Week 1-2:  Delete idle resources (P7, P9, P10)           $  4,038/yr
  Week 3-4:  Purchase Reserved Instances (P1, P2)           $ 84,700/yr
  Week 5-8:  Implement off-hours scheduling (P4)            $ 18,600/yr
             Subtotal Q1:                                  $107,338/yr

Quarter 2 (Medium Effort)
  Week 1-4:  Right-size instances (P5)                      $ 15,200/yr
  Week 5-8:  Migrate batch to Spot (P3)                     $ 22,800/yr
  Week 9-12: Adopt Savings Plans (P6) + S3 lifecycle (P8)   $ 12,940/yr
             Subtotal Q2:                                  $ 50,940/yr

                             TOTAL FIRST-YEAR SAVINGS:     $158,278/yr
```

## Why This Works

1. **Data-driven approach.** Every recommendation ties back to measured
   utilization data, not guesses. This makes it easy to justify to finance
   and leadership.

2. **Prioritization by ROI.** Sorting by savings-to-effort ratio ensures the
   team captures the biggest wins first. Reserved Instance purchases alone
   often recover 30-40% of potential savings with minimal engineering effort.

3. **Tag-based allocation.** When teams see their own cost numbers, they
   naturally optimize. Making cost visible is the single most effective FinOps
   lever.

4. **Layered pricing models.** No single pricing model is optimal for all
   workloads. Combining Reserved Instances (steady-state), Savings Plans
   (flexible commitment), and Spot (fault-tolerant) covers all patterns.

## Common Mistakes to Avoid

- **Buying Reserved Instances before right-sizing.** If you RI an m5.2xlarge
  that should be an m5.xlarge, you lock in waste for 1-3 years.

- **Ignoring data transfer costs.** Cross-AZ and cross-region traffic often
  accounts for 10-20% of the bill and is easy to overlook.

- **Not setting up alerts.** Cost anomalies (a misconfigured service burning
  $10k overnight) are common. Set up daily spend alerts at 120% of forecast.

- **Treating Spot as free.** Spot interruptions have real operational cost
  (restart time, data loss risk). Only use Spot for workloads that handle
  interruption gracefully.

- **Forgetting about the tagging gap.** Untagged resources make cost
  attribution impossible. Enforce tagging at provisioning time with policy
  (see Solution 05).
