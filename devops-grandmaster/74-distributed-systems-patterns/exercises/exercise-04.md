# Exercise 04: Saga Pattern for Distributed Transactions

**Type:** Challenge
**Time:** 45 minutes
**Difficulty:** Medium-Hard

## Objective

You will implement the Saga pattern for a distributed transaction that spans multiple services. This exercise practices both choreography and orchestration approaches, compensating transactions, and failure handling.

## Scenario

An e-commerce order requires four steps:

1. **Order Service:** Create the order (status: PENDING).
2. **Payment Service:** Charge the customer.
3. **Inventory Service:** Reserve the items.
4. **Notification Service:** Send confirmation email.

If any step fails, all previous steps must be compensated (rolled back). For example, if inventory reservation fails, the payment must be refunded and the order must be cancelled.

## Tasks

### Part A: Choreography Saga

Implement the choreography approach where each service publishes events and listens for events from others. Write the event handlers for each service:

1. Order Service: handles `OrderCreated`, `PaymentCompleted`, `PaymentFailed`, `InventoryReserved`, `InventoryFailed`.
2. Payment Service: handles `OrderCreated`, `RefundPayment`.
3. Inventory Service: handles `PaymentCompleted`, `PaymentFailed`.
4. Notification Service: handles `InventoryReserved`.

Use an event bus abstraction (`event_bus.publish(event_type, data)` and `event_bus.subscribe(event_type, handler)`).

<details>
<summary>Hint</summary>
Each service publishes events after completing its local transaction. The next service in the chain subscribes to the previous service's success event. Failure events trigger compensating actions in upstream services.
</details>

### Part B: Orchestration Saga

Implement the orchestration approach with a central `OrderSagaOrchestrator` that coordinates all steps. The orchestrator should:

1. Define the sequence of steps with forward actions and compensating actions.
2. Execute steps in order.
3. If a step fails, execute compensating actions in reverse order.
4. Track which steps completed to know which compensations to run.

<details>
<summary>Hint</summary>
Use a list of `(action, compensate)` tuples. Track the index of the last successful step. On failure, iterate backwards from that index and call each compensate function.
</details>

### Part C: Failure Scenario Analysis

For each scenario, describe what happens in both the choreography and orchestration approaches:

1. Payment succeeds but inventory reservation fails.
2. Payment succeeds, inventory succeeds, but notification fails.
3. The event bus loses the `PaymentCompleted` message.
4. The orchestrator crashes after payment but before inventory.

<details>
<summary>Hint</summary>
In choreography, think about what each service does when it receives a failure event. In orchestration, think about what the orchestrator's rollback logic does. For lost messages, think about idempotency and dead letter queues.
</details>

## Success Criteria

- [ ] Choreography saga correctly chains events and handles failures.
- [ ] Orchestration saga correctly sequences steps and runs compensations in reverse.
- [ ] All four failure scenarios are analyzed for both approaches.
- [ ] Compensating transactions undo the effects of completed steps.
- [ ] You can articulate the trade-off between choreography and orchestration.

## What You Should Understand After This Exercise

The Saga pattern replaces distributed transactions (two-phase commit) with a sequence of local transactions and compensating actions. Choreography distributes the logic across services (more decoupled, harder to debug). Orchestration centralizes it (easier to understand, single point of failure). Both approaches require compensating transactions that undo previous steps. The key insight is that you cannot roll back a distributed transaction atomically -- you can only compensate for it.
