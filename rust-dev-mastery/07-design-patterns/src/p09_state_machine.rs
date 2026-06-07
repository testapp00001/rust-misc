//! # State Machine Pattern
//!
//! Runtime state machines model systems with discrete states and transitions.
//! Unlike the typestate pattern (which encodes state in types at compile time),
//! runtime state machines use enums and match to handle transitions dynamically.
//!
//! ## Key Concepts
//! - **Enum-based states**: Each state is an enum variant
//! - **Transition tables**: Define valid state transitions
//! - **Guards**: Conditions that must be met for a transition
//! - **Actions**: Side effects that occur during transitions

use std::collections::HashMap;
use std::fmt;

/// A vending machine state machine.
#[derive(Debug, Clone, PartialEq)]
pub enum VendingState {
    Idle,
    HasCredit { amount: u32 },
    Dispensing { product: String },
    OutOfStock,
}

#[derive(Debug, Clone)]
pub enum VendingEvent {
    InsertCoin(u32),
    SelectProduct(String),
    DispenseComplete,
    Restock,
    Cancel,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VendingAction {
    AcceptCredit(u32),
    DispenseProduct(String),
    ReturnChange(u32),
    RejectEvent(String),
    DisplayMessage(String),
}

pub struct VendingMachine {
    state: VendingState,
    products: HashMap<String, (u32, u32)>, // name -> (price, quantity)
}

impl VendingMachine {
    pub fn new(products: Vec<(String, u32, u32)>) -> Self {
        let products: HashMap<String, (u32, u32)> =
            products.into_iter().map(|(n, p, q)| (n, (p, q))).collect();

        VendingMachine {
            state: if products.values().all(|(_, q)| *q == 0) {
                VendingState::OutOfStock
            } else {
                VendingState::Idle
            },
            products,
        }
    }

    pub fn handle(&mut self, event: VendingEvent) -> Vec<VendingAction> {
        let mut actions = Vec::new();

        match (&self.state, event) {
            (VendingState::Idle, VendingEvent::InsertCoin(amount)) => {
                actions.push(VendingAction::AcceptCredit(amount));
                actions.push(VendingAction::DisplayMessage(format!("Credit: {amount}")));
                self.state = VendingState::HasCredit { amount };
            }
            (VendingState::Idle, VendingEvent::SelectProduct(_)) => {
                actions.push(VendingAction::RejectEvent("Insert coins first".into()));
            }
            (VendingState::Idle, VendingEvent::Restock) => {
                // Restock handled in all states
                for (_, qty) in self.products.values_mut() {
                    *qty += 10;
                }
                actions.push(VendingAction::DisplayMessage("Restocked".into()));
            }

            (VendingState::HasCredit { amount }, VendingEvent::InsertCoin(extra)) => {
                let total = amount + extra;
                actions.push(VendingAction::AcceptCredit(extra));
                actions.push(VendingAction::DisplayMessage(format!("Credit: {total}")));
                self.state = VendingState::HasCredit { amount: total };
            }
            (VendingState::HasCredit { amount }, VendingEvent::SelectProduct(name)) => {
                if let Some(&(price, qty)) = self.products.get(&name) {
                    if qty == 0 {
                        actions.push(VendingAction::RejectEvent("Out of stock".into()));
                    } else if *amount < price {
                        actions
                            .push(VendingAction::RejectEvent(format!("Need {} more", price - amount)));
                    } else {
                        let change = amount - price;
                        if let Some(q) = self.products.get_mut(&name) {
                            q.1 -= 1;
                        }
                        actions.push(VendingAction::DispenseProduct(name.clone()));
                        actions.push(VendingAction::ReturnChange(change));
                        self.state = VendingState::Dispensing { product: name };
                    }
                } else {
                    actions.push(VendingAction::RejectEvent("Unknown product".into()));
                }
            }
            (VendingState::HasCredit { amount }, VendingEvent::Cancel) => {
                actions.push(VendingAction::ReturnChange(*amount));
                actions.push(VendingAction::DisplayMessage("Cancelled".into()));
                self.state = VendingState::Idle;
            }

            (VendingState::Dispensing { .. }, VendingEvent::DispenseComplete) => {
                if self.products.values().all(|(_, q)| *q == 0) {
                    self.state = VendingState::OutOfStock;
                } else {
                    self.state = VendingState::Idle;
                }
            }

            (VendingState::OutOfStock, VendingEvent::Restock) => {
                for (_, qty) in self.products.values_mut() {
                    *qty += 10;
                }
                actions.push(VendingAction::DisplayMessage("Restocked".into()));
                self.state = VendingState::Idle;
            }

            (_, event) => {
                actions.push(VendingAction::RejectEvent(format!(
                    "Invalid event in current state: {event:?}"
                )));
            }
        }

        actions
    }

