# Solution 05: Resilient Microservice Architecture

## Part A: Resilience Strategy per Service

| Service | Circuit Breaker | Retry | Bulkhead | Fallback |
|---------|----------------|-------|----------|----------|
| **gateway** | Not needed (it is the entry point) | Retry upstream 502/503/504 once with 1s delay | Max 200 concurrent requests, queue 50 | Return 503 with retry-after header |
| **ride** | threshold=3, timeout=15s (depends on driver and payment) | 3 retries, exponential 1-8s, retry 502/503/504/429 | Max 50 concurrent, queue 100 | Return "no drivers available" with ETA |
| **payment** | threshold=3, timeout=60s (external gateway, high latency variance) | 2 retries, exponential 2-16s, retry 502/503/504 only | Max 20 concurrent, queue 30 | Queue payment for async processing |
| **driver** | threshold=5, timeout=10s (high traffic, must recover fast) | No retry (location updates are frequent, next update replaces) | Max 100 concurrent, queue 200 | Return last known location from cache |
| **notification** | threshold=10, timeout=30s (best-effort, lenient) | 3 retries, exponential 1-30s, retry all errors | Max 30 concurrent, queue 500 | Drop notification (non-critical) |

### Why These Settings

- **Payment** has the strictest circuit breaker (3 failures) because the external gateway is unreliable and financial impact is high. The long timeout (60s) accounts for gateway latency spikes.
- **Driver** has the most aggressive bulkhead (100 concurrent) because surge pricing generates high traffic. No retries because location updates arrive every few seconds -- a lost update is replaced by the next one.
- **Notification** is the most lenient because it is best-effort. A dropped notification is acceptable. The large queue (500) buffers spikes.

### Common Mistakes to Avoid

- Using the same settings for all services. Each service has different failure modes and business criticality.
- Setting circuit breaker thresholds too low (1-2). Transient failures would open the circuit unnecessarily.
- Not setting bulkheads. Without them, a slow service consumes all threads and cascades to other services.

---

## Part B: Ride Request Flow

```python
import uuid
import time


class RideRequestHandler:
    def __init__(self, ride_service, driver_service, payment_service,
                 notification_service):
        self.ride_service = ride_service
        self.driver_service = driver_service
        self.payment_service = payment_service
        self.notification_service = notification_service

        # Circuit breakers per service
        self.driver_breaker = CircuitBreaker(failure_threshold=5,
                                              recovery_timeout=10)
        self.payment_breaker = CircuitBreaker(failure_threshold=3,
                                               recovery_timeout=60)

        # Bulkheads per service
        self.driver_bulkhead = Bulkhead(max_concurrent=100, max_queue=200)
        self.payment_bulkhead = Bulkhead(max_concurrent=20, max_queue=30)

    def request_ride(self, rider_id, pickup, destination):
        """Handle a ride request with full resilience."""
        ride_id = str(uuid.uuid4())
        context = {"ride_id": ride_id, "rider_id": rider_id}

        try:
            # Step 1: Create ride request
            ride = self.ride_service.create(ride_id, rider_id, pickup,
                                            destination)
            context["ride"] = ride

            # Step 2: Find available driver (circuit breaker + bulkhead)
            driver = self.driver_breaker.call(
                self.driver_bulkhead.execute,
                self.driver_service.find_available, pickup
            )
            context["driver"] = driver

            # Step 3: Confirm match
            self.ride_service.confirm_match(ride_id, driver["id"])

            # Step 4: Wait for ride completion (in production, this is async)
            # For this example, we simulate it
            ride_cost = self._wait_for_ride_completion(ride_id)
            context["cost"] = ride_cost

            # Step 5: Process payment (circuit breaker + bulkhead + idempotency)
            idempotency_key = str(uuid.uuid4())
            payment = self.payment_breaker.call(
                self.payment_bulkhead.execute,
                self.payment_service.charge,
                rider_id, ride_cost, idempotency_key
            )
            context["payment"] = payment

            # Step 6: Send notification (best-effort, no circuit breaker)
            try:
                self.notification_service.send(
                    rider_id,
                    f"Ride {ride_id} complete. Charged ${ride_cost}."
                )
            except Exception:
                pass  # Non-critical, ignore failure

            return {"status": "completed", "ride_id": ride_id,
                    "cost": ride_cost}

        except CircuitOpenError as e:
            # Fallback: queue the request
            return self._fallback_ride_request(context, str(e))

        except BulkheadFullError as e:
            # Fallback: return "try again later"
            return {"status": "busy", "message": "High demand, try again",
                    "retry_after": 10}

        except Exception as e:
            # Compensation: undo completed steps
            self._compensate(context)
            return {"status": "failed", "error": str(e)}

    def _wait_for_ride_completion(self, ride_id):
        """Simulate ride completion. In production, this is event-driven."""
        time.sleep(1)  # Simulated
        return 25.50

    def _fallback_ride_request(self, context, error):
        """Queue the ride request for later processing."""
        self.ride_service.queue(context["ride_id"])
        return {"status": "queued", "ride_id": context["ride_id"],
                "message": "Ride queued due to service issues"}

    def _compensate(self, context):
        """Undo completed steps in reverse order."""
        if "payment" in context:
            try:
                self.payment_service.refund(context["ride_id"])
            except Exception:
                pass  # Log and continue

        if "driver" in context:
            try:
                self.driver_service.release(context["driver"]["id"])
            except Exception:
                pass

        if "ride" in context:
            try:
                self.ride_service.cancel(context["ride_id"])
            except Exception:
                pass
```

