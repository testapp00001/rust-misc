# 74 - Distributed Systems Patterns

> **Previous:** [73 - Service Discovery](../73-service-discovery/README.md)
> **Next:** [75 - Multi-Cloud and Hybrid](../75-multi-cloud-and-hybrid/README.md)

## The Problem

You have multiple services running across multiple hosts. They can discover each other. But the network between them is unreliable. Services crash. Packets get lost. Clocks drift. When Service A calls Service B, what happens if B is slow? What if B is down? What if the response was sent but A never received it? What if both A and B think the other is dead?

Distributed systems are fundamentally different from single-machine programs. You cannot assume reliable delivery, ordered messages, or consistent clocks. The patterns in this lesson are the battle-tested solutions to these problems — the accumulated wisdom of decades of building systems that work despite everything going wrong.

---

## The Naive Way

Treat distributed services like local function calls.

```python
# The "it works on my laptop" approach to distributed systems
def process_order(order):
    user = user_service.get_user(order.user_id)           # What if this hangs?
    product = inventory_service.reserve(order.product_id)  # What if this fails?
    payment = payment_service.charge(order.total)          # What if this succeeds but the next step fails?
    notification = email_service.send_confirmation(order)  # What if this is slow?

    return {"status": "complete", "order_id": order.id}
```

**Why this fails:**
- No timeout — if `user_service` hangs, the caller hangs forever.
- No retry logic — a transient network blip causes permanent failure.
- No circuit breaker — if `payment_service` is down, every request waits for the timeout, queueing up until the entire system crashes.
- No idempotency — retrying a payment charge could double-charge the customer.
- No compensation — if `email_service` fails after payment succeeds, the customer is charged but never notified.
- No fallback — the call either succeeds or fails completely; there is no graceful degradation.

---

## The Right Way

Apply fundamental distributed systems patterns to handle the realities of networked computing.

### CAP Theorem

The CAP theorem, proven by Seth Gilbert and Nancy Lynch in 2002, states that a distributed data store can provide at most two out of three guarantees:

- **Consistency (C):** Every read receives the most recent write or an error. All nodes see the same data at the same time.
- **Availability (A):** Every request receives a non-error response, without guaranteeing it contains the most recent write.
- **Partition Tolerance (P):** The system continues to operate despite network partitions between nodes.

```
        Consistency
           /\
          /  \
         /    \
        / CP   \
       /--------\
      / AP    CA \
     /____________\
Availability    Partition
                Tolerance
```

In practice, network partitions are unavoidable — you must tolerate them. The real choice is between **CP** (consistency over availability) and **AP** (availability over consistency).

**CP Systems** (choose consistency during partitions):
- etcd, ZooKeeper, Consul (for KV store) — refuse writes during partitions to maintain consistency.
- Traditional RDBMS with synchronous replication.
- MongoDB (with majority write concern).

**AP Systems** (choose availability during partitions):
- Cassandra, DynamoDB, CouchDB — accept writes during partitions, resolve conflicts later.
- DNS — serves stale data rather than failing.
- Caching systems — serve cached data even if it might be stale.

**CA Systems** (consistency and availability, but cannot tolerate partitions):
- Single-node databases — not distributed, so partitions are not relevant.
- Not achievable in a real distributed system.

### PACELC Theorem

PACELC extends CAP by acknowledging that even when there is no partition, there is a trade-off:

- **If Partition:** choose **A**vailability or **C**onsistency.
- **Else (no partition):** choose **L**atency or **C**onsistency.

| System | Partition: A or C | Else: L or C |
|--------|-------------------|---------------|
| Cassandra | A (always writable) | L (eventual consistency, low latency) |
| MongoDB | C (majority write) | C (read from primary) |
| DynamoDB | A (always writable) | L (eventual consistency) |
| PostgreSQL | C (synchronous replication) | C (strong consistency) |
| CockroachDB | C (serializable) | C (serializable, higher latency) |

### Consensus Algorithms: Raft and Paxos

Consensus is the problem of getting multiple nodes to agree on a single value. Raft and Paxos are the two major consensus algorithms.

**Raft (used by etcd, Consul, Nomad):**

