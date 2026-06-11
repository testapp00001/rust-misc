# Exercise 05: Resilient Microservice Architecture

**Type:** Integration
**Time:** 60 minutes
**Difficulty:** Hard

## Objective

You will design a complete resilient architecture for a microservices application by applying all the distributed systems patterns from this module: circuit breakers, retries with backoff, bulkheads, idempotency, sagas, and health monitoring.

## Scenario

You are building a ride-sharing application with the following services:

- **gateway** -- API gateway, receives all client requests.
- **ride** -- Manages ride requests and matching.
- **payment** -- Processes payments.
- **driver** -- Tracks driver locations and availability.
- **notification** -- Sends push notifications to riders and drivers.

The system must handle: payment gateway outages, driver service overload during surge pricing, notification delivery failures, and network partitions between services.

## Tasks

### Part A: Resilience Strategy per Service

For each service, specify which resilience patterns to apply and why. Include:

1. Circuit breaker settings (threshold, timeout).
2. Retry strategy (max retries, backoff type, retryable errors).
3. Bulkhead settings (max concurrent, max queue).
4. Fallback behavior when the service is unavailable.

Create a table with columns: Service, Circuit Breaker, Retry, Bulkhead, Fallback.

<details>
<summary>Hint</summary>
Payment needs the strictest circuit breaker (external dependency, financial impact). Driver needs bulkheads (high traffic during surge). Notification can be the most lenient (best-effort delivery).
</details>

### Part B: Ride Request Flow

Design the complete flow for a ride request, including:

1. The sequence of service calls.
2. Where each resilience pattern is applied.
3. The saga that coordinates the ride lifecycle (request -> match -> ride -> payment -> complete).
4. Compensating actions if any step fails.

Write the pseudocode or Python code for the ride request handler.

<details>
<summary>Hint</summary>
The ride request saga: (1) Create ride request, (2) Find available driver, (3) Confirm match, (4) Complete ride, (5) Process payment, (6) Send notification. Compensation: cancel ride, release driver, refund payment.
</details>

### Part C: Failure Injection Testing

Design a chaos testing plan that validates the resilience of your architecture. For each test, specify:

1. What to inject (kill a service, add latency, drop messages).
2. Expected behavior of the system.
3. How to verify the system recovered correctly.

Include at least 4 failure injection scenarios.

<details>
<summary>Hint</summary>
Scenarios: (1) Kill payment service -- circuit breaker opens, rides continue without payment processing. (2) Add 10s latency to driver service -- bulkhead fills, requests are rejected fast. (3) Drop notification messages -- notifications are lost but rides complete. (4) Network partition between ride and payment -- saga compensates.
</details>

## Success Criteria

- [ ] Each service has a complete resilience strategy (circuit breaker, retry, bulkhead, fallback).
- [ ] The ride request flow includes all resilience patterns with specific parameters.
- [ ] The saga coordinates the ride lifecycle with compensating actions.
- [ ] At least 4 failure injection scenarios are defined with expected behavior and verification.
- [ ] You can explain why different services need different resilience settings.

## What You Should Understand After This Exercise

Resilience is not a single pattern -- it is a combination of patterns applied at different layers. Circuit breakers prevent cascading failures. Retries handle transient errors. Bulkheads isolate failures. Idempotency makes retries safe. Sagas coordinate multi-service transactions. The art is in choosing the right combination and tuning the parameters for each service's specific failure modes and business criticality.
