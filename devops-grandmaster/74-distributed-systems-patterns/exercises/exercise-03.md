# Exercise 03: Circuit Breaker Implementation

**Type:** Independent
**Time:** 40 minutes
**Difficulty:** Medium

## Objective

You will implement a circuit breaker that prevents an application from repeatedly calling a failing service. This exercise practices the three-state circuit breaker pattern (Closed, Open, Half-Open) and its integration with a real HTTP client.

## Scenario

Your application calls an external payment gateway. When the gateway is down, every request waits for the timeout (5 seconds), then fails. With 100 requests per second, this means 500 wasted seconds of thread time per second. The circuit breaker should detect the failure pattern and start failing fast.

## Tasks

### Part A: Circuit Breaker State Machine

Implement a `CircuitBreaker` class with the following states:

- **CLOSED:** Normal operation. Requests pass through. Track failures.
- **OPEN:** Fail fast. Reject all requests immediately. After a timeout, transition to HALF_OPEN.
- **HALF_OPEN:** Allow a limited number of test requests. If they succeed, close the circuit. If they fail, reopen it.

The class should support:

- `failure_threshold` -- Number of consecutive failures before opening (default: 5).
- `recovery_timeout` -- Seconds to wait before trying half-open (default: 30).
- `half_open_max` -- Maximum test requests in half-open state (default: 3).
- A `call(func, *args, **kwargs)` method that wraps any callable.

<details>
<summary>Hint</summary>
Use an enum for states. Track `failure_count`, `last_failure_time`, and `half_open_count`. In the `call` method, check state before calling the function and update state after success or failure.
</details>

### Part B: Integration with HTTP Client

Write a function `call_payment_gateway` that uses the circuit breaker to wrap HTTP calls to the payment gateway. Include:

1. The circuit breaker wrapping the HTTP call.
2. A fallback response when the circuit is open (return an error to the user, do not wait).
3. Logging of state transitions (CLOSED -> OPEN, OPEN -> HALF_OPEN, etc.).

<details>
<summary>Hint</summary>
The fallback should return a meaningful error (e.g., "Payment service temporarily unavailable"). Log state transitions by overriding the state setter or checking state before and after `call()`.
</details>

### Part C: Testing the Circuit Breaker

Write a test scenario that:

1. Sends 10 requests, 6 of which fail (exceeds threshold of 5).
2. Verifies the circuit opens and subsequent requests fail fast.
3. Waits for the recovery timeout.
4. Verifies the circuit transitions to half-open and allows test requests.
5. Verifies that a successful test request closes the circuit.

<details>
<summary>Hint</summary>
Use `time.sleep(recovery_timeout + 1)` to simulate waiting. Use a mock function that fails N times then succeeds to control the test.
</details>

## Success Criteria

- [ ] Circuit breaker correctly transitions between CLOSED, OPEN, and HALF_OPEN.
- [ ] The `call` method wraps any callable with circuit breaker logic.
- [ ] State transitions are logged.
- [ ] The test scenario demonstrates all three states and transitions.
- [ ] Open circuit fails fast (no waiting for timeout).

## What You Should Understand After This Exercise

The circuit breaker is a state machine that prevents cascading failures. In CLOSED state, it monitors failures. When failures exceed the threshold, it OPENs and fails fast, protecting both the caller (no wasted timeout) and the callee (no load during recovery). After a timeout, it HALF_OPENs to test recovery. This pattern is essential for any service that depends on an unreliable external dependency.