Raft decomposes consensus into three sub-problems:
1. **Leader election** — one node is elected leader; all writes go through it.
2. **Log replication** — the leader replicates its log to followers.
3. **Safety** — ensures logs are consistent across all nodes.

```
Leader Election:
  +-------+      +-------+      +-------+
  |Node 1 |      |Node 2 |      |Node 3 |
  | FOLLOWER     | FOLLOWER     | FOLLOWER |
  +-------+      +-------+      +-------+

  Node 2 starts election:
  +-------+      +-------+      +-------+
  |Node 1 |      |Node 2 |      |Node 3 |
  | FOLLOWER     | CANDIDATE    | FOLLOWER |
  +-------+      +-------+      +-------+

  Node 2 wins:
  +-------+      +-------+      +-------+
  |Node 1 |      |Node 2 |      |Node 3 |
  | FOLLOWER     | LEADER       | FOLLOWER |
  +-------+      +-------+      +-------+

  Client writes to leader:
  +-------+      +-------+      +-------+
  |Node 1 |<-----|Node 2 |----->|Node 3 |
  | FOLLOWER     | LEADER       | FOLLOWER |
  | [A,B,C]      | [A,B,C,D]    | [A,B,C]  |
  +-------+      +-------+      +-------+

  Log replication:
  +-------+      +-------+      +-------+
  |Node 1 |      |Node 2 |      |Node 3 |
  | FOLLOWER     | LEADER       | FOLLOWER |
  | [A,B,C,D]    | [A,B,C,D]    | [A,B,C,D]|
  +-------+      +-------+      +-------+
  (all nodes now agree)
```

**Key properties:**
- Requires a majority (quorum) of nodes to agree: `quorum = (n/2) + 1`.
- Tolerates `f` failures with `2f + 1` nodes (3 nodes tolerate 1 failure, 5 tolerate 2).
- Strong consistency — reads from the leader are always up-to-date.
- Not available during network partitions if a quorum cannot be reached.

### Eventual Consistency vs Strong Consistency

**Strong Consistency:**
Every read returns the most recent write. The system behaves as if there is a single copy of the data.

```python
# Strong consistency: read-your-writes guarantee
def update_and_read(user_id, new_name):
    db.execute("UPDATE users SET name = ? WHERE id = ?", new_name, user_id)
    # Strong consistency: the next read is guaranteed to see this write
    user = db.query("SELECT * FROM users WHERE id = ?", user_id)
    assert user["name"] == new_name  # always true
    return user
```

**When to use:** Financial transactions, inventory management, leaderboards where exact ordering matters.

**Eventual Consistency:**
If no new updates are made, eventually all reads will return the last updated value. There is a window where reads may return stale data.

```python
# Eventual consistency: writes propagate asynchronously
def update_user(user_id, new_name):
    # Write to primary
    primary_db.execute("UPDATE users SET name = ? WHERE id = ?", new_name, user_id)
    # Replication happens asynchronously
    # Other replicas will eventually have this update

def read_user(user_id):
    # Read from a replica — might be slightly behind
    return replica_db.query("SELECT * FROM users WHERE id = ?", user_id)
```

**When to use:** Social media feeds, product catalogs, DNS records, user profiles where a few seconds of staleness is acceptable.

### Saga Pattern

A saga is a sequence of local transactions. Each transaction updates the database and publishes an event. The next transaction is triggered by the event. If a step fails, compensating transactions undo the preceding steps.

```
Orchestration Saga:
  +--------+    +----------+    +-----------+    +-------------+
  | Order  |--->| Payment  |--->| Inventory |--->| Notification|
  | Service|    | Service  |    | Service   |    | Service     |
  +--------+    +----------+    +-----------+    +-------------+
       |              |                |                |
       |         (success)        (success)         (success)
       |              |                |                |
       v              v                v                v
  [Order Created] [Payment Made] [Item Reserved] [Email Sent]

  If Inventory fails:
  +--------+    +----------+    +-----------+
  | Order  |--->| Payment  |--->| Inventory |
  | Service|    | Service  |    | Service   |
  +--------+    +----------+    +-----------+
       ^              ^                |
       |              |           (failure)
       |              |                |
  [Order Cancelled] [Payment Refunded]  |
```

