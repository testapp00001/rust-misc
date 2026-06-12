# Module 10: Clock Synchronization

## Historical Context

Clock synchronization is one of the oldest and most fundamental problems in distributed systems. Lamport's seminal 1978 paper "Time, Clocks, and the Ordering of Events in a Distributed System" established that physical clocks are insufficient for ordering events, yet they remain essential for practical systems.

**NTP (Network Time Protocol)**, developed by David Mills in 1985, remains the most widely deployed synchronization protocol. NTP uses a client-server model where a client samples round-trip times to multiple servers and estimates clock offset using the formula:

```
offset = (t1 - t0) - (t3 - t2)
         ─────────────────────
                2
```

where t0, t1 are the client send/server receive timestamps and t2, t3 are the server send/client receive timestamps. Typical accuracy is 1-50ms over the public internet, limited by asymmetric network delays and variable routing.

**Google TrueTime** (2012), introduced with Google Spanner, represents a paradigm shift. Rather than hiding clock uncertainty, TrueTime exposes it directly. Each TrueTime call returns an interval [earliest, latest] with high confidence that the current time lies within that interval. This is achieved using a combination of:

- Atomic clocks ( cesium and rubidium ) for stability
- GPS receivers for absolute time reference
- Local crystal oscillators for continuity during GPS outages

TrueTime typically achieves 1-7ms uncertainty, with atomic clocks providing drift rates of ~10^-12 (compared to ~10^-5 for quartz crystals).

**Spanner's External Consistency** model exploits TrueTime by implementing a "commit wait" protocol: after writing a timestamp, Spanner waits until it can guarantee that no future timestamp assignment could produce a smaller value. This requires the uncertainty interval to be small enough that waiting it out is practical.

**Hybrid Logical Clocks (HLC)**, proposed by Kulkarni et al. (2014), combine physical and logical time. An HLC timestamp consists of a physical component (tracking wall clock) and a logical counter (for events within the same physical time unit). This provides:

- Timestamps close to physical time (useful for diagnostics and caching)
- Causality tracking (if A happened-before B, then HLC(A) < HLC(B))
- No requirement for synchronized physical clocks (unlike NTP/TrueTime)

## Formal Definition

### Clock Offset

Given two clocks C_A and C_B, the offset at real time t is:

```
offset(A, B, t) = C_A(t) - C_B(t)
```

A perfect synchronization protocol drives offset toward zero.

### Clock Uncertainty Bound

An uncertainty bound U satisfies:

```
For all t: |C_true(t) - C_approx(t)| <= U
```

where C_true is the true time and C_approx is the estimated time.

### Happened-Before (HLC)

For HLC timestamps (pt_A, lc_A) and (pt_B, lc_B):

```
(pt_A, lc_A) < (pt_B, lc_B) iff
    pt_A < pt_B OR
    (pt_A == pt_B AND lc_A < lc_B)
```

This ordering is total (every pair of timestamps is comparable) and respects the happened-before relation.

### TrueTime External Consistency

For any two transactions T1 and T2 where T1 commits before T2 starts:

```
commit_time(T1) < commit_time(T2)
```

This is guaranteed by waiting for the uncertainty interval to pass (commit wait).

## Exercise List

| # | Exercise | Difficulty | Description |
|---|----------|------------|-------------|
| 1 | NTP Clock Simulation | Easy | Implement the NTP offset estimation formula and track synchronization uncertainty across multiple rounds. |
| 2 | Clock Drift Model | Easy | Model crystal oscillator drift in parts-per-million and compute accumulated error over time. |
| 3 | TrueTime Interval Clock | Medium | Build an interval-based clock that exposes uncertainty bounds, simulating TrueTime's [earliest, latest] API. |
| 4 | HLC Implementation | Medium | Implement Hybrid Logical Clocks with physical+logical timestamp ordering and causality tracking. |
| 5 | HLC Causality | Medium | Verify and demonstrate that HLC timestamps preserve the happened-before relation across distributed events. |
| 6 | Commit Wait | Hard | Simulate Spanner's commit wait protocol, demonstrating how uncertainty bounds enable external consistency. |
| 7 | Spanner External Consistency | Hard | Model a simplified Spanner transaction protocol using TrueTime and commit wait for linearizable ordering. |
| 8 | Clock Comparison | Easy | Compare and contrast different clock models: NTP accuracy vs TrueTime uncertainty vs HLC causality guarantees. |

## References

1. Lamport, L. (1978). "Time, Clocks, and the Ordering of Events in a Distributed System." Communications of the ACM, 21(7), 558-565.
2. Mills, D.L. (1991). "Internet Time Synchronization: the Network Time Protocol." IEEE Transactions on Communications, 39(10), 1482-1493.
3. Corbett, J.C. et al. (2013). "Spanner: Google's Globally-Distributed Database." ACM Transactions on Computer Systems, 31(3), 1-22.
4. Kulkarni, S. et al. (2014). "Logical Physical Clocks and Consistent Snapshots in Globally Distributed Databases." International Conference on Principles of Distributed Systems (OPODIS).
5. Levine, J. (2006). "Introduction to Computer Time Synchronization." NTP documentation.
6. Google Cloud. "Spanner TrueTime API." https://cloud.google.com/spanner/docs/true-time
