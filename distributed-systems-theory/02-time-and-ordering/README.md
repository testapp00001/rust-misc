# Module 02: Time and Ordering

## Lamport's 1978 Paper

Leslie Lamport's 1978 paper *"Time, Clocks, and the Ordering of Events in a
Distributed System"* is one of the most cited papers in computer science. It
established the foundational concepts for reasoning about time and ordering in
distributed systems.

The key insight is that **physical time is unreliable in distributed systems**, and
we need logical mechanisms to establish ordering of events.

## The Happened-Before Relation

Lamport defined the **happened-before relation** (->) as the smallest transitive
relation satisfying:

1. If a and b are events in the same process and a occurs before b, then a -> b
2. If a is the sending of a message and b is the receipt of that message, then a -> b
3. If a -> b and b -> c, then a -> c (transitivity)

Two events are **concurrent** (written a || b) if neither a -> b nor b -> a.

The happened-before relation captures **causality**: if a -> b, then a could have
causally influenced b.

## Physical Clock Problems

Physical clocks are unreliable in distributed systems:

- **Clock drift**: Clocks run at slightly different rates (typically 10-100 ppm,
  meaning 10-100 microseconds per second drift).
- **NTP limitations**: Network Time Protocol can synchronize clocks to within 1-10ms
  on a LAN, but WAN synchronization is worse (10-100ms).
- **Clock jumps**: NTP corrections can cause sudden jumps backward or forward.
- **Leap seconds**: Occasional leap seconds break monotonicity.

These problems mean that physical timestamps alone cannot be used to determine
event ordering. A message sent at physical time T1 might be received at physical
time T0 < T1 on a faster clock.

## Lamport Timestamps vs Vector Clocks

### Lamport Timestamps

A Lamport timestamp is a single integer counter. Rules:
- Increment before each local event
- On send: increment and attach to message
- On receive: set counter = max(local, received) + 1

Properties:
- If a -> b, then L(a) < L(b)
- But L(a) < L(b) does NOT imply a -> b (concurrent events can have any order)

### Vector Clocks

A vector clock is a vector of integers, one per process. Rules:
- Increment own component before each local event
- On send: increment own component and attach to message
- On receive: element-wise max with received vector, then increment own component

Properties:
- If a -> b, then VC(a) < VC(b) (component-wise)
- VC(a) < VC(b) implies a -> b
- Two events are concurrent iff neither VC(a) <= VC(b) nor VC(b) <= VC(a)

Vector clocks provide **complete causality tracking** at the cost of O(n) space.

## Exercise List

| # | Exercise | Difficulty | Description |
|---|----------|------------|-------------|
| 1 | `p01_physical_clocks` | Easy | Simulate physical clock drift with configurable drift rate |
| 2 | `p02_clock_drift_sim` | Medium | Simulate multiple clocks drifting apart over time |
| 3 | `p03_happened_before` | Medium | Implement the happened-before relation |
| 4 | `p04_lamport_timestamps` | Medium | Implement Lamport Timestamps with send/receive rules |
| 5 | `p05_vector_clocks` | Hard | Implement Vector Clocks with causality tracking |
| 6 | `p06_version_vectors` | Hard | Implement Version Vectors for a key-value store |
| 7 | `p07_causal_ordering` | Hard | Implement causal ordering of events via topological sort |
| 8 | `p08_concurrent_detection` | Hard | Detect concurrent events in a simulated distributed system |
| 9 | `p09_hybrid_logical_clocks` | Hard | Implement Hybrid Logical Clocks (HLC) |
| 10 | `p10_hlc_key_value_store` | Expert | Build a causal-consistent KV store using HLC |

## References

- Lamport, L. (1978). *Time, Clocks, and the Ordering of Events in a Distributed
  System*. Communications of the ACM, 21(7), 558-565.
- Fidge, C. J. (1988). *Timestamps in Message-Passing Systems That Preserve the
  Partial Ordering*. Australian Computer Science Communications, 10(1), 56-66.
- Mattern, F. (1989). *Virtual Time and Global States of Distributed Systems*.
  Parallel and Distributed Algorithms, 215-226.
- Kulkarni, S. S., Demirbas, M., Madappa, D., Avva, B., & Leone, M. (2014).
  *Logical Physical Clocks and Consistent Snapshots in Globally Distributed Databases*.
  DISC 2014.
- Kulkarni, S. S., & Leonard, M. (2016). *Logical Physical Clocks*. In Advances in
  Real-Time and Fault-Tolerant Systems. Springer.
