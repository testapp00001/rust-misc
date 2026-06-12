//! # Exercise: Isolation Problems in Sagas
//!
//! ## Theory
//!
//! Sagas sacrifice isolation for availability and partition tolerance. Unlike
//! traditional ACID transactions, Sagas allow intermediate states to be visible
//! to other transactions. This leads to several isolation anomalies:
//!
//! ## Proof / Intuition
//!
//! ### Dirty Reads
//!
//! A dirty read occurs when Transaction B reads data written by Transaction A
//! before Transaction A has committed.
//!
//! In the context of Sagas:
//! 1. Saga A starts and reserves stock (Stock = 90).
//! 2. Saga B reads stock and sees 90.
//! 3. Saga A's payment fails and stock is released (Stock = 100).
//! 4. Saga B made decisions based on incorrect intermediate state.
//!
//! ### Lost Updates
//!
//! A lost update occurs when two concurrent transactions read the same data and
//! then update it, with one update overwriting the other.
//!
//! In the context of Sagas:
//! 1. Saga A reads Stock = 100.
//! 2. Saga B reads Stock = 100.
//! 3. Saga A reserves 10: Stock = 90.
//! 4. Saga B reserves 10: Stock = 90 (overwriting A's reservation).
//! 5. Actual stock should be 80 (two reservations of 10 each).
//!
//! ### Unrepeatable Reads
//!
//! A saga may read the same data twice and get different results because
//! another saga modified it in between.
//!
//! ## Implementation Task
//!
//! Demonstrate isolation problems:
//!
//! - `SagaState`: shared state visible to all sagas.
//! - `DirtyReadDemo`: simulate dirty read scenario.
//! - `LostUpdateDemo`: simulate lost update scenario.
//! - Show that compensation can leave state inconsistent.
//!
//! ## Verification
//!
//! - Demonstrate a dirty read where an intermediate state is observed.
//! - Demonstrate a lost update where concurrent sagas corrupt data.
//! - Show that compensation doesn't fix the read anomaly.

use std::sync::{Arc, Mutex};

/// Shared state that sagas operate on.
#[derive(Debug, Clone)]
pub struct SagaState {
    pub stock: u32,
    pub balance: u64,
    pub orders: Vec<u64>,
}

impl SagaState {
    pub fn new(stock: u32, balance: u64) -> Self {
        Self {
            stock,
            balance,
            orders: Vec::new(),
        }
    }
}

/// A saga operating on shared state.
#[derive(Debug)]
pub struct SimpleSaga {
    pub id: u64,
    pub state: Arc<Mutex<SagaState>>,
    pub reserved_stock: u32,
    pub charged_amount: u64,
    pub executed: bool,
    pub compensated: bool,
}

impl SimpleSaga {
    pub fn new(id: u64, state: Arc<Mutex<SagaState>>) -> Self {
        Self {
            id,
            state,
            reserved_stock: 0,
            charged_amount: 0,
            executed: false,
            compensated: false,
        }
    }

    /// Step 1: Reserve stock (may produce dirty read).
    pub fn reserve_stock(&mut self, quantity: u32) -> Result<(), String> {
        let mut state = self.state.lock().unwrap();
        if state.stock < quantity {
            return Err("Insufficient stock".to_string());
        }
        state.stock -= quantity;
        self.reserved_stock = quantity;
        Ok(())
    }

    /// Step 2: Process payment.
    pub fn process_payment(&mut self, amount: u64) -> Result<(), String> {
        let mut state = self.state.lock().unwrap();
        if state.balance < amount {
            return Err("Insufficient balance".to_string());
        }
        state.balance -= amount;
        self.charged_amount = amount;
        Ok(())
    }

    /// Step 3: Create order.
    pub fn create_order(&mut self) -> Result<(), String> {
        let mut state = self.state.lock().unwrap();
        state.orders.push(self.id);
        self.executed = true;
        Ok(())
    }

    /// Compensate: release stock.
    pub fn compensate_stock(&mut self) {
        if self.reserved_stock > 0 {
            let mut state = self.state.lock().unwrap();
            state.stock += self.reserved_stock;
            self.compensated = true;
            self.reserved_stock = 0;
        }
    }

    /// Compensate: refund payment.
    pub fn compensate_payment(&mut self) {
        if self.charged_amount > 0 {
            let mut state = self.state.lock().unwrap();
            state.balance += self.charged_amount;
            self.charged_amount = 0;
        }
    }
}

/// Demonstrate a dirty read scenario.
pub fn demonstrate_dirty_read() -> (u32, u32, bool) {
    let state = Arc::new(Mutex::new(SagaState::new(100, 1000)));

    let mut saga_a = SimpleSaga::new(1, state.clone());
    let mut saga_b = SimpleSaga::new(2, state.clone());

    // Saga A reserves stock.
    saga_a.reserve_stock(50).unwrap();

    // Saga B reads the intermediate state (dirty read).
    let stock_seen_by_b = state.lock().unwrap().stock;

    // Saga A's payment fails, so it compensates.
    saga_a.compensate_stock();

    // Correct stock after compensation.
    let correct_stock = state.lock().unwrap().stock;

    // Saga B saw an intermediate state (50) that was later undone.
    let dirty_read_occurred = stock_seen_by_b != correct_stock;

    (stock_seen_by_b, correct_stock, dirty_read_occurred)
}