**Choreography Saga:**

```python
# Each service publishes events and listens for events from others

# Order Service
class OrderService:
    def create_order(self, order):
        order.status = "PENDING"
        db.save(order)
        event_bus.publish("OrderCreated", {"order_id": order.id, "amount": order.total})

    def handle_payment_failed(self, event):
        order = db.get(event["order_id"])
        order.status = "CANCELLED"
        db.save(order)

    def handle_inventory_reserved(self, event):
        order = db.get(event["order_id"])
        order.status = "CONFIRMED"
        db.save(order)

# Payment Service
class PaymentService:
    def handle_order_created(self, event):
        try:
            payment_gateway.charge(event["amount"])
            event_bus.publish("PaymentCompleted", {"order_id": event["order_id"]})
        except PaymentError:
            event_bus.publish("PaymentFailed", {"order_id": event["order_id"]})

# Inventory Service
class InventoryService:
    def handle_payment_completed(self, event):
        try:
            inventory.reserve(event["order_id"])
            event_bus.publish("InventoryReserved", {"order_id": event["order_id"]})
        except OutOfStock:
            event_bus.publish("InventoryFailed", {"order_id": event["order_id"]})
            # Trigger compensation: refund payment
            event_bus.publish("RefundPayment", {"order_id": event["order_id"]})
```

**Orchestration Saga:**

```python
# A central orchestrator coordinates the saga
class OrderSagaOrchestrator:
    def execute(self, order):
        saga = SagaBuilder() \
            .step("create_order",
                  action=lambda: order_service.create(order),
                  compensate=lambda: order_service.cancel(order.id)) \
            .step("charge_payment",
                  action=lambda: payment_service.charge(order),
                  compensate=lambda: payment_service.refund(order.id)) \
            .step("reserve_inventory",
                  action=lambda: inventory_service.reserve(order),
                  compensate=lambda: inventory_service.release(order.id)) \
            .step("send_notification",
                  action=lambda: notification_service.notify(order),
                  compensate=lambda: None) \
            .build()

        try:
            saga.execute()
            return {"status": "complete"}
        except SagaStepFailed as e:
            return {"status": "failed", "failed_step": e.step}
```

### Idempotency

An idempotent operation produces the same result whether it is called once or many times. This is essential for safe retries in distributed systems.

```python
# Non-idempotent — retrying charges the customer twice
def charge_customer(customer_id, amount):
    payment_gateway.charge(customer_id, amount)

# Idempotent — same idempotency_key always returns the same result
def charge_customer(customer_id, amount, idempotency_key):
    # Check if we already processed this key
    existing = db.query(
        "SELECT * FROM payments WHERE idempotency_key = ?", idempotency_key
    )
    if existing:
        return existing  # Return previous result

    # Process payment
    result = payment_gateway.charge(customer_id, amount)

    # Store result with idempotency key
    db.execute(
        "INSERT INTO payments (idempotency_key, customer_id, amount, result) VALUES (?, ?, ?, ?)",
        idempotency_key, customer_id, amount, result
    )
    return result

# Usage
import uuid
idempotency_key = str(uuid.uuid4())
charge_customer("cust_123", 99.99, idempotency_key)
charge_customer("cust_123", 99.99, idempotency_key)  # returns same result, no double charge
```

### Circuit Breaker

A circuit breaker prevents an application from repeatedly trying an operation that is likely to fail. It monitors failures and "opens" the circuit when a threshold is reached, failing fast instead of waiting for timeouts.

```
State Machine:
  CLOSED ──(failures exceed threshold)──> OPEN
    ^                                       |
    |                                       |
    └──(success after half-open)── HALF-OPEN ┘
                                        |
                    (timeout expires)───┘
```

