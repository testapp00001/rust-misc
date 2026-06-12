//! # Exercise: Compensating Transactions Deep Dive
//!
//! ## Theory
//!
//! Compensating transactions are the backbone of the Saga pattern. Each forward
//! transaction has a corresponding compensation that "undoes" its effects.
//!
//! However, compensation is NOT the same as rollback:
//!
//! - **Rollback:** The database undoes all changes atomically. The transaction
//!   appears as if it never happened.
//! - **Compensation:** A new transaction that semantically reverses the effects.
//!   The original transaction DID happen and its side effects (emails sent, logs
//!   written, external API calls) cannot be truly undone.
//!
//! ## Proof / Intuition
//!
//! Consider a payment charge:
//! - **Forward:** Charge $100 to credit card.
//! - **Compensation:** Refund $100 to credit card.
//!
//! The refund does NOT undo the charge -- the customer's statement will show both
//! the charge and the refund. The compensation "undoes" the business effect (the
//! customer has their money back) but not the actual events.
//!
//! Additional properties of compensating transactions:
//!
//! 1. **Idempotency:** A compensation may be applied multiple times without
//!    additional effect. This is necessary because retries are common.
//! 2. **Commutativity:** Compensations should produce the same result regardless
//!    of the order they are applied (within a single saga).
//! 3. **Semantic reversal:** Compensation must restore the *business* state,
//!    not necessarily the exact technical state.
//!
//! ## Implementation Task
//!
//! Implement compensating transactions for:
//!
//! - `InventoryReserve` / `InventoryRelease`
//! - `PaymentCharge` / `PaymentRefund`
//! - `OrderCreate` / `OrderCancel`
//!
//! Each must implement:
//! - `CompensatingTransaction` trait with `execute()` and `compensate()`.
//! - Idempotent compensation (safe to call multiple times).
//! - Track the semantic state changes.
//!
//! ## Verification
//!
//! - Verify compensation restores business state.
//! - Verify idempotent compensation (calling compensate twice is safe).
//! - Verify that compensation does not perfectly undo side effects.

/// Trait for compensating transactions.
pub trait CompensatingTransaction {
    /// Execute the forward transaction.
    fn execute(&mut self) -> Result<(), String>;
    /// Execute the compensating transaction.
    fn compensate(&mut self) -> Result<(), String>;
    /// Check if the transaction has been executed.
    fn is_executed(&self) -> bool;
    /// Check if the transaction has been compensated.
    fn is_compensated(&self) -> bool;
}

/// Inventory reservation and its compensation.
#[derive(Debug)]
pub struct InventoryReserve {
    pub product_id: u64,
    pub quantity: u32,
    pub reserved: bool,
    pub released: bool,
    /// Side effect log (simulates external system calls).
    pub side_effects: Vec<String>,
}

impl InventoryReserve {
    pub fn new(product_id: u64, quantity: u32) -> Self {
        Self {
            product_id,
            quantity,
            reserved: false,
            released: false,
            side_effects: Vec::new(),
        }
    }
}

impl CompensatingTransaction for InventoryReserve {
    fn execute(&mut self) -> Result<(), String> {
        self.reserved = true;
        self.released = false;
        self.side_effects
            .push(format!("RESERVED {} units of product {}", self.quantity, self.product_id));
        Ok(())
    }

    fn compensate(&mut self) -> Result<(), String> {
        if !self.reserved {
            return Ok(()); // Idempotent: already released.
        }
        self.released = true;
        self.reserved = false;
        self.side_effects
            .push(format!("RELEASED {} units of product {}", self.quantity, self.product_id));
        Ok(())
    }

    fn is_executed(&self) -> bool {
        self.reserved
    }

    fn is_compensated(&self) -> bool {
        self.released
    }
}

/// Payment charge and its compensation.
#[derive(Debug)]
pub struct PaymentCharge {
    pub customer_id: u64,
    pub amount: u64,
    pub charged: bool,
    pub refunded: bool,
    /// Side effects: actual payment gateway calls.
    pub side_effects: Vec<String>,
}

impl PaymentCharge {
    pub fn new(customer_id: u64, amount: u64) -> Self {
        Self {
            customer_id,
            amount,
            charged: false,
            refunded: false,
            side_effects: Vec::new(),
        }
    }
}

impl CompensatingTransaction for PaymentCharge {
    fn execute(&mut self) -> Result<(), String> {
        self.charged = true;
        self.refunded = false;
        self.side_effects
            .push(format!("CHARGED ${} from customer {}", self.amount, self.customer_id));
        Ok(())
    }

    fn compensate(&mut self) -> Result<(), String> {
        if !self.charged {
            return Ok(()); // Idempotent: nothing to refund.
        }
        self.refunded = true;
        self.charged = false;
        self.side_effects
            .push(format!("REFUNDED ${} to customer {}", self.amount, self.customer_id));
        Ok(())
    }

