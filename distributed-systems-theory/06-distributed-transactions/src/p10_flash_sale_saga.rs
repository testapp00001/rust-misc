//! # Exercise: Flash Sale Purchase Saga
//!
//! ## Theory
//!
//! This exercise implements an end-to-end Saga for a flash sale purchase flow.
//! Flash sales have specific challenges:
//!
//! - **High concurrency:** Many users try to purchase simultaneously.
//! - **Limited stock:** Inventory is finite and must be accurately tracked.
//! - **Fast response:** Users expect quick feedback.
//! - **Fairness:** The system should handle requests in order.
//!
//! The saga has four steps:
//!
//! 1. **ReserveStock:** Check and reserve inventory.
//!    - Compensation: ReleaseStock.
//! 2. **ProcessPayment:** Charge the customer.
//!    - Compensation: RefundPayment.
//! 3. **CreateOrder:** Create the order record.
//!    - Compensation: CancelOrder.
//! 4. **SendNotification:** Notify the customer (best-effort, no compensation).
//!
//! ## Proof / Intuition
//!
//! The flash sale saga must handle:
//!
//! - **Insufficient stock:** ReserveStock fails, saga aborts immediately.
//! - **Payment failure:** ProcessPayment fails, compensate by releasing stock.
//! - **Order creation failure:** CreateOrder fails, compensate by refunding
//!   payment and releasing stock.
//! - **Notification failure:** SendNotification fails, but this doesn't affect
//!   the order. The order is still created.
//!
//! Each failure point triggers a different compensation chain:
//!
//! | Failure Point | Compensations |
//! |---------------|---------------|
//! | ReserveStock | None (no state to undo) |
//! | ProcessPayment | ReleaseStock |
//! | CreateOrder | RefundPayment, ReleaseStock |
//! | SendNotification | None (order is valid) |
//!
//! ## Implementation Task
//!
//! Implement the flash sale saga:
//!
//! - `FlashSaleState`: shared inventory and order state.
//! - `FlashSaleSaga`: orchestrates the purchase flow.
//! - Handle each failure mode with appropriate compensation.
//! - Return a `PurchaseResult` with the outcome.
//!
//! ## Verification
//!
//! - Verify happy path completes purchase successfully.
//! - Verify stock compensation on payment failure.
//! - Verify full rollback on order creation failure.

use std::collections::HashMap;

/// The state of a flash sale.
#[derive(Debug, Clone)]
pub struct FlashSaleState {
    /// Product ID.
    pub product_id: u64,
    /// Available stock.
    pub stock: u32,
    /// Price per unit.
    pub price: u64,
    /// Customer balances.
    pub balances: HashMap<u64, u64>,
    /// Created orders.
    pub orders: Vec<u64>,
    /// Sent notifications.
    pub notifications: Vec<u64>,
    /// Reserved stock (pending commit).
    pub reserved: HashMap<u64, u32>,
}

impl FlashSaleState {
    pub fn new(product_id: u64, stock: u32, price: u64) -> Self {
        Self {
            product_id,
            stock,
            price,
            balances: HashMap::new(),
            orders: Vec::new(),
            notifications: Vec::new(),
            reserved: HashMap::new(),
        }
    }

    pub fn set_balance(&mut self, customer_id: u64, balance: u64) {
        self.balances.insert(customer_id, balance);
    }
}

/// Result of a purchase attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PurchaseResult {
    Success { order_id: u64 },
    InsufficientStock,
    PaymentFailed { reason: String },
    OrderFailed { reason: String },
}

/// Compensating actions log entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompensationAction {
    ReleasedStock { quantity: u32 },
    RefundedPayment { amount: u64 },
    CancelledOrder { order_id: u64 },
}

/// The flash sale saga orchestrator.
pub struct FlashSaleSaga {
    pub state: FlashSaleState,
    pub compensations: Vec<CompensationAction>,
    pub customer_id: u64,
    pub quantity: u32,
    /// Simulate failures at each step.
    pub stock_will_fail: bool,
    pub payment_will_fail: bool,
    pub order_will_fail: bool,
}

impl FlashSaleSaga {
    pub fn new(
        state: FlashSaleState,
        customer_id: u64,
        quantity: u32,
    ) -> Self {
        Self {
            state,
            compensations: Vec::new(),
            customer_id,
            quantity,
            stock_will_fail: false,
            payment_will_fail: false,
            order_will_fail: false,
        }
    }

    /// Execute the full flash sale purchase saga.
    pub fn execute(&mut self) -> PurchaseResult {
        // Step 1: Reserve stock.
        match self.reserve_stock() {
            Ok(_) => {}
            Err(result) => return result,
        }

        // Step 2: Process payment.
        match self.process_payment() {
            Ok(_) => {}
            Err(result) => {
                self.compensate_stock();
                return result;
            }
        }

        // Step 3: Create order.
        match self.create_order() {
            Ok(order_id) => {
                // Step 4: Send notification (best-effort).
                self.send_notification(order_id);
                PurchaseResult::Success { order_id }
            }
            Err(result) => {
                self.compensate_payment();
                self.compensate_stock();
                result
            }
        }
    }

    /// Step 1: Reserve stock.
    fn reserve_stock(&mut self) -> Result<(), PurchaseResult> {
        if self.stock_will_fail {
            return Err(PurchaseResult::InsufficientStock);
        }

        if self.state.stock < self.quantity {
            return Err(PurchaseResult::InsufficientStock);
        }

        self.state.stock -= self.quantity;
        self.state.reserved.insert(self.customer_id, self.quantity);
        Ok(())
    }

