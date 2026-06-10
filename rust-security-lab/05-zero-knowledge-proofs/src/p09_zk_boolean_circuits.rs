//! # Lesson 09: ZK for Boolean Circuits
//!
//! ## From Protocols to Computation
//!
//! Everything we've learned so far proves specific statements (discrete log, set membership, etc.).
//! But what if you want to prove ANY computation is correct?
//!
//! The breakthrough idea: **compile any computation into a Boolean circuit, then prove
//! the circuit was evaluated correctly using ZK.**
//!
//! ## Boolean Circuits
//!
//! A Boolean circuit is a directed acyclic graph of logic gates:
//! - **AND**: output = a AND b
//! - **OR**: output = a OR b
//! - **NOT**: output = NOT a
//! - **XOR**: output = a XOR b
//!
//! Any computation can be expressed as a Boolean circuit. For example:
//! - Addition, multiplication
//! - Comparisons (is x > y?)
//! - Hash functions
//! - Signature verification
//!
//! ## How ZK for Circuits Works (Simplified)
//!
//! 1. **Arithmetization**: Convert the circuit into a system of polynomial equations
//! 2. **Commit**: Prover commits to all intermediate wire values
//! 3. **Verify gates**: For each gate, prove the output is correct given the inputs
//! 4. **Zero-knowledge**: Randomize all committed values so they reveal nothing
//!
//! ## Real-World Systems
//!
//! - **zk-SNARKs**: Convert computation to R1CS (Rank-1 Constraint System) → QAP → proof
//! - **zk-STARKs**: Convert to AIR (Algebraic Intermediate Representation) → polynomial IOP
//! - **libsnark, bellman, circom**: Libraries for building ZK circuits
//!
//! ## ATTACK: Why Not Just Re-execute?
//!
//! "Why not just run the computation yourself?"
//! - You might not have the inputs (they're private)
//! - The computation might be expensive (ZK proofs compress verification)
//! - The inputs might be secret (privacy)
//!
//! ## Example: Prove You Know a Preimage (Circuit Style)
//!
//! Circuit for SHA-256(x) == y:
//! - Inputs: x (private witness), y (public)
//! - Gates: SHA-256 circuit (~20,000 AND gates)
//! - Output: 1 if hash matches, 0 otherwise
//! - ZK proof: "I know x such that the circuit outputs 1"

/// Exercise 1: Represent a logic gate.
///
/// A gate has a type (AND, OR, NOT, XOR, INPUT) and references to input wires.
#[derive(Debug, Clone, PartialEq)]
pub enum GateType {
    And,
    Or,
    Not,
    Xor,
    Input(bool), // constant input value
}

#[derive(Debug, Clone)]
pub struct Gate {
    pub gate_type: GateType,
    pub inputs: Vec<usize>, // indices of input wires
}

/// Exercise 2: Create a gate.
///
/// Hints:
/// - Just construct a Gate struct
pub fn make_gate(gate_type: GateType, inputs: Vec<usize>) -> Gate {
    todo!("Create a logic gate")
}

/// Exercise 3: Evaluate a single gate given input values.
///
/// Hints:
/// - Match on gate_type:
///   - And: inputs[0] & inputs[1]
///   - Or: inputs[0] | inputs[1]
///   - Not: !inputs[0]
///   - Xor: inputs[0] ^ inputs[1]
///   - Input(v): v
pub fn evaluate_gate(gate: &Gate, wire_values: &[bool]) -> bool {
    todo!("Evaluate a single logic gate")
}

/// Exercise 4: Evaluate an entire circuit.
///
/// A circuit is a list of gates in topological order (inputs first).
/// Evaluate each gate and store the result in wire_values.
///
/// Hints:
/// - Create a Vec<bool> for wire values
/// - For each gate, look up its input wire values and evaluate
/// - Push the result to wire_values
pub fn evaluate_circuit(gates: &[Gate]) -> Vec<bool> {
    todo!("Evaluate entire circuit")
}