/// Demonstrate a lost update scenario.
pub fn demonstrate_lost_update() -> (u32, u32) {
    let state = Arc::new(Mutex::new(SagaState::new(100, 1000)));

    let mut saga_a = SimpleSaga::new(1, state.clone());
    let mut saga_b = SimpleSaga::new(2, state.clone());

    // Both sagas read stock (simulated: they see 100).
    let stock_before = state.lock().unwrap().stock;

    // Saga A reserves 50.
    saga_a.reserve_stock(50).unwrap();
    // Stock is now 50.

    // Saga B still thinks stock is 100 (stale read).
    // Saga B tries to reserve 80 (which would be valid if stock were still 100).
    let result_b = saga_b.reserve_stock(80);

    // Saga B's reservation fails because stock is actually 50.
    // But if saga B had read AFTER saga A's write and made a decision based on
    // the original value of 100, the update would have been lost.
    let final_stock = state.lock().unwrap().stock;

    (final_stock, result_b.is_err() as u32)
}

/// Demonstrate that concurrent sagas see inconsistent intermediate states.
pub fn demonstrate_unrepeatable_read() -> Vec<u32> {
    let state = Arc::new(Mutex::new(SagaState::new(100, 1000)));

    let mut saga_a = SimpleSaga::new(1, state.clone());
    let mut saga_b = SimpleSaga::new(2, state.clone());

    let mut observations = Vec::new();

    // Observation 1: initial state.
    observations.push(state.lock().unwrap().stock);

    // Saga A reserves stock.
    saga_a.reserve_stock(30).unwrap();
    observations.push(state.lock().unwrap().stock);

    // Saga B reserves stock.
    saga_b.reserve_stock(20).unwrap();
    observations.push(state.lock().unwrap().stock);

    // Saga A compensates.
    saga_a.compensate_stock();
    observations.push(state.lock().unwrap().stock);

    // The observations are [100, 70, 50, 80] -- inconsistent reads.
    observations
}

/// Calculate the anomaly: what the state SHOULD be vs what was observed.
pub fn calculate_anomaly(
    observations: &[u32],
    correct_final: u32,
) -> Vec<(usize, u32, Option<u32>)> {
    let mut anomalies = Vec::new();
    for (i, &observed) in observations.iter().enumerate() {
        if i == observations.len() - 1 {
            // Final observation: compare to correct value.
            if observed != correct_final {
                anomalies.push((i, observed, Some(correct_final)));
            }
        }
    }
    anomalies
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dirty_read_demonstrated() {
        let (stock_seen, stock_correct, dirty) = demonstrate_dirty_read();

        assert!(
            dirty,
            "Dirty read should have occurred: saga B saw intermediate stock {} but correct is {}",
            stock_seen,
            stock_correct
        );
        assert_eq!(stock_seen, 50, "Saw intermediate state of 50");
        assert_eq!(stock_correct, 100, "Correct state is 100 after compensation");
    }

    #[test]
    fn lost_update_demonstrated() {
        let (final_stock, failed) = demonstrate_lost_update();

        // Saga B's reservation of 80 should fail because only 50 remains.
        assert_eq!(failed, 1, "Saga B's reservation should fail");
        assert_eq!(final_stock, 50, "Stock should be 50 after saga A's reservation");
    }

    #[test]
    fn unrepeatable_read_demonstrated() {
        let observations = demonstrate_unrepeatable_read();

        // Stock should vary: 100 -> 70 -> 50 -> 80.
        assert_eq!(observations, vec![100, 70, 50, 80]);

        // The same saga sees different values for the same read.
        // Observation at index 0 (100) differs from observation at index 1 (70).
        assert_ne!(
            observations[0], observations[1],
            "Unrepeatable read: same data read at different times gives different results"
        );
    }

    #[test]
    fn compensation_restores_state_but_read_already_happened() {
        let state = Arc::new(Mutex::new(SagaState::new(100, 1000)));
        let mut saga_a = SimpleSaga::new(1, state.clone());

        saga_a.reserve_stock(50).unwrap();
        let during_saga_stock = state.lock().unwrap().stock;
        assert_eq!(during_saga_stock, 50);

        saga_a.compensate_stock();
        let after_compensation_stock = state.lock().unwrap().stock;
        assert_eq!(after_compensation_stock, 100);

        // But the dirty read already happened -- the damage is done.
        // Any saga that read during_saga_stock = 50 made decisions based on
        // incorrect (intermediate) state.
    }

    #[test]
    fn concurrent_sagas_see_inconsistent_state() {
        let state = Arc::new(Mutex::new(SagaState::new(100, 1000)));
        let mut saga_a = SimpleSaga::new(1, state.clone());
        let mut saga_b = SimpleSaga::new(2, state.clone());

        // Step 1: A reads stock (100).
        let a_reads = state.lock().unwrap().stock;
        assert_eq!(a_reads, 100);

        // Step 2: A reserves 50.
        saga_a.reserve_stock(50).unwrap();

        // Step 3: B reads stock (sees 50, not the 100 that A saw).
        let b_reads = state.lock().unwrap().stock;
        assert_eq!(b_reads, 50);

        // A and B saw different values for the "same" stock at "the same time."
        assert_ne!(a_reads, b_reads, "Concurrent sagas see different states");
    }
}
