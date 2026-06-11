# Solution 03: Circuit Breaker Implementation

## Part A: Circuit Breaker State Machine

```python
import time
from enum import Enum


class CircuitState(Enum):
    CLOSED = "closed"
    OPEN = "open"
    HALF_OPEN = "half_open"


class CircuitOpenError(Exception):
    """Raised when the circuit breaker is open."""
    pass


class CircuitBreaker:
    def __init__(self, failure_threshold=5, recovery_timeout=30,
                 half_open_max=3):
        self.failure_threshold = failure_threshold
        self.recovery_timeout = recovery_timeout
        self.half_open_max = half_open_max

        self._state = CircuitState.CLOSED
        self._failure_count = 0
        self._last_failure_time = None
        self._half_open_count = 0

    @property
    def state(self):
        # Check if open circuit should transition to half-open
        if self._state == CircuitState.OPEN:
            if (self._last_failure_time and
                    time.time() - self._last_failure_time > self.recovery_timeout):
                self._transition_to(CircuitState.HALF_OPEN)
        return self._state

    def call(self, func, *args, **kwargs):
        """Execute func with circuit breaker protection."""
        current_state = self.state  # triggers potential OPEN -> HALF_OPEN

        if current_state == CircuitState.OPEN:
            raise CircuitOpenError(
                f"Circuit is open. Retry after {self.recovery_timeout}s. "
                f"Last failure: {self._last_failure_time}"
            )

        if current_state == CircuitState.HALF_OPEN:
            if self._half_open_count >= self.half_open_max:
                self._transition_to(CircuitState.OPEN)
                raise CircuitOpenError("Half-open limit reached")

        try:
            result = func(*args, **kwargs)
            self._on_success()
            return result
        except Exception as e:
            self._on_failure()
            raise

    def _on_success(self):
        if self._state == CircuitState.HALF_OPEN:
            self._transition_to(CircuitState.CLOSED)
        self._failure_count = 0
        self._half_open_count = 0

    def _on_failure(self):
        self._failure_count += 1
        self._last_failure_time = time.time()

        if self._state == CircuitState.HALF_OPEN:
            self._transition_to(CircuitState.OPEN)
        elif (self._state == CircuitState.CLOSED and
              self._failure_count >= self.failure_threshold):
            self._transition_to(CircuitState.OPEN)

    def _transition_to(self, new_state):
        old_state = self._state
        self._state = new_state
        print(f"Circuit breaker: {old_state.value} -> {new_state.value}")

        if new_state == CircuitState.HALF_OPEN:
            self._half_open_count = 0
```

### Why This Works

- **State property with lazy transition:** The `state` property checks if the open circuit should transition to half-open. This avoids needing a separate timer thread.
- **Failure counting:** In CLOSED state, failures are counted. When the count reaches the threshold, the circuit opens.
- **Half-open limiting:** In HALF_OPEN state, a limited number of test requests are allowed. If they succeed, the circuit closes. If they fail, the circuit reopens.
- **State transition logging:** Every transition is logged, making it easy to monitor circuit breaker behavior in production.

### Common Mistakes to Avoid

- Not resetting the failure count on success. The circuit would open after `threshold` failures across the entire lifetime, not consecutive failures.
- Using a fixed timer for OPEN -> HALF_OPEN transition. The lazy check (on next call) is simpler and avoids threading complexity.
- Not limiting half-open requests. Without the limit, a burst of requests during half-open could overwhelm the recovering service.

---

## Part B: Integration with HTTP Client

```python
import requests
import logging

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


class PaymentGatewayClient:
    def __init__(self):
        self.breaker = CircuitBreaker(
            failure_threshold=5,
            recovery_timeout=30,
            half_open_max=3
        )
        self.base_url = "https://payment-gateway.example.com"

    def charge(self, customer_id, amount):
        """Charge a customer with circuit breaker protection."""
        try:
            result = self.breaker.call(
                self._make_request,
                "POST",
                "/charge",
                json={"customer_id": customer_id, "amount": amount}
            )
            return result
        except CircuitOpenError as e:
            logger.warning(f"Circuit open for payment gateway: {e}")
            return self._fallback_charge(customer_id, amount)
        except requests.RequestException as e:
            logger.error(f"Payment gateway error: {e}")
            raise

    def _make_request(self, method, path, **kwargs):
        """Make an HTTP request to the payment gateway."""
        url = f"{self.base_url}{path}"
        response = requests.request(method, url, timeout=5, **kwargs)
        response.raise_for_status()
        return response.json()

    def _fallback_charge(self, customer_id, amount):
        """Fallback when circuit is open."""
        logger.info(f"Queuing charge for {customer_id}: ${amount}")
        return {
            "status": "queued",
            "message": "Payment will be processed when service recovers",
            "customer_id": customer_id,
            "amount": amount
        }
```