```python
import time
from enum import Enum

class CircuitState(Enum):
    CLOSED = "closed"        # Normal operation, requests pass through
    OPEN = "open"            # Fail fast, requests are rejected immediately
    HALF_OPEN = "half_open"  # Allow a few test requests through

class CircuitBreaker:
    def __init__(self, failure_threshold=5, recovery_timeout=30, half_open_max=3):
        self.failure_threshold = failure_threshold
        self.recovery_timeout = recovery_timeout
        self.half_open_max = half_open_max

        self.state = CircuitState.CLOSED
        self.failure_count = 0
        self.last_failure_time = None
        self.half_open_count = 0

    def call(self, func, *args, **kwargs):
        if self.state == CircuitState.OPEN:
            if time.time() - self.last_failure_time > self.recovery_timeout:
                self.state = CircuitState.HALF_OPEN
                self.half_open_count = 0
            else:
                raise CircuitOpenError("Circuit is open — failing fast")

        if self.state == CircuitState.HALF_OPEN:
            if self.half_open_count >= self.half_open_max:
                raise CircuitOpenError("Half-open limit reached")

        try:
            result = func(*args, **kwargs)
            self._on_success()
            return result
        except Exception as e:
            self._on_failure()
            raise

    def _on_success(self):
        if self.state == CircuitState.HALF_OPEN:
            self.state = CircuitState.CLOSED
        self.failure_count = 0

    def _on_failure(self):
        self.failure_count += 1
        self.last_failure_time = time.time()

        if self.state == CircuitState.HALF_OPEN:
            self.state = CircuitState.OPEN
        elif self.failure_count >= self.failure_threshold:
            self.state = CircuitState.OPEN

class CircuitOpenError(Exception):
    pass

# Usage
breaker = CircuitBreaker(failure_threshold=5, recovery_timeout=30)

def call_payment_service(order):
    return breaker.call(
        requests.post,
        "http://payment-service/charge",
        json=order,
        timeout=5
    )
```

### Retry with Exponential Backoff and Jitter

Transient failures (network blips, temporary overload) are resolved by retrying. But naive retries create thundering herds. Exponential backoff spaces retries out, and jitter adds randomness to prevent synchronized retries.

```python
import time
import random

def retry_with_backoff(func, max_retries=5, base_delay=1, max_delay=60):
    """Retry with exponential backoff and full jitter."""
    for attempt in range(max_retries):
        try:
            return func()
        except TransientError as e:
            if attempt == max_retries - 1:
                raise  # Last attempt failed

            # Exponential backoff: 1s, 2s, 4s, 8s, 16s...
            delay = min(base_delay * (2 ** attempt), max_delay)

            # Full jitter: random delay between 0 and calculated delay
            jittered_delay = random.uniform(0, delay)

            print(f"Attempt {attempt + 1} failed: {e}. Retrying in {jittered_delay:.2f}s")
            time.sleep(jittered_delay)

# Usage
response = retry_with_backoff(
    lambda: requests.get("http://api/users/123", timeout=5)
)
```

**Backoff strategies:**

```
Fixed:       1s, 1s, 1s, 1s, 1s          (constant delay)
Linear:      1s, 2s, 3s, 4s, 5s          (increasing delay)
Exponential: 1s, 2s, 4s, 8s, 16s         (doubling delay)
Exp+Jitter:  0.3s, 1.7s, 2.1s, 6.4s, 11.2s  (doubling + random)
```

**Retryable vs non-retryable errors:**

```python
RETRYABLE_ERRORS = {
    502,  # Bad Gateway
    503,  # Service Unavailable
    504,  # Gateway Timeout
    408,  # Request Timeout
    429,  # Too Many Requests
}

def should_retry(status_code):
    return status_code in RETRYABLE_ERRORS

# 400 Bad Request, 401 Unauthorized, 404 Not Found — do NOT retry
```

### Bulkhead Pattern

Isolate components so that a failure in one does not cascade to others. Named after the watertight compartments in ship hulls.

```python
import threading
from concurrent.futures import ThreadPoolExecutor

class Bulkhead:
    def __init__(self, max_concurrent, max_queue=10):
        self.semaphore = threading.Semaphore(max_concurrent)
        self.max_queue = max_queue
        self.queue_size = 0
        self.lock = threading.Lock()

    def execute(self, func, *args, **kwargs):
        with self.lock:
            if self.queue_size >= self.max_queue:
                raise BulkheadFullError("Bulkhead queue is full")
            self.queue_size += 1

        try:
            self.semaphore.acquire()
            return func(*args, **kwargs)
        finally:
            self.semaphore.release()
            with self.lock:
                self.queue_size -= 1

class BulkheadFullError(Exception):
    pass

# Separate bulkheads for different services
payment_bulkhead = Bulkhead(max_concurrent=10, max_queue=20)
inventory_bulkhead = Bulkhead(max_concurrent=50, max_queue=100)

# If payment service is slow, it only blocks payment_bulkhead
# inventory_bulkhead continues unaffected
```

