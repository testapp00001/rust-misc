# Solution 04: Saga Pattern for Distributed Transactions

## Part A: Choreography Saga

```python
class EventBus:
    def __init__(self):
        self._subscribers = {}

    def subscribe(self, event_type, handler):
        if event_type not in self._subscribers:
            self._subscribers[event_type] = []
        self._subscribers[event_type].append(handler)

    def publish(self, event_type, data):
        print(f"Event: {event_type} | Data: {data}")
        for handler in self._subscribers.get(event_type, []):
            handler(data)


# Initialize event bus
bus = EventBus()


# --- Order Service ---
class OrderService:
    def __init__(self, db):
        self.db = db

    def create_order(self, order_data):
        order = {"id": order_data["id"], "status": "PENDING",
                 "amount": order_data["amount"]}
        self.db.save("orders", order)
        bus.publish("OrderCreated", {
            "order_id": order["id"],
            "amount": order["amount"],
            "customer_id": order_data["customer_id"]
        })

    def handle_payment_completed(self, event):
        order = self.db.get("orders", event["order_id"])
        order["status"] = "PAID"
        self.db.save("orders", order)

    def handle_payment_failed(self, event):
        order = self.db.get("orders", event["order_id"])
        order["status"] = "CANCELLED"
        self.db.save("orders", order)
        print(f"Order {event['order_id']} cancelled: payment failed")

    def handle_inventory_reserved(self, event):
        order = self.db.get("orders", event["order_id"])
        order["status"] = "CONFIRMED"
        self.db.save("orders", order)

    def handle_inventory_failed(self, event):
        order = self.db.get("orders", event["order_id"])
        order["status"] = "CANCELLED"
        self.db.save("orders", order)
        print(f"Order {event['order_id']} cancelled: inventory unavailable")


# --- Payment Service ---
class PaymentService:
    def __init__(self, gateway):
        self.gateway = gateway

    def handle_order_created(self, event):
        try:
            self.gateway.charge(event["customer_id"], event["amount"])
            bus.publish("PaymentCompleted", {"order_id": event["order_id"]})
        except Exception as e:
            bus.publish("PaymentFailed", {"order_id": event["order_id"]})

    def handle_refund_payment(self, event):
        try:
            self.gateway.refund(event["order_id"])
            bus.publish("PaymentRefunded", {"order_id": event["order_id"]})
        except Exception as e:
            print(f"Refund failed for order {event['order_id']}: {e}")


# --- Inventory Service ---
class InventoryService:
    def __init__(self, db):
        self.db = db

    def handle_payment_completed(self, event):
        try:
            self.db.reserve(event["order_id"])
            bus.publish("InventoryReserved", {"order_id": event["order_id"]})
        except OutOfStockError:
            bus.publish("InventoryFailed", {"order_id": event["order_id"]})
            # Trigger compensation: refund payment
            bus.publish("RefundPayment", {"order_id": event["order_id"]})

    def handle_payment_failed(self, event):
        # Payment failed, no inventory action needed
        pass


# --- Notification Service ---
class NotificationService:
    def handle_inventory_reserved(self, event):
        try:
            self._send_email(event["order_id"])
            bus.publish("NotificationSent", {"order_id": event["order_id"]})
        except Exception as e:
            # Notification failure is non-critical
            print(f"Notification failed for order {event['order_id']}: {e}")

    def _send_email(self, order_id):
        print(f"Sending confirmation email for order {order_id}")


# --- Wire up subscriptions ---
order_service = OrderService(db)
payment_service = PaymentService(gateway)
inventory_service = InventoryService(db)
notification_service = NotificationService()

bus.subscribe("OrderCreated", payment_service.handle_order_created)
bus.subscribe("PaymentCompleted", order_service.handle_payment_completed)
bus.subscribe("PaymentFailed", order_service.handle_payment_failed)
bus.subscribe("PaymentCompleted", inventory_service.handle_payment_completed)
bus.subscribe("InventoryReserved", order_service.handle_inventory_reserved)
bus.subscribe("InventoryFailed", order_service.handle_inventory_failed)
bus.subscribe("InventoryReserved", notification_service.handle_inventory_reserved)
bus.subscribe("RefundPayment", payment_service.handle_refund_payment)
```

