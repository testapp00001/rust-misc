//! # Exercise: Saga-Based Order System
//!
//! ## Theory
//!
//! The Saga pattern is a way to manage distributed transactions across multiple
//! services. Instead of a single ACID transaction spanning all services, a saga
//! breaks the transaction into a sequence of local transactions, each with a
//! corresponding compensating action (rollback).
//!
//! If any step fails, the saga executes the compensating actions for all
//! completed steps in reverse order, ensuring the system returns to a
//! consistent state. This is the "forward recovery" approach.
//!
//! ## Proof / Intuition
//!
//! Consider an order system with 3 services: Inventory, Payment, Shipping.
//! A successful order requires: Reserve Inventory -> Charge Payment -> Ship.
//!
//! If Payment fails after Inventory was reserved:
//! 1. Execute compensation for Payment (no-op since it failed)
//! 2. Execute compensation for Inventory (release reserved items)
//!
//! This guarantees that no partial state is left behind. The key invariant is:
//! For every forward action A_i, there exists a compensation C_i such that
//! executing C_i after A_i effectively undoes the effect of A_i.
//!
//! ## Implementation Task
//!
//! 1. Implement a SagaOrchestrator that manages sequential step execution
//! 2. Create service stubs for Inventory, Payment, Shipping, and Notification
//! 3. Implement compensation logic for each service
//! 4. Handle failures by rolling back completed steps in reverse order
//! 5. Verify that the system is always in a consistent state
//!
//! ## Verification
//!
//! - Test happy path: all steps succeed, order is complete
//! - Test failure: payment fails, inventory is released
//! - Test partial rollback: shipping fails after payment, both are compensated

use std::fmt;

/// Result of executing a saga step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SagaStepResult {
    Success,
    Failure(String),
}

/// Status of the overall saga.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SagaStatus {
    Running,
    Completed,
    Compensating,
    Compensated,
    Failed(String),
}

/// A step in a saga, with an action and a compensation.
pub struct SagaStep {
    /// Name of this step for logging/tracking.
    pub name: String,
    /// The forward action to execute.
    pub action: Box<dyn Fn() -> SagaStepResult>,
    /// The compensation (rollback) action.
    pub compensation: Box<dyn Fn() -> SagaStepResult>,
}

impl fmt::Debug for SagaStep {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SagaStep")
            .field("name", &self.name)
            .finish()
    }
}

/// Orchestrates the execution of saga steps with compensation on failure.
pub struct SagaOrchestrator {
    /// The steps to execute in order.
    steps: Vec<SagaStep>,
    /// Current status of the saga.
    status: SagaStatus,
    /// Names of steps that have been completed (for compensation tracking).
    completed_steps: Vec<String>,
    /// Index of the next step to execute.
    current_step: usize,
}

impl SagaOrchestrator {
    /// Create a new saga orchestrator.
    pub fn new() -> Self {
        Self {
            steps: Vec::new(),
            status: SagaStatus::Running,
            completed_steps: Vec::new(),
            current_step: 0,
        }
    }

    /// Add a step to the saga.
    pub fn add_step(&mut self, step: SagaStep) {
        self.steps.push(step);
    }

    /// Execute all saga steps sequentially. If any step fails,
    /// compensate all completed steps in reverse order.
    pub fn execute(&mut self) -> &SagaStatus {
        while self.current_step < self.steps.len() {
            let step = &self.steps[self.current_step];
            let step_name = step.name.clone();

            // Execute the forward action
            let result = (step.action)();

            match result {
                SagaStepResult::Success => {
                    self.completed_steps.push(step_name);
                    self.current_step += 1;
                }
                SagaStepResult::Failure(msg) => {
                    self.status = SagaStatus::Compensating;
                    // Compensate completed steps in reverse order
                    self.compensate();
                    self.status = SagaStatus::Failed(msg);
                    return &self.status;
                }
            }
        }

        self.status = SagaStatus::Completed;
        &self.status
    }

    /// Compensate all completed steps in reverse order.
    fn compensate(&mut self) {
        for step_name in self.completed_steps.iter().rev() {
            if let Some(step) = self.steps.iter().find(|s| &s.name == step_name) {
                let _ = (step.compensation)();
            }
        }
        self.completed_steps.clear();
    }

    /// Get the current status.
    pub fn get_status(&self) -> &SagaStatus {
        &self.status
    }

    /// Get the list of completed step names.
    pub fn completed_steps(&self) -> &[String] {
        &self.completed_steps
    }
}

impl Default for SagaOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Service Stubs
// =============================================================================

/// Inventory service stub.
pub struct InventoryService {
    /// Items currently reserved.
    reserved: Vec<String>,
    /// If true, reserve operations will fail.
    fail_reserve: bool,
}

impl InventoryService {
    pub fn new() -> Self {
        Self {
            reserved: Vec::new(),
            fail_reserve: false,
        }
    }