---

## The Production Way

### Combining Patterns

Real systems combine multiple patterns. Here is a production-grade HTTP client:

```python
import requests
import time
import random
import threading

class ResilientHttpClient:
    def __init__(self, base_url, service_name):
        self.base_url = base_url
        self.service_name = service_name
        self.circuit_breaker = CircuitBreaker(
            failure_threshold=5,
            recovery_timeout=30
        )
        self.bulkhead = Bulkhead(max_concurrent=20, max_queue=50)

    def request(self, method, path, **kwargs):
        kwargs.setdefault("timeout", (3, 10))  # connect, read timeout

        def make_request():
            return self.circuit_breaker.call(
                requests.request,
                method,
                f"{self.base_url}{path}",
                **kwargs
            )

        try:
            return self.bulkhead.execute(
                lambda: retry_with_backoff(make_request, max_retries=3)
            )
        except CircuitOpenError:
            return self._fallback(method, path, **kwargs)
        except BulkheadFullError:
            return self._fallback(method, path, **kwargs)

    def _fallback(self, method, path, **kwargs):
        """Graceful degradation when the service is unavailable."""
        if method == "GET" and "/products" in path:
            return CachedResponse(cache.get(f"products:{path}"))
        raise ServiceUnavailableError(f"{self.service_name} is unavailable")

class CachedResponse:
    def __init__(self, data):
        self.data = data
        self.status_code = 200
```

### Observability for Distributed Systems

```python
# Distributed tracing with OpenTelemetry
from opentelemetry import trace
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.exporter.jaeger.thrift import JaegerExporter
from opentelemetry.sdk.trace.export import BatchSpanProcessor

# Setup
provider = TracerProvider()
exporter = JaegerExporter(agent_host_name="jaeger", agent_port=6831)
provider.add_span_processor(BatchSpanProcessor(exporter))
trace.set_tracer_provider(provider)
tracer = trace.get_tracer(__name__)

# Instrumented service call
def process_order(order):
    with tracer.start_as_current_span("process_order") as span:
        span.set_attribute("order.id", order.id)
        span.set_attribute("order.total", order.total)

        with tracer.start_as_current_span("charge_payment"):
            payment = payment_service.charge(order)

        with tracer.start_as_current_span("reserve_inventory"):
            inventory = inventory_service.reserve(order)

        with tracer.start_as_current_span("send_notification"):
            notification_service.notify(order)
```

---

## Hands-On Lab

### Exercise 1: CAP Theorem in Practice

```bash
# Start a 3-node Cassandra cluster (AP system)
docker run -d --name cass1 -e CASSANDRA_SEEDS=cass1 cassandra:4
docker run -d --name cass2 -e CASSANDRA_SEEDS=cass1 cassandra:4
docker run -d --name cass3 -e CASSANDRA_SEEDS=cass1 cassandra:4

# Create a keyspace with replication factor 3
docker exec -it cass1 cqlsh
# CREATE KEYSPACE demo WITH replication = {'class': 'SimpleStrategy', 'replication_factor': 3};
# CREATE TABLE demo.kv (key text PRIMARY KEY, value text);
# INSERT INTO demo.kv (key, value) VALUES ('test', 'hello');

# Simulate a network partition
docker network disconnect bridge cass3

# Write to the partitioned node (still accepts writes — AP behavior)
docker exec -it cass3 cqlsh -e "INSERT INTO demo.kv (key, value) VALUES ('test', 'updated_on_3');"

# Read from the majority partition
docker exec -it cass1 cqlsh -e "SELECT * FROM demo.kv WHERE key='test';"
# May return old value — eventual consistency

# Heal partition
docker network connect bridge cass3
sleep 10
docker exec -it cass1 cqlsh -e "SELECT * FROM demo.kv WHERE key='test';"
# Now returns updated value — consistency achieved
```