### Why This Works

- **Fallback behavior:** When the circuit is open, the request is queued instead of failing immediately. The user gets a response (queued) rather than a timeout or error.
- **Separation of concerns:** The circuit breaker is a wrapper around the HTTP call. The payment logic does not know about circuit breaking.
- **Logging:** State transitions are logged automatically by the circuit breaker. HTTP errors are logged by the client.

### Common Mistakes to Avoid

- Not having a fallback. An open circuit that returns an error is better than a timeout, but a fallback (queue, cached response, default value) is even better.
- Retrying inside the circuit breaker. The circuit breaker counts failures; retries should be outside the circuit breaker (or the circuit breaker would count each retry as a separate failure).
- Using the same circuit breaker for different operations. Each external dependency should have its own circuit breaker.

---

## Part C: Testing the Circuit Breaker

```python
import time


def test_circuit_breaker():
    breaker = CircuitBreaker(
        failure_threshold=5,
        recovery_timeout=2,  # short timeout for testing
        half_open_max=3
    )

    # 1. Initial state is CLOSED
    assert breaker.state == CircuitState.CLOSED

    # 2. Fail 6 times (exceeds threshold of 5)
    def failing_func():
        raise Exception("Service unavailable")

    for i in range(6):
        try:
            breaker.call(failing_func)
        except Exception:
            pass

    # 3. Circuit should be OPEN
    assert breaker.state == CircuitState.OPEN
    print("PASS: Circuit opens after 5 failures")

    # 4. Calls should fail fast (no waiting)
    start = time.time()
    try:
        breaker.call(failing_func)
    except CircuitOpenError:
        elapsed = time.time() - start
        assert elapsed < 0.1, f"Should fail fast, took {elapsed}s"
    print("PASS: Open circuit fails fast")

    # 5. Wait for recovery timeout
    time.sleep(3)

    # 6. Circuit should transition to HALF_OPEN
    assert breaker.state == CircuitState.HALF_OPEN
    print("PASS: Circuit transitions to half-open after timeout")

    # 7. Successful call should close the circuit
    def success_func():
        return "ok"

    result = breaker.call(success_func)
    assert result == "ok"
    assert breaker.state == CircuitState.CLOSED
    print("PASS: Successful call closes the circuit")

    # 8. Test half-open failure (re-opens circuit)
    breaker2 = CircuitBreaker(failure_threshold=2, recovery_timeout=1)
    for _ in range(3):
        try:
            breaker2.call(failing_func)
        except Exception:
            pass
    assert breaker2.state == CircuitState.OPEN

    time.sleep(2)
    assert breaker2.state == CircuitState.HALF_OPEN

    try:
        breaker2.call(failing_func)
    except Exception:
        pass

    assert breaker2.state == CircuitState.OPEN
    print("PASS: Failed half-open call re-opens circuit")

    print("\nAll tests passed!")


test_circuit_breaker()
```

### Common Mistakes to Avoid

- Not waiting long enough for the recovery timeout in tests. Use a short timeout (1-2 seconds) for testing.
- Testing only the happy path. The most important tests are: circuit opens on failure threshold, fails fast when open, transitions to half-open after timeout, and re-opens on half-open failure.

---

## Key Takeaway

The circuit breaker is a state machine that prevents cascading failures by monitoring the failure rate of an external dependency. In CLOSED state, it passes requests through and counts failures. When failures exceed the threshold, it OPENs and fails fast. After a timeout, it HALF_OPENs to test recovery. This pattern protects both the caller (no wasted timeouts) and the callee (no load during recovery). Integrate it with a fallback mechanism for the best user experience.