### Why This Works

- **Layered resilience:** Circuit breakers prevent cascading failures. Bulkheads limit concurrent requests. Retries handle transient errors. Fallbacks provide graceful degradation.
- **Idempotent payments:** The idempotency key ensures that retries do not double-charge.
- **Compensation on failure:** If any step fails, previous steps are undone in reverse order.
- **Non-critical notification:** Notification failure does not trigger compensation. The ride is complete regardless.

### Common Mistakes to Avoid

- Applying the same resilience settings to all services. Each service has different characteristics.
- Not compensating on failure. A paid ride that is not matched to a driver leaves the customer charged with no service.
- Making notification critical. If notification failure triggers compensation, a transient email server issue would cancel completed rides.

---

## Part C: Failure Injection Testing

### Scenario 1: Kill Payment Service

**Injection:** Stop the payment service container. `docker stop payment-service`

**Expected behavior:**
1. First 3 requests to payment fail (circuit breaker threshold).
2. Circuit opens. Subsequent requests fail fast (< 1ms).
3. Rides continue to be requested and matched (payment is async).
4. Payment requests are queued.
5. After 60 seconds, circuit half-opens. If payment service is restored, payments process.
6. If payment service is still down, circuit re-opens.

**Verification:**
- Monitor circuit breaker state transitions in logs.
- Verify ride request latency does not increase (fails fast, not timeout).
- Verify queued payments process after recovery.

### Scenario 2: Add 10s Latency to Driver Service

**Injection:** Use a proxy or `tc` to add 10s delay. `tc qdisc add dev eth0 root netem delay 10000ms`

**Expected behavior:**
1. Driver service calls take 10s instead of 100ms.
2. Bulkhead fills up (100 concurrent requests, each holding a slot for 10s).
3. New requests are rejected immediately (bulkhead full).
4. Users get "High demand, try again" response.
5. After removing latency, bulkhead drains and normal operation resumes.

**Verification:**
- Monitor bulkhead utilization metrics.
- Verify response time for rejected requests is < 1ms (not 10s).
- Verify recovery after latency is removed.

### Scenario 3: Drop Notification Messages

**Injection:** Block notification service port. `iptables -A OUTPUT -p tcp --dport notification-port -j DROP`

**Expected behavior:**
1. Notification calls timeout (no connection refused, just hangs).
2. Notification has lenient circuit breaker (threshold=10, timeout=30s).
3. After 10 failures, circuit opens. Notifications are silently dropped.
4. Rides complete normally. Customers do not receive confirmation emails.
5. After restoring port, notifications resume.

**Verification:**
- Verify rides complete without error.
- Verify no customer-facing errors.
- Verify notifications resume after recovery.

### Scenario 4: Network Partition Between Ride and Payment

**Injection:** Block traffic between ride and payment services. `iptables -A INPUT -s ride-service-ip -p tcp --dport payment-port -j DROP`

**Expected behavior:**
1. Payment calls fail immediately (connection refused or timeout).
2. Circuit breaker opens after 3 failures.
3. Rides that reach the payment step fail and trigger compensation.
4. Compensation: release driver, cancel ride.
5. Users get "Payment service unavailable" error.
6. After partition heals, normal operation resumes.

**Verification:**
- Verify compensation is triggered (driver released, ride cancelled).
- Verify no double-charges (idempotency keys).
- Verify recovery after partition heals.

### Common Mistakes to Avoid

- Only testing the happy path. Resilience patterns are useless if they are not tested.
- Not testing recovery. A system that never recovers from a failure is not resilient.
- Testing one service at a time. Real failures often involve multiple services (e.g., payment down causes ride queue to fill, which causes bulkhead to overflow).

---

## Key Takeaway

Resilience is a combination of patterns applied at different layers. Circuit breakers prevent cascading failures by failing fast. Retries handle transient errors with exponential backoff. Bulkheads isolate failures so one slow service does not affect others. Idempotency makes retries safe. Sagas coordinate multi-service transactions with compensation. The art is in tuning each pattern for each service's specific failure modes and business criticality.