### Why This Works

- **Event-driven:** Each service publishes events after completing its local transaction. The next service subscribes to the previous service's success event.
- **Compensation via events:** When Inventory fails, it publishes `InventoryFailed` (which cancels the order) and `RefundPayment` (which triggers a refund). The compensation logic is distributed across services.
- **Decoupling:** Services do not know about each other directly. They only know about events. This makes it easy to add new services without modifying existing ones.

### Common Mistakes to Avoid

- Not handling the case where compensation itself fails (e.g., refund fails). You need retry logic or a dead letter queue for failed compensations.
- Publishing events before the local transaction commits. If the transaction rolls back, the event was already published, causing inconsistency.
- Not making event handlers idempotent. If the same event is delivered twice (due to retry), the handler must handle it gracefully.

---

## Part B: Orchestration Saga

```python
class SagaStep:
    def __init__(self, name, action, compensate):
        self.name = name
        self.action = action
        self.compensate = compensate


class SagaOrchestrator:
    def __init__(self):
        self.steps = []
        self.completed_steps = []

    def add_step(self, name, action, compensate):
        self.steps.append(SagaStep(name, action, compensate))
        return self

    def execute(self, context):
        """Execute all steps. On failure, compensate in reverse order."""
        for i, step in enumerate(self.steps):
            try:
                print(f"Executing step: {step.name}")
                result = step.action(context)
                context[f"{step.name}_result"] = result
                self.completed_steps.append(i)
                print(f"Step {step.name} completed")
            except Exception as e:
                print(f"Step {step.name} failed: {e}")
                self._compensate(context)
                raise SagaStepFailed(step.name, e)

        print("Saga completed successfully")
        return context

    def _compensate(self, context):
        """Run compensating actions in reverse order."""
        print("Starting compensation...")
        for i in reversed(self.completed_steps):
            step = self.steps[i]
            try:
                print(f"Compensating step: {step.name}")
                step.compensate(context)
                print(f"Compensation for {step.name} completed")
            except Exception as e:
                print(f"Compensation for {step.name} failed: {e}")
                # Log and continue -- best effort compensation


class SagaStepFailed(Exception):
    def __init__(self, step, cause):
        self.step = step
        self.cause = cause
        super().__init__(f"Saga failed at step '{step}': {cause}")


# --- Define the order saga ---
def create_order(ctx):
    order = {"id": ctx["order_id"], "status": "PENDING",
             "amount": ctx["amount"]}
    ctx["db"].save("orders", order)
    return order

def cancel_order(ctx):
    order = ctx["db"].get("orders", ctx["order_id"])
    order["status"] = "CANCELLED"
    ctx["db"].save("orders", order)

def charge_payment(ctx):
    return ctx["gateway"].charge(ctx["customer_id"], ctx["amount"])

def refund_payment(ctx):
    ctx["gateway"].refund(ctx["order_id"])

def reserve_inventory(ctx):
    return ctx["db"].reserve(ctx["order_id"])

def release_inventory(ctx):
    ctx["db"].release(ctx["order_id"])

def send_notification(ctx):
    # Non-critical -- compensation is a no-op
    print(f"Sending confirmation for order {ctx['order_id']}")

def noop(ctx):
    pass  # No compensation needed


# --- Build and execute the saga ---
saga = SagaOrchestrator()
saga.add_step("create_order", create_order, cancel_order)
saga.add_step("charge_payment", charge_payment, refund_payment)
saga.add_step("reserve_inventory", reserve_inventory, release_inventory)
saga.add_step("send_notification", send_notification, noop)

context = {
    "order_id": "order-123",
    "customer_id": "cust-456",
    "amount": 99.99,
    "db": db,
    "gateway": payment_gateway
}

try:
    result = saga.execute(context)
    print(f"Order {result['order_id']} completed")
except SagaStepFailed as e:
    print(f"Order failed at {e.step}: {e.cause}")
```

### Why This Works

