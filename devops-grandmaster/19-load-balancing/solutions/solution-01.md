# Solution 01: Load Balancing Algorithms

## Part A -- Algorithm Descriptions

### 1. Round Robin

Round Robin distributes requests to servers in a fixed circular order:
server 1, server 2, server 3, server 1, server 2, and so on. Each server
receives the same number of requests over time, regardless of its current
load or capacity. It is the simplest algorithm and works well when all
servers are identical and requests have similar processing costs.

### 2. Weighted Round Robin

Weighted Round Robin extends Round Robin by assigning a weight to each
server. A server with weight 3 receives three times as many requests as a
server with weight 1. The distribution follows the ratio of weights. This
is useful when servers have different hardware capabilities and you want
more powerful machines to handle proportionally more traffic.

### 3. Least Connections

Least Connections routes each new request to the server with the fewest
active (concurrent) connections at that moment. It dynamically adapts to
the current load on each server. This algorithm is effective when request
processing times vary widely, because a server handling fast requests will
free up connections quickly and naturally attract more traffic.

### 4. IP Hash

IP Hash computes a hash of the client's IP address and uses it to select a
server. The same client IP always maps to the same server (as long as the
server pool does not change). This provides a form of session affinity
without cookies. It is useful for stateful applications where a client must
consistently reach the same backend.

### 5. Random

The Random algorithm selects a server uniformly at random for each request.
Over a large number of requests, the distribution converges to roughly
equal shares. It has virtually no coordination overhead because there is no
state to maintain (no counters, no connection tracking). It works well for
stateless services with homogeneous servers.

### 6. Least Response Time

Least Response Time routes each request to the server with the lowest
average response time (and fewest active connections as a tiebreaker). It
accounts for the fact that some backends may be slower due to hardware
differences, network proximity, or downstream dependencies. This produces
the best end-user latency but requires the load balancer to continuously
measure response times.

## Part B -- Scenario Matching

### Scenario 1: Identical hardware, uniform traffic

**Algorithm: Round Robin**

When all servers are identical and traffic is uniform, Round Robin provides a
perfectly even distribution with minimal overhead. There is no benefit to
using a more complex algorithm because every server handles requests at the
same rate.

### Scenario 2: Different CPU/RAM specs

**Algorithm: Weighted Round Robin**

Weighted Round Robin lets you assign higher weights to more powerful machines.
A 16-core machine could receive weight 4 while a 4-core machine receives
weight 1, ensuring the distribution matches hardware capacity.

### Scenario 3: Stateful application, IP-based affinity required

**Algorithm: IP Hash**

IP Hash maps each client IP to a specific server, providing session affinity
without cookies. This ensures a user's subsequent requests reach the same
backend for the duration of their session.

### Scenario 4: Unpredictable latency from external APIs

**Algorithm: Least Response Time**

When response times are unpredictable, Least Response Time dynamically routes
requests to whichever backend is currently fastest. If one backend is waiting
on a slow external API call, the load balancer will route around it
automatically.

### Scenario 5: Stateless service, simplest approach

**Algorithm: Round Robin (or Random)**

Both work. Round Robin is the simplest deterministic choice. Random is even
simpler in implementation (no counter needed) but gives no guarantee of even
distribution on small sample sizes. For most practical purposes, Round Robin
is the default "just works" choice.

### Scenario 6: Long-lived connections (WebSocket)

**Algorithm: Least Connections**

With long-lived connections, servers accumulate connections over time. Least
Connections ensures new connections go to the server with the fewest active
connections, preventing a single server from being overwhelmed. Round Robin
would distribute connection *attempts* evenly but would not account for the
fact that some connections last much longer than others.

## Part C -- Trade-off Analysis

### 1. Why might Least Connections perform poorly if backend servers have very different processing capacities?

Least Connections tracks the *number* of active connections, not the
*resource usage* of those connections. If a 4-core server and a 16-core
server both have 10 active connections, Least Connections considers them
equally loaded. In reality, the 4-core server is 4x more loaded per core.
The 4-core server will become a bottleneck while the 16-core server remains
underutilised. The fix is to use a capacity-aware variant (sometimes called
"weighted least connections") where each server's connection count is divided
by its weight before comparison.

### 2. What happens to IP Hash when many users share one NAT IP?

When a large corporate network or mobile carrier funnels thousands of users
through a single public IP, IP Hash treats all those users as one client.
They all map to the same backend server. This creates severe load imbalance:
one server handles a disproportionate share of traffic while others sit idle.
If the mapped server becomes overloaded or fails, all those users lose access
simultaneously. Solutions include hashing on additional attributes (e.g.,
X-Forwarded-For header, a cookie, or a combination of IP and User-Agent) or
switching to a different algorithm.

### 3. Under what conditions does Random outperform Round Robin?

Random outperforms Round Robin in two main scenarios:

1. **Bursty traffic with short windows**: If you measure over a short time
   window, Round Robin's deterministic pattern can accidentally align with
   traffic patterns, causing periodic spikes. Random's jitter smooths this
   out.

2. **Thundering herd on recovery**: When a server comes back online after a
   failure, Round Robin immediately starts sending it a full share of
   traffic. If many requests arrive simultaneously (thundering herd), the
   recovering server may be overwhelmed. Random naturally distributes the
   burst unevenly, giving the new server time to warm up.

3. **Multiple load balancer instances**: If you run several independent load
   balancers (e.g., in different availability zones) and each uses Round
   Robin starting from a different offset, their combined effect may still
   create uneven distribution. Random avoids this coordination problem
   entirely.

In practice, the difference is marginal for most workloads, and Round Robin's
predictability is often preferred for debugging and capacity planning.
