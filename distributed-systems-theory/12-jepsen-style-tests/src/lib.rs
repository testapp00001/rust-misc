//! # Module 12: Jepsen-Style Tests
//!
//! A collection of Jepsen-style correctness tests for distributed systems.
//! Each submodule implements a different consistency model checker or fault
//! injector used to verify that distributed systems uphold their claims under
//! adverse conditions.

pub mod causal_consistency;
pub mod eventual_convergence;
pub mod linearizability;
pub mod network_chaos;
pub mod partition_tolerance;
pub mod sequential_consistency;