- **Explicit step sequence:** The orchestrator defines the exact order of steps and their compensations.
- **Completed step tracking:** Only completed steps are compensated. If step 3 fails, steps 1 and 2 are compensated (in reverse order).
- **Best-effort compensation:** If a compensation fails, the orchestrator logs the error and continues with remaining compensations. This is better than stopping mid-compensation.
- **No-op compensation:** Notification is best-effort. Its compensation is a no-op because there is nothing to undo.

### Common Mistakes to Avoid

- Compensating steps that were not executed. The `completed_steps` list prevents this.
- Not handling compensation failures. If a refund fails, you still need to release the inventory and cancel the order.
- Making the orchestrator too generic. A simple list of steps is easier to understand and debug than a complex framework.

---

## Part C: Failure Scenario Analysis

### Scenario 1: Payment succeeds but inventory reservation fails

**Choreography:**
1. Order Service creates order, publishes `OrderCreated`.
2. Payment Service charges customer, publishes `PaymentCompleted`.
3. Inventory Service fails to reserve, publishes `InventoryFailed` and `RefundPayment`.
4. Order Service receives `InventoryFailed`, cancels order.
5. Payment Service receives `RefundPayment`, refunds customer.

**Orchestration:**
1. Orchestrator executes `create_order` (success).
2. Orchestrator executes `charge_payment` (success).
3. Orchestrator executes `reserve_inventory` (fails).
4. Orchestrator compensates: `refund_payment` (step 2), then `cancel_order` (step 1).

**Both approaches achieve the same result, but orchestration is easier to follow because the sequence is explicit.**

### Scenario 2: Payment succeeds, inventory succeeds, but notification fails

**Choreography:**
1. All events chain correctly until `NotificationSent` fails.
2. Notification failure is non-critical. The order is confirmed.
3. No compensation needed -- the order is complete.

**Orchestration:**
1. Steps 1-3 succeed.
2. Step 4 (`send_notification`) fails.
3. Orchestrator compensates steps 3, 2, 1.
4. **Problem:** The order was confirmed, but compensation cancels it.

**This is a design decision.** If notification is truly non-critical, the orchestrator should not compensate on notification failure. Either make notification a no-op failure or use a try/catch that does not trigger compensation.

### Scenario 3: Event bus loses the `PaymentCompleted` message

**Choreography:**
1. Payment Service completes payment and publishes `PaymentCompleted`.
2. The event is lost (network issue, broker crash).
3. Inventory Service never receives the event. The order stays in "PAID" state forever.
4. **Mitigation:** Use a dead letter queue. Payment Service stores the event in a local outbox table. A background process publishes events from the outbox. If publishing fails, it retries.

**Orchestration:**
1. Orchestrator calls `charge_payment` (success).
2. Orchestrator calls `reserve_inventory` (success).
3. No event bus is involved -- direct function calls.
4. **This scenario does not apply to orchestration.** This is a key advantage of orchestration over choreography.

### Scenario 4: Orchestrator crashes after payment but before inventory

**Orchestration:**
1. Orchestrator executes `create_order` (success).
2. Orchestrator executes `charge_payment` (success).
3. Orchestrator crashes.
4. The order is charged but not reserved.
5. **Mitigation:** Persist the saga state (which steps completed) in a database. On restart, the orchestrator reads the state and resumes from the last completed step.

**Choreography:**
1. Order Service creates order, publishes `OrderCreated`.
2. Payment Service charges, publishes `PaymentCompleted`.
3. The choreography continues without a central coordinator.
4. **This scenario does not apply to choreography.** This is a key advantage of choreography over orchestration.

### Common Mistakes to Avoid

- Not persisting saga state in orchestration. A crash loses all progress.
- Not using an outbox pattern in choreography. Lost events leave the saga in an inconsistent state.
- Treating notification failure as a saga failure. Non-critical steps should not trigger compensation.

---

## Key Takeaway

The Saga pattern replaces distributed transactions with local transactions and compensating actions. Choreography distributes logic across events (more decoupled, vulnerable to lost events). Orchestration centralizes logic (easier to understand, vulnerable to orchestrator crashes). Both require compensating transactions that undo previous steps. The choice between them depends on your tolerance for coupling, debugging complexity, and failure modes.