    pub fn state(&self) -> &VendingState {
        &self.state
    }
}

/// A generic state machine with a transition table.
pub struct TransitionTable<S: Eq + std::hash::Hash + Clone, E: Eq + std::hash::Hash + Clone> {
    transitions: HashMap<(S, E), S>,
    current: S,
}

impl<S: Eq + std::hash::Hash + Clone, E: Eq + std::hash::Hash + Clone> TransitionTable<S, E> {
    pub fn new(initial: S) -> Self {
        TransitionTable {
            transitions: HashMap::new(),
            current: initial,
        }
    }

    pub fn add_transition(&mut self, from: S, event: E, to: S) {
        self.transitions.insert((from, event), to);
    }

    pub fn handle(&mut self, event: &E) -> Option<&S> {
        let key = (self.current.clone(), event.clone());
        if let Some(next) = self.transitions.get(&key) {
            self.current = next.clone();
            Some(&self.current)
        } else {
            None // Invalid transition
        }
    }

    pub fn current(&self) -> &S {
        &self.current
    }

    pub fn can_transition(&self, event: &E) -> bool {
        let key = (self.current.clone(), event.clone());
        self.transitions.contains_key(&key)
    }
}

/// A turnstile state machine — classic state machine example.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TurnstileState {
    Locked,
    Unlocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TurnstileEvent {
    Coin,
    Push,
}

pub struct Turnstile {
    table: TransitionTable<TurnstileState, TurnstileEvent>,
    pub push_count: u32,
}

impl Turnstile {
    pub fn new() -> Self {
        let mut table = TransitionTable::new(TurnstileState::Locked);
        table.add_transition(TurnstileState::Locked, TurnstileEvent::Coin, TurnstileState::Unlocked);
        table.add_transition(TurnstileState::Unlocked, TurnstileEvent::Push, TurnstileState::Locked);
        table.add_transition(TurnstileState::Locked, TurnstileEvent::Push, TurnstileState::Locked);
        table.add_transition(TurnstileState::Unlocked, TurnstileEvent::Coin, TurnstileState::Unlocked);

        Turnstile {
            table,
            push_count: 0,
        }
    }

    pub fn handle(&mut self, event: TurnstileEvent) -> TurnstileAction {
        let old_state = *self.table.current();
        if self.table.handle(&event).is_some() {
            match (old_state, event) {
                (TurnstileState::Locked, TurnstileEvent::Coin) => {
                    TurnstileAction::Unlock
                }
                (TurnstileState::Unlocked, TurnstileEvent::Push) => {
                    self.push_count += 1;
                    TurnstileAction::Lock
                }
                (TurnstileState::Locked, TurnstileEvent::Push) => {
                    TurnstileAction::Reject("Pushed without coin".into())
                }
                (TurnstileState::Unlocked, TurnstileEvent::Coin) => {
                    TurnstileAction::Reject("Already unlocked".into())
                }
                _ => TurnstileAction::Reject("Unknown".into()),
            }
        } else {
            TurnstileAction::Reject("Invalid transition".into())
        }
    }

