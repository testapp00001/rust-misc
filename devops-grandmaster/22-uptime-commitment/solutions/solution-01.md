# Solution 01: Calculate Downtime for Availability Targets

## Part A -- Downtime Table

| Availability | Per Day        | Per Week       | Per Month      | Per Year       |
|--------------|----------------|----------------|----------------|----------------|
| 99%          | 864s (14m 24s) | 1h 41m         | 7h 12m         | 3d 15h 36m     |
| 99.5%        | 432s (7m 12s)  | 50m            | 3h 36m         | 1d 19h 48m     |
| 99.9%        | 86.4s (1m 26s) | 10m 5s         | 43m 12s        | 8h 45m 36s     |
| 99.95%       | 43.2s          | 5m 2s          | 21m 36s        | 4h 22m 48s     |
| 99.99%       | 8.64s          | 1m 0s          | 4m 19s         | 52m 34s        |
| 99.999%      | 0.864s         | 6s             | 26s            | 5m 15s         |

**Worked example for 99.9% monthly:**

```
30 days * 24 hours * 60 minutes * 60 seconds = 2,592,000 seconds
2,592,000 * (1 - 0.999) = 2,592,000 * 0.001 = 2,592 seconds
2,592 / 60 = 43.2 minutes
```

## Part B -- Scenario Analysis

### Question 1

Monthly error budget for 99.9%:
```
2,592,000 seconds * 0.001 = 2,592 seconds = 43 minutes 12 seconds
```

After a 15-minute outage:
```
43m 12s - 15m = 28m 12s remaining
```

### Question 2

After the 15-minute outage and the 10-minute equivalent:
```
Budget used: 15m + 10m = 25m
Budget remaining: 43m 12s - 25m = 18m 12s
Percentage remaining: 18m 12s / 43m 12s = 1,092 / 2,592 = 42.1%
```

### Question 3

Annual error budget:
```
31,536,000 seconds * 0.001 = 31,536 seconds = 8 hours 45 minutes 36 seconds
```

Each two-week cycle consumes 25 minutes of budget:
```
8h 45m 36s / 25m = 525.36 / 25 = 21.01 times
```

The pattern can repeat **21 times** before exceeding the annual budget (which
covers 42 weeks of the year, leaving about 10 weeks of buffer).

## Part C -- Reflection

Going from 99.99% to 99.999% is disproportionately harder because:

1. **Diminishing returns on prevention.** The failures that remain at 99.99% are
   the rarest and hardest to predict -- multi-region network partitions, obscure
   race conditions, cascading failures across many dependencies. Each additional
   "nine" requires eliminating entire classes of failure rather than fixing
   individual bugs.

2. **Operational constraints multiply.** At 99.999%, you have only 26 seconds
   of downtime per month. This means you likely cannot perform rolling
   deployments during business hours, cannot have any single points of failure,
   and need automated failover that completes in seconds. Every operational
   process must be redesigned.

3. **Testing becomes exhaustive.** You must test failure modes that have never
   occurred in production but theoretically could. The cost of testing and
   redundancy grows superlinearly with the availability target.