    /// Step 2: Process payment.
    fn process_payment(&mut self) -> Result<(), PurchaseResult> {
        if self.payment_will_fail {
            return Err(PurchaseResult::PaymentFailed {
                reason: "Payment declined".to_string(),
            });
        }

        let total_cost = self.state.price * self.quantity as u64;
        let balance = self.state.balances.get(&self.customer_id).copied().unwrap_or(0);

        if balance < total_cost {
            return Err(PurchaseResult::PaymentFailed {
                reason: "Insufficient balance".to_string(),
            });
        }

        self.state
            .balances
            .insert(self.customer_id, balance - total_cost);
        Ok(())
    }

    /// Step 3: Create order.
    fn create_order(&mut self) -> Result<u64, PurchaseResult> {
        if self.order_will_fail {
            return Err(PurchaseResult::OrderFailed {
                reason: "Database error".to_string(),
            });
        }

        let order_id = self.state.orders.len() as u64 + 1;
        self.state.orders.push(order_id);
        Ok(order_id)
    }

    /// Step 4: Send notification (best-effort).
    fn send_notification(&mut self, order_id: u64) {
        self.state.notifications.push(order_id);
    }

    /// Compensate: release stock.
    fn compensate_stock(&mut self) {
        let quantity = self.state.reserved.remove(&self.customer_id).unwrap_or(0);
        self.state.stock += quantity;
        self.compensations
            .push(CompensationAction::ReleasedStock { quantity });
    }

    /// Compensate: refund payment.
    fn compensate_payment(&mut self) {
        let total_cost = self.state.price * self.quantity as u64;
        let balance = self.state.balances.get(&self.customer_id).copied().unwrap_or(0);
        self.state
            .balances
            .insert(self.customer_id, balance + total_cost);
        self.compensations
            .push(CompensationAction::RefundedPayment {
                amount: total_cost,
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_state() -> FlashSaleState {
        let mut state = FlashSaleState::new(1, 100, 100);
        state.set_balance(1001, 10000);
        state
    }

    #[test]
    fn happy_path_completes_purchase() {
        let state = make_state();
        let mut saga = FlashSaleSaga::new(state, 1001, 1);
        let result = saga.execute();

        match result {
            PurchaseResult::Success { order_id } => {
                assert!(order_id > 0);
            }
            _ => panic!("Expected Success, got {:?}", result),
        }

        assert_eq!(saga.compensations.len(), 0, "No compensations should be needed");
        assert_eq!(saga.state.stock, 99);
        assert_eq!(saga.state.orders.len(), 1);
    }

    #[test]
    fn stock_compensation_on_payment_failure() {
        let state = make_state();
        let mut saga = FlashSaleSaga::new(state, 1001, 1);
        saga.payment_will_fail = true;

        let result = saga.execute();

        assert_eq!(
            result,
            PurchaseResult::PaymentFailed {
                reason: "Payment declined".to_string()
            }
        );

        // Stock should have been released (compensation).
        assert!(
            saga.compensations
                .iter()
                .any(|c| matches!(c, CompensationAction::ReleasedStock { .. })),
            "Stock should be released on payment failure"
        );
        // Stock should be restored to original value.
        assert_eq!(saga.state.stock, 100);
    }

    #[test]
    fn full_rollback_on_order_creation_failure() {
        let state = make_state();
        let mut saga = FlashSaleSaga::new(state, 1001, 1);
        saga.order_will_fail = true;

        let result = saga.execute();

        assert!(matches!(result, PurchaseResult::OrderFailed { .. }));

        // Both stock and payment should be compensated.
        assert!(
            saga.compensations
                .iter()
                .any(|c| matches!(c, CompensationAction::ReleasedStock { .. })),
            "Stock should be released"
        );
        assert!(
            saga.compensations
                .iter()
                .any(|c| matches!(c, CompensationAction::RefundedPayment { .. })),
            "Payment should be refunded"
        );

        // State should be fully restored.
        assert_eq!(saga.state.stock, 100);
        assert_eq!(
            saga.state.balances.get(&1001).copied().unwrap_or(0),
            10000
        );
    }

    #[test]
    fn insufficient_stock_fails_immediately() {
        let mut state = make_state();
        state.stock = 0;
        let mut saga = FlashSaleSaga::new(state, 1001, 1);
        let result = saga.execute();

        assert_eq!(result, PurchaseResult::InsufficientStock);
        assert_eq!(saga.compensations.len(), 0, "No compensations needed for insufficient stock");
    }

    #[test]
    fn multiple_purchases_deplete_stock() {
        let state = make_state();
        let mut saga = FlashSaleSaga::new(state, 1001, 100);
        let result = saga.execute();

        assert!(matches!(result, PurchaseResult::Success { .. }));
        assert_eq!(saga.state.stock, 0, "Stock should be fully depleted");
    }

    #[test]
    fn compensation_actions_logged() {
        let state = make_state();
        let mut saga = FlashSaleSaga::new(state, 1001, 1);
        saga.payment_will_fail = true;

        let _ = saga.execute();

        assert_eq!(saga.compensations.len(), 1);
        assert!(matches!(
            saga.compensations[0],
            CompensationAction::ReleasedStock { quantity: 1 }
        ));
    }
}