    pub fn set_fail(&mut self, fail: bool) {
        self.fail_reserve = fail;
    }

    pub fn reserve(&mut self, item: &str) -> SagaStepResult {
        if self.fail_reserve {
            return SagaStepResult::Failure(format!("Failed to reserve {}", item));
        }
        self.reserved.push(item.to_string());
        SagaStepResult::Success
    }

    pub fn release(&mut self, item: &str) -> SagaStepResult {
        self.reserved.retain(|i| i != item);
        SagaStepResult::Success
    }

    pub fn reserved_items(&self) -> &[String] {
        &self.reserved
    }
}

impl Default for InventoryService {
    fn default() -> Self {
        Self::new()
    }
}

/// Payment service stub.
pub struct PaymentService {
    /// If true, charge operations will fail.
    fail_charge: bool,
    /// Amount charged.
    charged: bool,
}

impl PaymentService {
    pub fn new() -> Self {
        Self {
            fail_charge: false,
            charged: false,
        }
    }

    pub fn set_fail(&mut self, fail: bool) {
        self.fail_charge = fail;
    }

    pub fn charge(&mut self, amount: f64) -> SagaStepResult {
        if self.fail_charge {
            return SagaStepResult::Failure(format!("Payment failed for ${:.2}", amount));
        }
        self.charged = true;
        SagaStepResult::Success
    }

    pub fn refund(&mut self) -> SagaStepResult {
        self.charged = false;
        SagaStepResult::Success
    }

    pub fn is_charged(&self) -> bool {
        self.charged
    }
}

impl Default for PaymentService {
    fn default() -> Self {
        Self::new()
    }
}

/// Shipping service stub.
pub struct ShippingService {
    /// If true, schedule operations will fail.
    fail_schedule: bool,
    /// Whether a shipment is scheduled.
    scheduled: bool,
}

impl ShippingService {
    pub fn new() -> Self {
        Self {
            fail_schedule: false,
            scheduled: false,
        }
    }

    pub fn set_fail(&mut self, fail: bool) {
        self.fail_schedule = fail;
    }

    pub fn schedule(&mut self, order_id: &str) -> SagaStepResult {
        if self.fail_schedule {
            return SagaStepResult::Failure(format!("Shipping failed for order {}", order_id));
        }
        self.scheduled = true;
        SagaStepResult::Success
    }

    pub fn cancel(&mut self) -> SagaStepResult {
        self.scheduled = false;
        SagaStepResult::Success
    }

    pub fn is_scheduled(&self) -> bool {
        self.scheduled
    }
}

impl Default for ShippingService {
    fn default() -> Self {
        Self::new()
    }
}

/// Notification service stub.
pub struct NotificationService {
    /// Whether a notification was sent.
    sent: bool,
}

impl NotificationService {
    pub fn new() -> Self {
        Self { sent: false }
    }

    pub fn send(&mut self, message: &str) -> SagaStepResult {
        self.sent = true;
        let _ = message;
        SagaStepResult::Success
    }

    pub fn cancel(&mut self) -> SagaStepResult {
        self.sent = false;
        SagaStepResult::Success
    }

    pub fn was_sent(&self) -> bool {
        self.sent
    }
}

impl Default for NotificationService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn test_happy_path_all_succeed() {
        let inventory = Rc::new(RefCell::new(InventoryService::new()));
        let payment = Rc::new(RefCell::new(PaymentService::new()));
        let shipping = Rc::new(RefCell::new(ShippingService::new()));
        let notification = Rc::new(RefCell::new(NotificationService::new()));

        let inv_reserved = Rc::new(RefCell::new(Vec::<String>::new()));
        let inv_reserved_clone = inv_reserved.clone();

        let inventory_clone = inventory.clone();
        let payment_clone = payment.clone();
        let payment_clone2 = payment.clone();
        let shipping_clone = shipping.clone();
        let shipping_clone2 = shipping.clone();
        let notification_clone = notification.clone();
        let notification_clone2 = notification.clone();

        let mut saga = SagaOrchestrator::new();

        saga.add_step(SagaStep {
            name: "reserve_inventory".to_string(),
            action: Box::new(move || {
                let result = inventory_clone.borrow_mut().reserve("widget");
                if let SagaStepResult::Success = result {
                    inv_reserved_clone.borrow_mut().push("widget".to_string());
                }
                result
            }),
            compensation: Box::new(move || {
                // This won't be called in happy path
                SagaStepResult::Success
            }),
        });

        saga.add_step(SagaStep {
            name: "charge_payment".to_string(),
            action: Box::new(move || payment_clone.borrow_mut().charge(29.99)),
            compensation: Box::new(move || payment_clone2.borrow_mut().refund()),
        });

        saga.add_step(SagaStep {
            name: "schedule_shipping".to_string(),
            action: Box::new(move || shipping_clone.borrow_mut().schedule("ORD-001")),
            compensation: Box::new(move || shipping_clone2.borrow_mut().cancel()),
        });

