# Solution 02: Round-Robin Implementation

## Part A: Basic Round-Robin Implementation

```python
class RoundRobinLoadBalancer:
    def __init__(self, backends):
        self.backends = backends
        self.current_index = 0

    def get_next_server(self):
        server = self.backends[self.current_index]
        self.current_index = (self.current_index + 1) % len(self.backends)
        return server

# Usage
lb = RoundRobinLoadBalancer(["Server-A", "Server-B", "Server-C"])
for i in range(9):
    print(f"Request {i+1} → {lb.get_next_server()}")
```

```
Request 1 → Server-A
Request 2 → Server-B
Request 3 → Server-C
Request 4 → Server-A
Request 5 → Server-B
Request 6 → Server-C
Request 7 → Server-A
Request 8 → Server-B
Request 9 → Server-C
```

### Why This Works

The modulo operation (`%`) wraps the counter back to 0 after reaching the
end of the list. This creates an infinite cycle through the backends without
ever going out of bounds. The `current_index` is the only state the load
balancer needs to maintain.

### Thread Safety Note

In a production system, the counter must be thread-safe. Multiple worker
processes serving requests concurrently could read and increment the counter
simultaneously. Solutions:

```python
import threading

class ThreadSafeRoundRobin:
    def __init__(self, backends):
        self.backends = backends
        self.current_index = 0
        self.lock = threading.Lock()

    def get_next_server(self):
        with self.lock:
            server = self.backends[self.current_index]
            self.current_index = (self.current_index + 1) % len(self.backends)
            return server
```

An alternative is to use an atomic counter, which avoids lock contention.

## Part B: Trace Request Distribution

```
Backends: [Server-A, Server-B, Server-C]

Request  1 → Server: A    (index=0, then index becomes 1)
Request  2 → Server: B    (index=1, then index becomes 2)
Request  3 → Server: C    (index=2, then index becomes 0)
Request  4 → Server: A    (index=0, then index becomes 1)
Request  5 → Server: B    (index=1, then index becomes 2)
Request  6 → Server: C    (index=2, then index becomes 0)
Request  7 → Server: A    (index=0, then index becomes 1)
Request  8 → Server: B    (index=1, then index becomes 2)
Request  9 → Server: C    (index=2, then index becomes 0)
Request 10 → Server: A    (index=0, then index becomes 1)
Request 11 → Server: B    (index=1, then index becomes 2)
Request 12 → Server: C    (index=2, then index becomes 0)
```

### Distribution Summary

| Server | Requests Handled | Percentage |
|--------|-----------------|------------|
| Server-A | 4 | 33.3% |
| Server-B | 4 | 33.3% |
| Server-C | 4 | 33.3% |

Perfectly even distribution -- the key advantage of round-robin.

## Part C: Round-Robin Limitations

### 1. Does basic round-robin account for server capacity differences?

**No.** Basic round-robin treats all servers identically. Server-B (200 req/s
capacity) receives the same number of requests as Server-A (100 req/s) and
Server-C (100 req/s). The total capacity is 400 req/s, but round-robin
distributes as if all servers have equal capacity.

### 2. What happens to Server-A and Server-C under high load?

```
At 300 requests/second:
  Server-A: receives 100 req/s → at 100% capacity → saturated
  Server-B: receives 100 req/s → at 50% capacity → underutilized
  Server-C: receives 100 req/s → at 100% capacity → saturated

Result: Server-A and Server-C are overloaded. Response times increase.
Requests queue up. Some requests timeout. Meanwhile, Server-B has spare
capacity that is not being used.
```

The load balancer is distributing requests evenly, but the *load* is not
even because the servers have different capacities.

### 3. How to handle unequal capacity?

**Weighted Round-Robin** assigns weights proportional to capacity:

```
Server-A: weight=1 (capacity 100 req/s)
Server-B: weight=2 (capacity 200 req/s)
Server-C: weight=1 (capacity 100 req/s)

Effective distribution: A gets 25%, B gets 50%, C gets 25%
At 300 req/s: A=75, B=150, C=75 → all at 75% capacity → balanced
```

## Part D: Weighted Round-Robin Implementation

```python
class WeightedRoundRobin:
    def __init__(self, backends_with_weights):
        # backends_with_weights: [("Server-A", 1), ("Server-B", 2), ("Server-C", 1)]
        self.backends = backends_with_weights
        self.current_index = 0
        self.current_weight = 0
        self.max_weight = max(w for _, w in backends_with_weights)
        self.gcd_weight = self._gcd([w for _, w in backends_with_weights])

    def _gcd(self, weights):
        from math import gcd
        result = weights[0]
        for w in weights[1:]:
            result = gcd(result, w)
        return result

    def get_next_server(self):
        while True:
            self.current_index = (self.current_index + 1) % len(self.backends)
            if self.current_index == 0:
                self.current_weight -= self.gcd_weight
                if self.current_weight <= 0:
                    self.current_weight = self.max_weight
            name, weight = self.backends[self.current_index]
            if weight >= self.current_weight:
                return name
```

### Simpler Approach: Expanded List

```python
class SimpleWeightedRoundRobin:
    def __init__(self, backends_with_weights):
        # Expand: [("A",1), ("B",2), ("C",1)] → ["A", "B", "B", "C"]
        self.expanded = []
        for name, weight in backends_with_weights:
            self.expanded.extend([name] * weight)
        self.current_index = 0

    def get_next_server(self):
        server = self.expanded[self.current_index]
        self.current_index = (self.current_index + 1) % len(self.expanded)
        return server
```

### Trace with Weights [1, 2, 1]

Using the expanded list approach: `["A", "B", "B", "C"]`

```
Request 1 → Server: A    (expanded index=0)
Request 2 → Server: B    (expanded index=1)
Request 3 → Server: B    (expanded index=2)
Request 4 → Server: C    (expanded index=3)
Request 5 → Server: A    (expanded index=0, wraps)
Request 6 → Server: B    (expanded index=1)
Request 7 → Server: B    (expanded index=2)
Request 8 → Server: C    (expanded index=3)
```

### Distribution with Weights

| Server | Weight | Requests (of 8) | Percentage |
|--------|--------|-----------------|------------|
| Server-A | 1 | 2 | 25% |
| Server-B | 2 | 4 | 50% |
| Server-C | 1 | 2 | 25% |

Server-B receives twice as many requests as Server-A or Server-C,
proportional to its weight (capacity).

### Common Mistakes to Avoid

- **Forgetting modulo wrap-around.** Without `% len(backends)`, the index
  grows indefinitely and causes an index-out-of-bounds error.
- **Not making the counter thread-safe.** In multi-threaded servers,
  concurrent access to the counter causes race conditions and uneven
  distribution.
- **Using round-robin for long-lived connections.** Round-robin distributes
  *new* connections evenly, but if connections have different durations,
  the actual load becomes uneven over time. Use least-connections instead.
- **Confusing request distribution with load distribution.** Round-robin
  ensures equal requests, not equal load. If requests have varying
  processing times, the load will be uneven.

## Key Takeaway

Round-robin is the simplest load balancing algorithm: a single counter and
modulo arithmetic. It works perfectly when all servers have equal capacity
and requests have uniform duration. Weighted round-robin extends this to
handle unequal capacities. However, round-robin remains blind to real-time
server load -- it distributes requests based on count, not on how busy each
server actually is.
