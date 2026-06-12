//! # Exercise: CRDT Merge Properties
//!
//! ## Theory
//!
//! CRDTs must satisfy three algebraic properties for their merge function:
//!
//! 1. **Commutativity**: merge(a, b) = merge(b, a)
//!    The order of merging two states doesn't matter.
//!
//! 2. **Associativity**: merge(merge(a, b), c) = merge(a, merge(b, c))
//!    Grouping of merges doesn't matter.
//!
//! 3. **Idempotency**: merge(a, a) = a
//!    Merging a state with itself doesn't change it.
//!
//! Together, these form a join-semilattice and guarantee that all replicas
//! converge to the same state after receiving the same set of updates.
//!
//! ## Proof / Intuition
//!
//! These properties are necessary and sufficient for Strong Eventual
//! Consistency. If all replicas receive the same set of operations (in
//! any order), repeated merging will eventually produce identical states
//! regardless of the order operations were applied.
//!
//! ## Implementation Task
//!
//! Use property-based testing (proptest) to verify these properties hold
//! for all CRDT types: GCounter, PNCounter, GSet, ORSet, LWWRegister.
//!
//! ## Verification
//!
//! Run proptest to verify commutativity, associativity, and idempotency
//! with randomly generated inputs.
//!
#[cfg(test)]
mod tests {
    use crate::p01_g_counter::GCounter;
    use crate::p02_pn_counter::PNCounter;
    use crate::p03_g_set::GSet;
    use crate::p05_lww_register::LWWRegister;
    use proptest::prelude::*;

    // -- GCounter property tests --

    proptest! {
        #[test]
        fn gcounter_merge_commutative(
            a_ops in prop::collection::vec(0usize..5usize, 0..20),
            b_ops in prop::collection::vec(0usize..5usize, 0..20),
        ) {
            let mut a = GCounter::new();
            for op in &a_ops { a.increment(*op); }

            let mut b = GCounter::new();
            for op in &b_ops { b.increment(*op); }

            let mut ab = a.clone();
            ab.merge(&b);

            let mut ba = b.clone();
            ba.merge(&a);

            prop_assert_eq!(ab.value(), ba.value(), "GCounter merge must be commutative");
        }

        #[test]
        fn gcounter_merge_idempotent(
            ops in prop::collection::vec(0usize..5usize, 0..20),
        ) {
            let mut counter = GCounter::new();
            for op in &ops { counter.increment(*op); }

            let original = counter.clone();
            counter.merge(&original.clone());

            prop_assert_eq!(counter, original, "GCounter merge must be idempotent");
        }

        #[test]
        fn gcounter_merge_associative(
            a_ops in prop::collection::vec(0usize..5usize, 0..10),
            b_ops in prop::collection::vec(0usize..5usize, 0..10),
            c_ops in prop::collection::vec(0usize..5usize, 0..10),
        ) {
            let mut a = GCounter::new();
            for op in &a_ops { a.increment(*op); }
            let mut b = GCounter::new();
            for op in &b_ops { b.increment(*op); }
            let mut c = GCounter::new();
            for op in &c_ops { c.increment(*op); }

            let mut ab_then_c = a.clone();
            ab_then_c.merge(&b);
            ab_then_c.merge(&c);

            let mut a_then_bc = a;
            let mut bc = b.clone();
            bc.merge(&c);
            a_then_bc.merge(&bc);

            prop_assert_eq!(ab_then_c.value(), a_then_bc.value(), "GCounter merge must be associative");
        }
    }

    // -- PNCounter property tests --

    proptest! {
        #[test]
        fn pncounter_merge_commutative(
            a_incs in prop::collection::vec(0usize..5usize, 0..15),
            a_decs in prop::collection::vec(0usize..5usize, 0..15),
            b_incs in prop::collection::vec(0usize..5usize, 0..15),
            b_decs in prop::collection::vec(0usize..5usize, 0..15),
        ) {
            let mut a = PNCounter::new();
            for op in &a_incs { a.increment(*op); }
            for op in &a_decs { a.decrement(*op); }

            let mut b = PNCounter::new();
            for op in &b_incs { b.increment(*op); }
            for op in &b_decs { b.decrement(*op); }

            let mut ab = a.clone();
            ab.merge(&b);

            let mut ba = b.clone();
            ba.merge(&a);

            prop_assert_eq!(ab.value(), ba.value(), "PNCounter merge must be commutative");
        }

        #[test]
        fn pncounter_merge_idempotent(
            incs in prop::collection::vec(0usize..5usize, 0..15),
            decs in prop::collection::vec(0usize..5usize, 0..15),
        ) {
            let mut counter = PNCounter::new();
            for op in &incs { counter.increment(*op); }
            for op in &decs { counter.decrement(*op); }

            let original = counter.clone();
            counter.merge(&original.clone());

            prop_assert_eq!(counter.value(), original.value(), "PNCounter merge must be idempotent");
        }
    }

    // -- GSet property tests --

    proptest! {
        #[test]
        fn gset_merge_commutative(
            a_vals in prop::collection::vec(0i32..100, 0..20),
            b_vals in prop::collection::vec(0i32..100, 0..20),
        ) {
            let mut a = GSet::new();
            for v in &a_vals { a.add(*v); }

            let mut b = GSet::new();
            for v in &b_vals { b.add(*v); }

            let mut ab = a.clone();
            ab.merge(&b);

            let mut ba = b.clone();
            ba.merge(&a);

            prop_assert_eq!(ab.len(), ba.len(), "GSet merge must be commutative");
        }

        #[test]
        fn gset_merge_idempotent(
            vals in prop::collection::vec(0i32..100, 0..20),
        ) {
            let mut set = GSet::new();
            for v in &vals { set.add(*v); }

            let original = set.clone();
            set.merge(&original.clone());

            prop_assert_eq!(set.len(), original.len(), "GSet merge must be idempotent");
        }
    }

    // -- LWWRegister property tests --

    proptest! {
        #[test]
        fn lww_merge_commutative(
            a_val in 0i32..1000,
            a_ts in 0u64..10000,
            b_val in 0i32..1000,
            b_ts in 0u64..10000,
        ) {
            let a = LWWRegister::new(a_val, a_ts);
            let b = LWWRegister::new(b_val, b_ts);

            let mut ab = a.clone();
            ab.merge(&b);

            let mut ba = b.clone();
            ba.merge(&a);

            prop_assert_eq!(*ab.get(), *ba.get(), "LWW merge must be commutative");
        }

        #[test]
        fn lww_merge_idempotent(
            val in 0i32..1000,
            ts in 0u64..10000,
        ) {
            let mut reg = LWWRegister::new(val, ts);
            let original = reg.clone();
            reg.merge(&original.clone());

            prop_assert_eq!(*reg.get(), *original.get(), "LWW merge must be idempotent");
        }
    }
}
