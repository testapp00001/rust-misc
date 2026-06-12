# Module 07: Consistent Hashing

## Overview

Consistent hashing is a fundamental technique in distributed systems for mapping keys to nodes
in a way that minimizes disruption when nodes are added or removed. This module covers the
theory, algorithms, and practical considerations of consistent hashing and related partitioning
strategies.

## Background

### Naive Hashing (hash % N)

The simplest approach: `node = hash(key) % num_nodes`. While easy to understand, this approach
has a critical flaw: when the number of nodes changes, almost every key is remapped to a
different node. For a cluster of N nodes, adding or removing one node causes approximately
`(N-1)/N` of all keys to move -- roughly 99% redistribution in a 100-node cluster.

### Consistent Hashing (Karger et al., 1997)

David Karger et al. introduced consistent hashing in their 1997 paper "Consistent Hashing and
Random Trees: Distributed Caching Protocols for Relieving Hot Spots on the World Wide Web."

The key insight: instead of mapping keys directly to nodes, both keys and nodes are mapped to
points on a ring (the hash ring). A key is assigned to the first node found by walking clockwise
around the ring. When a node is added or removed, only the keys between it and its neighbor
need to be redistributed -- roughly 1/N of all keys.

**Virtual Nodes (VNodes):** To improve load balance, each physical node is mapped to multiple
points on the ring (virtual nodes). With enough virtual nodes (typically 100-200 per physical
node), the load distribution approaches uniform.

### Jump Consistent Hash (Google, 2014)

Lamping and Veach at Google proposed Jump Consistent Hash in their 2014 paper. It achieves
perfectly uniform distribution with minimal memory (O(1)) and O(ln(n)) time. The tradeoff:
it only supports adding/removing buckets at the end (numbered 0..n-1), making it unsuitable
for arbitrary node addition/removal.

### Rendezvous Hashing (Highest Random Weight)

Also known as "random choice hashing." For each key, compute `hash(key, node)` for all nodes
and pick the node with the highest hash value. This naturally provides consistent hashing
properties: only keys on a removed node need to be remapped, and the algorithm requires no
special data structures.

## Sharding Strategies

| Strategy | Pros | Cons |
|---|---|---|
| Range-based | Range queries efficient | Hotspots on sequential keys |
| Hash-based | Even distribution | Range queries require scatter-gather |
| Directory-based | Flexible placement | Single point of failure |
| Consistent Hashing | Minimal redistribution | Virtual nodes add complexity |

## Rebalancing and Hotspot Handling

When hotspots occur (a node receiving disproportionate traffic), strategies include:

- **Virtual nodes**: More vnodes improve load balance
- **Key salting**: Append random suffix to create multiple keys
- **Shard splitting**: Split a hot shard into multiple shards
- **Caching**: Cache hot data at the application layer
- **Rate limiting**: Throttle excessive requests to hot shards

## Exercise List

| # | Exercise | Difficulty | Key Concepts |
|---|---|---|---|
| p01 | Naive Hash Partitioning | Beginner | hash % N, redistribution |
| p02 | Consistent Hash Ring | Intermediate | BTreeMap ring, virtual nodes |
| p03 | Virtual Node Analysis | Intermediate | Load distribution, stddev |
| p04 | Jump Consistent Hash | Intermediate | Google's algorithm, O(1) memory |
| p05 | Rendezvous Hashing | Intermediate | HRW, highest random weight |
| p06 | Load Balance Analysis | Advanced | Strategy comparison, metrics |
| p07 | Rebalancing Cost | Advanced | Cost measurement, comparison |
| p08 | Hotspot Detection | Advanced | Monitoring, mitigation |
| p09 | Sharded KV Store | Advanced | Distributed data structure |
| p10 | Hash Function Comparison | Advanced | SipHash, FNV, performance |

## References

- Karger, D. et al. (1997). "Consistent Hashing and Random Trees"
- Lamping, J. & Veach, E. (2014). "A Fast, Minimal Memory, Consistent Hash Algorithm"
- Thaler, D. & Root, C. (2008). "Consistent Hashing and Load Balancing"
- DeCandia, G. et al. (2007). "Dynamo: Amazon's Highly Available Key-value Store"