### Exercise 2: Circuit Breaker

```python
# circuit_breaker_lab.py
import threading
import time

# Simulate a failing service
class FailingService:
    def __init__(self):
        self.call_count = 0
        self.should_fail = True

    def call(self):
        self.call_count += 1
        if self.should_fail:
            raise ConnectionError("Service unavailable")
        return "success"

# Test circuit breaker
service = FailingService()
breaker = CircuitBreaker(failure_threshold=3, recovery_timeout=5)

print("=== Phase 1: Trip the circuit breaker ===")
for i in range(5):
    try:
        breaker.call(service.call)
    except (ConnectionError, CircuitOpenError) as e:
        print(f"  Call {i+1}: {type(e).__name__}: {e}")
    time.sleep(0.5)

print(f"\n  Circuit state: {breaker.state.value}")

print("\n=== Phase 2: Wait for recovery timeout ===")
time.sleep(5)

print("\n=== Phase 3: Circuit transitions to half-open ===")
service.should_fail = False
for i in range(3):
    try:
        result = breaker.call(service.call)
        print(f"  Call {i+1}: Success: {result}")
    except (ConnectionError, CircuitOpenError) as e:
        print(f"  Call {i+1}: {type(e).__name__}: {e}")

print(f"\n  Circuit state: {breaker.state.value}")
```

### Exercise 3: Exponential Backoff

```python
# backoff_lab.py
import time
import random

def unreliable_api():
    """Fails 70% of the time."""
    if random.random() < 0.7:
        raise ConnectionError("Connection reset")
    return {"status": "ok"}

def fixed_backoff(func, retries=5, delay=1):
    for i in range(retries):
        try:
            return func()
        except ConnectionError:
            if i < retries - 1:
                time.sleep(delay)
    raise Exception("All retries failed")

def exponential_backoff(func, retries=5, base=1):
    for i in range(retries):
        try:
            return func()
        except ConnectionError:
            if i < retries - 1:
                delay = min(base * (2 ** i), 30)
                jitter = random.uniform(0, delay)
                time.sleep(jitter)
    raise Exception("All retries failed")

# Run both and compare
start = time.time()
try:
    result = fixed_backoff(unreliable_api)
    print(f"Fixed backoff: {time.time() - start:.2f}s - {result}")
except Exception as e:
    print(f"Fixed backoff: {time.time() - start:.2f}s - FAILED")

start = time.time()
try:
    result = exponential_backoff(unreliable_api)
    print(f"Exponential backoff: {time.time() - start:.2f}s - {result}")
except Exception as e:
    print(f"Exponential backoff: {time.time() - start:.2f}s - FAILED")
```

### Exercise 4: Raft Consensus Visualization

```bash
# Run a 3-node etcd cluster and observe Raft in action
docker run -d --name etcd1 --net host \
  quay.io/coreos/etcd:v3.5.9 \
  etcd --name etcd1 \
  --initial-advertise-peer-urls http://127.0.0.1:2380 \
  --listen-peer-urls http://127.0.0.1:2380 \
  --listen-client-urls http://127.0.0.1:2379 \
  --advertise-client-urls http://127.0.0.1:2379 \
  --initial-cluster etcd1=http://127.0.0.1:2380

# Write and read
etcdctl put greeting "hello distributed world"
etcdctl get greeting

# Observe Raft logs
docker exec etcd1 etcdctl endpoint status --write-out=table
docker exec etcd1 etcdctl endpoint health

# View cluster membership
docker exec etcd1 etcdctl member list --write-out=table
```

---

## Limitation

The distributed systems patterns in this lesson apply to systems running within a single cloud provider or datacenter. When you need to run across multiple cloud providers — AWS for compute, GCP for machine learning, Azure for enterprise integration — or maintain hybrid deployments spanning on-premises and cloud, you face a new set of challenges: cross-cloud networking, data gravity, inconsistent APIs, and avoiding vendor lock-in while leveraging the best features of each provider.

---

## Next Topic

[75 - Multi-Cloud and Hybrid](../75-multi-cloud-and-hybrid/README.md) — Design systems that span multiple cloud providers and on-premises infrastructure without vendor lock-in.
