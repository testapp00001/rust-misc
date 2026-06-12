//! # Exercise: Saga Pattern with Choreography
//!
//! ## Theory
//!
//! The Saga pattern decomposes a distributed transaction into a sequence of local
//! transactions, each with a compensating transaction. Unlike 2PC, Sagas do not
//! provide atomicity but rather eventual consistency.
//!
//! **Choreography** is the decentralized coordination approach:
//! - Each service publishes events after completing its step.
//! - Other services react to events and execute their steps.
//! - No central coordinator -- the workflow emerges from event interactions.
//!
//! ## Proof / Intuition
//!
//! Consider a purchase saga with three services:
//!
//! 1. **StockService:** Reserve stock, publish `StockReserved` event.
//!    - Compensation: `ReleaseStock`.
//! 2. **PaymentService:** On `StockReserved`, process payment.
//!    - Publish `PaymentProcessed`.
//!    - Compensation: `RefundPayment`.
//! 3. **OrderService:** On `PaymentProcessed`, create order.
//!    - Publish `OrderCreated`.
//!    - No compensation needed (order can be cancelled separately).
//!
//! **Happy path:** StockReserved -> PaymentProcessed -> OrderCreated.
//!
//! **Failure path (payment fails):**
//! 1. StockService reserves stock.
//! 2. PaymentService fails to process payment.
//! 3. PaymentService publishes `PaymentFailed`.
//! 4. StockService reacts to `PaymentFailed` and releases stock.
//!
//! **Advantages of choreography:**
//! - Loose coupling between services.
//! - No single point of failure.
//!
//! **Disadvantages:**
//! - Hard to understand the overall flow.
//! - Difficult to add new steps.
//! - Cyclic dependencies possible.
//!
//! ## Implementation Task
//!
//! Implement a choreography-based saga:
//!
//! - `Event`: An event in the system (StockReserved, PaymentProcessed, etc.).
//! - `EventBus`: A simple pub-sub event bus.
//! - `StockService`, `PaymentService`, `OrderService`: Service implementations.
//! - Wire them together via the event bus.
//!
//! ## Verification
//!
//! - Verify happy path completes with all steps succeeding.
//! - Verify compensation runs when payment fails.
//! - Verify no orphaned state after compensation.

/// Events in the saga.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// Saga started with an order.
    SagaStarted { order_id: u64 },
    /// Stock reserved successfully.
    StockReserved { order_id: u64, quantity: u32 },
    /// Stock reservation failed.
    StockReservationFailed { order_id: u64, reason: String },
    /// Payment processed successfully.
    PaymentProcessed { order_id: u64, amount: u64 },
    /// Payment failed.
    PaymentFailed { order_id: u64, reason: String },
    /// Order created successfully.
    OrderCreated { order_id: u64 },
    /// Stock released (compensation).
    StockReleased { order_id: u64 },
    /// Payment refunded (compensation).
    PaymentRefunded { order_id: u64 },
}

/// A simple synchronous event bus.
pub struct EventBus {
    /// Listeners for each event type (simplified: all listeners get all events).
    handlers: Vec<Box<dyn FnMut(&Event) -> Vec<Event>>>,
    /// History of all events.
    pub history: Vec<Event>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
            history: Vec::new(),
        }
    }

    /// Register a handler function.
    pub fn subscribe(&mut self, handler: Box<dyn FnMut(&Event) -> Vec<Event>>) {
        self.handlers.push(handler);
    }

    /// Publish an event and collect all resulting events.
    pub fn publish(&mut self, event: Event) -> Vec<Event> {
        self.history.push(event.clone());
        let mut all_new_events = Vec::new();

        for handler in &mut self.handlers {
            let new_events = handler(&event);
            all_new_events.extend(new_events);
        }

        all_new_events
    }

    /// Publish an event and recursively process all resulting events.
    pub fn publish_recursive(&mut self, event: Event) {
        let mut queue = vec![event];

        while let Some(current) = queue.pop() {
            let new_events = self.publish(current);
            for e in new_events {
                queue.push(e);
            }
        }
    }
}

/// State of a stock reservation.
#[derive(Debug, Clone)]
pub struct StockState {
    pub reserved: bool,
    pub quantity: u32,
    pub order_id: Option<u64>,
}

/// State of a payment.
#[derive(Debug, Clone)]
pub struct PaymentState {
    pub processed: bool,
    pub amount: u64,
    pub order_id: Option<u64>,
}

/// State of an order.
#[derive(Debug, Clone)]
pub struct OrderState {
    pub created: bool,
    pub order_id: Option<u64>,
}

/// Stock service that handles stock events.
pub struct StockService {
    pub stock_state: StockState,
    /// If true, the next reservation will fail.
    pub will_fail: bool,
}

impl StockService {
    pub fn new() -> Self {
        Self {
            stock_state: StockState {
                reserved: false,
                quantity: 0,
                order_id: None,
            },
            will_fail: false,
        }
    }

    pub fn handle(&mut self, event: &Event) -> Vec<Event> {
        match event {
            Event::SagaStarted { order_id } => {
                if self.will_fail {
                    vec![Event::StockReservationFailed {
                        order_id: *order_id,
                        reason: "Insufficient stock".to_string(),
                    }]
                } else {
                    self.stock_state = StockState {
                        reserved: true,
                        quantity: 1,
                        order_id: Some(*order_id),
                    };
                    vec![Event::StockReserved {
                        order_id: *order_id,
                        quantity: 1,
                    }]
                }
            }
            Event::PaymentFailed { order_id, .. } => {
                // Compensation: release stock.
                self.stock_state = StockState {
                    reserved: false,
                    quantity: 0,
                    order_id: None,
                };
                vec![Event::StockReleased { order_id: *order_id }]
            }
            _ => vec![],
        }
    }
}