/// Exercise 5: Build a simple circuit: AND gate.
///
/// Wire 0: Input(a)
/// Wire 1: Input(b)
/// Wire 2: AND(wire 0, wire 1)
///
/// Returns the gates.
pub fn and_circuit(a: bool, b: bool) -> Vec<Gate> {
    todo!("Build AND gate circuit")
}

/// Exercise 6: Build a more complex circuit: 2-bit equality check.
///
/// Check if (a0, a1) == (b0, b1):
/// - xnor0 = NOT(XOR(a0, b0))
/// - xnor1 = NOT(XOR(a1, b1))
/// - equal = AND(xnor0, xnor1)
///
/// Wires: a0(0), b0(1), a1(2), b1(3), xor0(4), xnor0(5), xor1(6), xnor1(7), equal(8)
///
/// Hints:
/// - Build gates in topological order
/// - Use Input gates for the 4 inputs
/// - XOR, NOT, AND gates for the logic
pub fn equality_circuit(a0: bool, a1: bool, b0: bool, b1: bool) -> Vec<Gate> {
    todo!("Build 2-bit equality check circuit")
}

/// Exercise 7: Build a circuit for XOR of two values (simplified hash component).
///
/// This shows how hash functions are built from circuits.
///
/// Wire 0: Input(a)
/// Wire 1: Input(b)
/// Wire 2: XOR(wire 0, wire 1)
pub fn xor_circuit(a: bool, b: bool) -> Vec<Gate> {
    todo!("Build XOR circuit")
}

/// Exercise 8: Demonstrate that circuit evaluation is deterministic.
///
/// Same inputs should always produce same outputs.
///
/// Hints:
/// - Build a circuit
/// - Evaluate twice
/// - Compare outputs
pub fn demonstrate_determinism(a: bool, b: bool) -> bool {
    todo!("Show circuit evaluation is deterministic")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gate_creation() {
        let gate = make_gate(GateType::And, vec![0, 1]);
        assert_eq!(gate.gate_type, GateType::And);
        assert_eq!(gate.inputs, vec![0, 1]);
    }

    #[test]
    fn test_evaluate_and_gate() {
        let wire_values = vec![true, true];
        let gate = Gate { gate_type: GateType::And, inputs: vec![0, 1] };
        assert!(evaluate_gate(&gate, &wire_values));
    }

    #[test]
    fn test_evaluate_and_gate_false() {
        let wire_values = vec![true, false];
        let gate = Gate { gate_type: GateType::And, inputs: vec![0, 1] };
        assert!(!evaluate_gate(&gate, &wire_values));
    }

    #[test]
    fn test_evaluate_not_gate() {
        let wire_values = vec![true];
        let gate = Gate { gate_type: GateType::Not, inputs: vec![0] };
        assert!(!evaluate_gate(&gate, &wire_values));
    }

    #[test]
    fn test_and_circuit() {
        let gates = and_circuit(true, true);
        let values = evaluate_circuit(&gates);
        assert!(values[2], "AND(true, true) should be true");
    }

    #[test]
    fn test_and_circuit_false() {
        let gates = and_circuit(true, false);
        let values = evaluate_circuit(&gates);
        assert!(!values[2], "AND(true, false) should be false");
    }

    #[test]
    fn test_equality_circuit() {
        // (1, 0) == (1, 0) => true
        let gates = equality_circuit(true, false, true, false);
        let values = evaluate_circuit(&gates);
        assert!(values[values.len() - 1]);
    }

    #[test]
    fn test_equality_circuit_not_equal() {
        // (1, 0) != (0, 1) => false
        let gates = equality_circuit(true, false, false, true);
        let values = evaluate_circuit(&gates);
        assert!(!values[values.len() - 1]);
    }

    #[test]
    fn test_xor_circuit() {
        let gates = xor_circuit(true, false);
        let values = evaluate_circuit(&gates);
        assert!(values[2], "XOR(true, false) should be true");
    }

    #[test]
    fn test_determinism() {
        assert!(demonstrate_determinism(true, true));
        assert!(demonstrate_determinism(false, true));
    }
}