        saga.add_step(SagaStep {
            name: "send_notification".to_string(),
            action: Box::new(move || notification_clone.borrow_mut().send("Order confirmed!")),
            compensation: Box::new(move || notification_clone2.borrow_mut().cancel()),
        });

        let status = saga.execute();
        assert_eq!(status, &SagaStatus::Completed);
        assert_eq!(inv_reserved.borrow().len(), 1);
    }

    #[test]
    fn test_compensation_on_payment_failure() {
        let inventory = Rc::new(RefCell::new(InventoryService::new()));
        let inv_reserved = Rc::new(RefCell::new(Vec::<String>::new()));

        let payment = Rc::new(RefCell::new(PaymentService::new()));
        payment.borrow_mut().set_fail(true);

        let inv_reserved_clone = inv_reserved.clone();
        let inv_reserved_clone2 = inv_reserved.clone();
        let inventory_clone = inventory.clone();
        let inventory_clone2 = inventory.clone();

        let mut saga = SagaOrchestrator::new();

        saga.add_step(SagaStep {
            name: "reserve_inventory".to_string(),
            action: Box::new(move || {
                let result = inventory_clone.borrow_mut().reserve("gadget");
                if let SagaStepResult::Success = result {
                    inv_reserved_clone.borrow_mut().push("gadget".to_string());
                }
                result
            }),
            compensation: Box::new(move || {
                inventory_clone2.borrow_mut().release("gadget");
                inv_reserved_clone2.borrow_mut().clear();
                SagaStepResult::Success
            }),
        });

        let payment_clone = payment.clone();
        let payment_clone2 = payment.clone();

        saga.add_step(SagaStep {
            name: "charge_payment".to_string(),
            action: Box::new(move || payment_clone.borrow_mut().charge(49.99)),
            compensation: Box::new(move || payment_clone2.borrow_mut().refund()),
        });

        let status = saga.execute();

        match status {
            SagaStatus::Failed(msg) => {
                assert!(msg.contains("Payment failed"));
            }
            _ => panic!("Expected Failed status, got {:?}", status),
        }

        // Inventory should have been released by compensation
        assert_eq!(inv_reserved.borrow().len(), 0);
    }

    #[test]
    fn test_partial_rollback_on_shipping_failure() {
        let inventory = Rc::new(RefCell::new(InventoryService::new()));
        let inv_reserved = Rc::new(RefCell::new(Vec::<String>::new()));

        let payment = Rc::new(RefCell::new(PaymentService::new()));
        let payment_charged = Rc::new(RefCell::new(false));

        let shipping = Rc::new(RefCell::new(ShippingService::new()));
        shipping.borrow_mut().set_fail(true);

        let inv_reserved_clone = inv_reserved.clone();
        let inv_reserved_clone2 = inv_reserved.clone();
        let inventory_clone = inventory.clone();
        let inventory_clone2 = inventory.clone();

        let payment_charged_clone = payment_charged.clone();
        let payment_clone = payment.clone();
        let payment_clone2 = payment.clone();
        let payment_charged_clone2 = payment_charged.clone();

        let shipping_clone = shipping.clone();
        let shipping_clone2 = shipping.clone();

        let mut saga = SagaOrchestrator::new();

        saga.add_step(SagaStep {
            name: "reserve_inventory".to_string(),
            action: Box::new(move || {
                let result = inventory_clone.borrow_mut().reserve("item");
                if let SagaStepResult::Success = result {
                    inv_reserved_clone.borrow_mut().push("item".to_string());
                }
                result
            }),
            compensation: Box::new(move || {
                inventory_clone2.borrow_mut().release("item");
                inv_reserved_clone2.borrow_mut().clear();
                SagaStepResult::Success
            }),
        });

        saga.add_step(SagaStep {
            name: "charge_payment".to_string(),
            action: Box::new(move || {
                let result = payment_clone.borrow_mut().charge(99.99);
                if let SagaStepResult::Success = result {
                    *payment_charged_clone.borrow_mut() = true;
                }
                result
            }),
            compensation: Box::new(move || {
                payment_clone2.borrow_mut().refund();
                *payment_charged_clone2.borrow_mut() = false;
                SagaStepResult::Success
            }),
        });

        saga.add_step(SagaStep {
            name: "schedule_shipping".to_string(),
            action: Box::new(move || shipping_clone.borrow_mut().schedule("ORD-002")),
            compensation: Box::new(move || shipping_clone2.borrow_mut().cancel()),
        });

        let status = saga.execute();

        match status {
            SagaStatus::Failed(msg) => {
                assert!(msg.contains("Shipping failed"));
            }
            _ => panic!("Expected Failed status, got {:?}", status),
        }

        // Both inventory and payment should be compensated
        assert_eq!(inv_reserved.borrow().len(), 0);
        assert!(!*payment_charged.borrow());
    }
}