/// Payment service that handles payment events.
pub struct PaymentService {
    pub payment_state: PaymentState,
    /// If true, the next payment will fail.
    pub will_fail: bool,
}

impl PaymentService {
    pub fn new() -> Self {
        Self {
            payment_state: PaymentState {
                processed: false,
                amount: 0,
                order_id: None,
            },
            will_fail: false,
        }
    }

    pub fn handle(&mut self, event: &Event) -> Vec<Event> {
        match event {
            Event::StockReserved { order_id, .. } => {
                if self.will_fail {
                    vec![Event::PaymentFailed {
                        order_id: *order_id,
                        reason: "Insufficient funds".to_string(),
                    }]
                } else {
                    self.payment_state = PaymentState {
                        processed: true,
                        amount: 100,
                        order_id: Some(*order_id),
                    };
                    vec![Event::PaymentProcessed {
                        order_id: *order_id,
                        amount: 100,
                    }]
                }
            }
            _ => vec![],
        }
    }
}

/// Order service that handles order events.
pub struct OrderService {
    pub order_state: OrderState,
}

impl OrderService {
    pub fn new() -> Self {
        Self {
            order_state: OrderState {
                created: false,
                order_id: None,
            },
        }
    }

    pub fn handle(&mut self, event: &Event) -> Vec<Event> {
        match event {
            Event::PaymentProcessed { order_id, .. } => {
                self.order_state = OrderState {
                    created: true,
                    order_id: Some(*order_id),
                };
                vec![Event::OrderCreated { order_id: *order_id }]
            }
            _ => vec![],
        }
    }
}

/// Result of a saga execution.
#[derive(Debug)]
pub struct SagaResult {
    pub stock_reserved: bool,
    pub payment_processed: bool,
    pub order_created: bool,
    pub event_count: usize,
}

/// Run the choreography saga with the given service configurations.
pub fn run_choreography_saga(
    order_id: u64,
    stock_will_fail: bool,
    payment_will_fail: bool,
) -> (SagaResult, Vec<Event>) {
    let mut bus = EventBus::new();
    let mut stock = StockService::new();
    stock.will_fail = stock_will_fail;
    let mut payment = PaymentService::new();
    payment.will_fail = payment_will_fail;
    let mut order = OrderService::new();

    let stock_handler = Box::new(move |e: &Event| stock.handle(e));
    let payment_handler = Box::new(move |e: &Event| payment.handle(e));
    let order_handler = Box::new(move |e: &Event| order.handle(e));

    bus.subscribe(stock_handler);
    bus.subscribe(payment_handler);
    bus.subscribe(order_handler);

    // Start the saga.
    bus.publish_recursive(Event::SagaStarted { order_id });

    let result = SagaResult {
        stock_reserved: bus.history.iter().any(|e| matches!(e, Event::StockReserved { .. })),
        payment_processed: bus
            .history
            .iter()
            .any(|e| matches!(e, Event::PaymentProcessed { .. })),
        order_created: bus
            .history
            .iter()
            .any(|e| matches!(e, Event::OrderCreated { .. })),
        event_count: bus.history.len(),
    };

    (result, bus.history)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn happy_path_completes_all_steps() {
        let (result, history) = run_choreography_saga(1, false, false);

        assert!(result.stock_reserved, "Stock should be reserved");
        assert!(result.payment_processed, "Payment should be processed");
        assert!(result.order_created, "Order should be created");
        assert!(
            result.event_count >= 4,
            "Should have at least 4 events (Started, Reserved, Paid, Created)"
        );
    }

    #[test]
    fn compensation_on_payment_failure() {
        let (result, history) = run_choreography_saga(1, false, true);

        assert!(result.stock_reserved, "Stock should be reserved initially");
        assert!(!result.payment_processed, "Payment should fail");
        assert!(!result.order_created, "Order should not be created");

        // Stock should be released (compensation).
        let stock_released = history
            .iter()
            .any(|e| matches!(e, Event::StockReleased { .. }));
        assert!(
            stock_released,
            "Stock should be released after payment failure"
        );
    }

    #[test]
    fn stock_failure_prevents_further_steps() {
        let (result, history) = run_choreography_saga(1, true, false);

        assert!(!result.stock_reserved, "Stock should not be reserved");
        assert!(!result.payment_processed, "Payment should not be processed");
        assert!(!result.order_created, "Order should not be created");
    }

    #[test]
    fn event_history_captured_correctly() {
        let (_, history) = run_choreography_saga(42, false, false);

        // First event should be SagaStarted.
        assert_eq!(history[0], Event::SagaStarted { order_id: 42 });

        // Last event should be OrderCreated.
        assert!(matches!(
            history.last(),
            Some(Event::OrderCreated { order_id: 42 })
        ));
    }

    #[test]
    fn multiple_sagas_independent() {
        let (result1, _) = run_choreography_saga(1, false, false);
        let (result2, _) = run_choreography_saga(2, false, true);

        assert!(result1.order_created);
        assert!(!result2.order_created);
    }
}
