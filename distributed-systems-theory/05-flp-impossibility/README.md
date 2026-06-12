# Module 05: FLP Impossibility

## The Fischer-Lynch-Paterson (FLP) Result

In 1982, Michael Fischer, Nancy Lynch, and Michael Paterson published one of the most
important results in distributed computing. The FLP impossibility theorem states:

> **Theorem (FLP, 1985):** In an asynchronous distributed system with even a single
> crash-faulty process, no deterministic consensus protocol can guarantee termination.

This means that there is no protocol that always reaches consensus in bounded time when
processes may crash.

### System Model

- **Asynchronous:** Messages are arbitrarily delayed (but eventually delivered). No
  bounds on process speeds or message latencies.
- **Crash failure:** A process may stop executing at any point but never behaves
  maliciously. It simply stops sending or receiving messages.
- **Consensus:** Every correct process eventually decides on the same value from the
  set of proposed values.

### The Indistinguishability Argument

The proof relies on a powerful indistinguishability argument:

1. Suppose a deterministic protocol P solves consensus.
2. Consider two initial configurations: one where all processes propose 0, and one
   where all but one propose 0 (with one process proposing 1).
3. Both configurations must decide 0 (by validity).
4. Due to asynchrony, a process can be delayed arbitrarily long. While it is delayed,
   the other processes cannot distinguish whether it is "slow" or "crashed."
5. If the protocol decides 0 in this state, it must also decide 0 if the delayed
   process were actually crashed (since they look identical).
6. By carefully constructing a sequence of reachable configurations, the proof shows
   that the protocol can be "tricked" into an infinite loop of undecided states.

### How Real Systems Work Around FLP

Since FLP proves deterministic consensus impossible in pure async systems, real systems
use one of several techniques:

1. **Partial Synchrony:** Assume that after some unknown Global Stabilization Time
   (GST), all messages are delivered within a known bound. Raft, Paxos, and PBFT all
   rely on this.

2. **Failure Detectors:** Use unreliable failure detectors that eventually become
   accurate. The omega (omega) failure detector (which eventually correctly identifies
   the eventual leader) is sufficient for consensus.

3. **Randomization:** Use random coin flips to break symmetry. Randomized protocols
   (like Ben-Or's algorithm) guarantee termination with probability 1 but not in
   bounded time.

4. **Timeouts (Practical):** Use adaptive timeouts that eventually stabilize. This is
   how Raft works -- election timeouts are random and adaptive, so eventually a leader
   is elected.

### Connection to Raft's Election Timeout

Raft cleverly sidesteps FLP through its randomized election timeout:

- Each node picks a random election timeout.
- The node with the shortest timeout becomes a candidate first.
- If no majority is reached, the node restarts with a new random timeout.
- Under partial synchrony, eventually the timeout will be long enough for messages
  to be delivered before the next election, allowing a leader to be elected.

This is essentially an application of the failure detector approach: the timeout acts
as an omega failure detector that eventually works correctly.

### Connection to Two Generals (Module 01)

The FLP impossibility is a generalization of the Two Generals impossibility. Two Generals
shows that reliable agreement is impossible over unreliable channels. FLP extends this to
show that even with reliable channels (messages eventually arrive), crash failures in
async systems still prevent guaranteed termination.

## Exercises

| # | Exercise | Difficulty | Description |
|---|----------|------------|-------------|
| 01 | Indistinguishability | ★★☆ | Show that an observer cannot distinguish crashed from slow |
| 02 | Deterministic Consensus | ★★☆ | Implement consensus and show it deadlocks under asynchrony |
| 03 | Deadlock Demonstration | ★★★ | 3-process consensus where crash causes indefinite blocking |
| 04 | Failure Detectors | ★★★ | Implement omega-style failure detectors with suspect tracking |
| 05 | Partial Synchrony | ★★★★ | Simulate GST and show consensus becomes possible |
| 06 | Randomized Consensus | ★★★★ | Implement coin-flip consensus that terminates with prob. 1 |
| 07 | Timeout Tradeoffs | ★★★ | Analyze false positive rate vs. detection latency |

## Key Theorems

- **FLP Impossibility (1985):** No deterministic consensus protocol guarantees
  termination in a crash-faulty async system.
- **Failure Detector Completeness:** The omega failure detector is the weakest
  detector sufficient for consensus.
- **Partial Synchrony:** After GST, consensus protocols (Paxos, PBFT) terminate
  in bounded time.