    pub fn state(&self) -> &TurnstileState {
        self.table.current()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TurnstileAction {
    Unlock,
    Lock,
    Reject(String),
}

impl fmt::Display for TurnstileAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TurnstileAction::Unlock => write!(f, "Unlocked"),
            TurnstileAction::Lock => write!(f, "Locked"),
            TurnstileAction::Reject(msg) => write!(f, "Rejected: {msg}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vending_machine_basic() {
        let mut vm = VendingMachine::new(vec![
            ("Cola".into(), 150, 5),
            ("Water".into(), 100, 3),
        ]);

        let actions = vm.handle(VendingEvent::InsertCoin(100));
        assert!(actions.contains(&VendingAction::AcceptCredit(100)));

        let actions = vm.handle(VendingEvent::SelectProduct("Water".into()));
        assert!(actions.contains(&VendingAction::DispenseProduct("Water".into())));
        assert!(actions.contains(&VendingAction::ReturnChange(0)));
    }

    #[test]
    fn test_vending_machine_insufficient_credit() {
        let mut vm = VendingMachine::new(vec![("Cola".into(), 150, 5)]);

        vm.handle(VendingEvent::InsertCoin(100));
        let actions = vm.handle(VendingEvent::SelectProduct("Cola".into()));

        // Should reject with message about needing more
        assert!(actions.iter().any(|a| matches!(a,
            VendingAction::RejectEvent(msg) if msg.contains("more")
        )));
    }

    #[test]
    fn test_vending_machine_cancel() {
        let mut vm = VendingMachine::new(vec![("Cola".into(), 150, 5)]);

        vm.handle(VendingEvent::InsertCoin(100));
        let actions = vm.handle(VendingEvent::Cancel);
        assert!(actions.contains(&VendingAction::ReturnChange(100)));
        assert_eq!(*vm.state(), VendingState::Idle);
    }

    #[test]
    fn test_vending_machine_restock() {
        let mut vm = VendingMachine::new(vec![("Cola".into(), 150, 0)]);

        assert_eq!(*vm.state(), VendingState::OutOfStock);

        vm.handle(VendingEvent::Restock);
        assert_eq!(*vm.state(), VendingState::Idle);
    }

    #[test]
    fn test_transition_table() {
        let mut table = TransitionTable::new("start");
        table.add_transition("start", "go", "running");
        table.add_transition("running", "stop", "stopped");

        assert_eq!(table.current(), &"start");
        table.handle(&"go");
        assert_eq!(table.current(), &"running");
        table.handle(&"stop");
        assert_eq!(table.current(), &"stopped");
    }

    #[test]
    fn test_transition_table_invalid() {
        let mut table = TransitionTable::new("start");
        table.add_transition("start", "go", "running");

        let result = table.handle(&"invalid");
        assert!(result.is_none());
        assert_eq!(table.current(), &"start");
    }

    #[test]
    fn test_turnstile_normal_use() {
        let mut turnstile = Turnstile::new();

        // Push without coin — locked
        let action = turnstile.handle(TurnstileEvent::Push);
        assert_eq!(action, TurnstileAction::Reject("Pushed without coin".into()));
        assert_eq!(turnstile.state(), &TurnstileState::Locked);

        // Insert coin — unlocks
        let action = turnstile.handle(TurnstileEvent::Coin);
        assert_eq!(action, TurnstileAction::Unlock);
        assert_eq!(turnstile.state(), &TurnstileState::Unlocked);

        // Push — locks again
        let action = turnstile.handle(TurnstileEvent::Push);
        assert_eq!(action, TurnstileAction::Lock);
        assert_eq!(turnstile.state(), &TurnstileState::Locked);
        assert_eq!(turnstile.push_count, 1);
    }

    #[test]
    fn test_turnstile_double_coin() {
        let mut turnstile = Turnstile::new();

        turnstile.handle(TurnstileEvent::Coin);
        let action = turnstile.handle(TurnstileEvent::Coin);
        assert_eq!(action, TurnstileAction::Reject("Already unlocked".into()));
    }
}