    fn is_executed(&self) -> bool {
        self.charged
    }

    fn is_compensated(&self) -> bool {
        self.refunded
    }
}

/// Order creation and its compensation.
#[derive(Debug)]
pub struct OrderCreate {
    pub order_id: u64,
    pub created: bool,
    pub cancelled: bool,
    pub side_effects: Vec<String>,
}

impl OrderCreate {
    pub fn new(order_id: u64) -> Self {
        Self {
            order_id,
            created: false,
            cancelled: false,
            side_effects: Vec::new(),
        }
    }
}

impl CompensatingTransaction for OrderCreate {
    fn execute(&mut self) -> Result<(), String> {
        self.created = true;
        self.cancelled = false;
        self.side_effects
            .push(format!("ORDER_CREATED({})", self.order_id));
        Ok(())
    }

    fn compensate(&mut self) -> Result<(), String> {
        if !self.created {
            return Ok(()); // Idempotent.
        }
        self.cancelled = true;
        self.created = false;
        self.side_effects
            .push(format!("ORDER_CANCELLED({})", self.order_id));
        Ok(())
    }

    fn is_executed(&self) -> bool {
        self.created
    }

    fn is_compensated(&self) -> bool {
        self.cancelled
    }
}

/// Check that calling compensate twice doesn't cause errors.
pub fn test_idempotent_compensation(tx: &mut impl CompensatingTransaction) -> bool {
    let _ = tx.execute();
    let first_result = tx.compensate();
    let second_result = tx.compensate();
    first_result.is_ok() && second_result.is_ok()
}

/// Check that side effects are NOT undone (compensation != rollback).
pub fn check_side_effects_not_undone(tx: &mut (impl CompensatingTransaction + HasSideEffects)) -> bool {
    let _ = tx.execute();
    let side_effects_after_execute = tx.side_effects().to_vec();

    let _ = tx.compensate();
    let side_effects_after_compensate = tx.side_effects().to_vec();

    // The side effects from execution should still be present after compensation.
    // Compensation ADDS new entries but doesn't remove old ones.
    side_effects_after_execute.len() > 0
        && side_effects_after_compensate.len() > side_effects_after_execute.len()
}

/// Helper trait to access side effects.
pub trait HasSideEffects {
    fn side_effects(&self) -> &[String];
}

impl HasSideEffects for InventoryReserve {
    fn side_effects(&self) -> &[String] {
        &self.side_effects
    }
}

impl HasSideEffects for PaymentCharge {
    fn side_effects(&self) -> &[String] {
        &self.side_effects
    }
}

impl HasSideEffects for OrderCreate {
    fn side_effects(&self) -> &[String] {
        &self.side_effects
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compensation_restores_business_state() {
        let mut inventory = InventoryReserve::new(1, 10);
        let mut payment = PaymentCharge::new(100, 50);
        let mut order = OrderCreate::new(42);

        // Execute all.
        inventory.execute().unwrap();
        payment.execute().unwrap();
        order.execute().unwrap();

        // Compensate all.
        order.compensate().unwrap();
        payment.compensate().unwrap();
        inventory.compensate().unwrap();

        // Business state should be restored.
        assert!(!inventory.reserved);
        assert!(!payment.charged);
        assert!(!order.created);
    }

    #[test]
    fn idempotent_compensation_safe() {
        let mut inventory = InventoryReserve::new(1, 5);
        assert!(test_idempotent_compensation(&mut inventory));

        let mut payment = PaymentCharge::new(100, 50);
        assert!(test_idempotent_compensation(&mut payment));

        let mut order = OrderCreate::new(1);
        assert!(test_idempotent_compensation(&mut order));
    }

    #[test]
    fn side_effects_not_undone_by_compensation() {
        let mut payment = PaymentCharge::new(100, 50);
        assert!(check_side_effects_not_undone(&mut payment));

        let mut inventory = InventoryReserve::new(1, 10);
        assert!(check_side_effects_not_undone(&mut inventory));
    }

    #[test]
    fn inventory_compensation_records_release() {
        let mut inv = InventoryReserve::new(1, 10);
        inv.execute().unwrap();
        inv.compensate().unwrap();

        assert!(
            inv.side_effects
                .iter()
                .any(|e| e.contains("RELEASED")),
            "Compensation should log the release"
        );
    }

    #[test]
    fn compensate_without_execute_is_noop() {
        let mut inv = InventoryReserve::new(1, 5);
        let result = inv.compensate();
        assert!(result.is_ok(), "Compensating without execute should be a no-op");
        assert!(!inv.released, "Should not be marked as released");
    }
}
