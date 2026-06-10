//! # Lesson 09: ZK for Boolean Circuits (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

#[derive(Debug, Clone, PartialEq)]
pub enum GateType {
    And,
    Or,
    Not,
    Xor,
    Input(bool),
}

#[derive(Debug, Clone)]
pub struct Gate {
    pub gate_type: GateType,
    pub inputs: Vec<usize>,
}

pub fn make_gate(gate_type: GateType, inputs: Vec<usize>) -> Gate {
    Gate { gate_type, inputs }
}

/// Evaluate a single gate given wire values.
pub fn evaluate_gate(gate: &Gate, wire_values: &[bool]) -> bool {
    match &gate.gate_type {
        GateType::And => wire_values[gate.inputs[0]] & wire_values[gate.inputs[1]],
        GateType::Or => wire_values[gate.inputs[0]] | wire_values[gate.inputs[1]],
        GateType::Not => !wire_values[gate.inputs[0]],
        GateType::Xor => wire_values[gate.inputs[0]] ^ wire_values[gate.inputs[1]],
        GateType::Input(v) => *v,
    }
}

/// Evaluate an entire circuit (gates in topological order).
pub fn evaluate_circuit(gates: &[Gate]) -> Vec<bool> {
    let mut wire_values = Vec::with_capacity(gates.len());
    for gate in gates {
        let value = evaluate_gate(gate, &wire_values);
        wire_values.push(value);
    }
    wire_values
}

/// Build an AND gate circuit.
pub fn and_circuit(a: bool, b: bool) -> Vec<Gate> {
    vec![
        Gate { gate_type: GateType::Input(a), inputs: vec![] },
        Gate { gate_type: GateType::Input(b), inputs: vec![] },
        Gate { gate_type: GateType::And, inputs: vec![0, 1] },
    ]
}

/// Build a 2-bit equality check circuit.
pub fn equality_circuit(a0: bool, a1: bool, b0: bool, b1: bool) -> Vec<Gate> {
    vec![
        // 0: a0
        Gate { gate_type: GateType::Input(a0), inputs: vec![] },
        // 1: b0
        Gate { gate_type: GateType::Input(b0), inputs: vec![] },
        // 2: a1
        Gate { gate_type: GateType::Input(a1), inputs: vec![] },
        // 3: b1
        Gate { gate_type: GateType::Input(b1), inputs: vec![] },
        // 4: xor0 = XOR(a0, b0)
        Gate { gate_type: GateType::Xor, inputs: vec![0, 1] },
        // 5: xnor0 = NOT(xor0)
        Gate { gate_type: GateType::Not, inputs: vec![4] },
        // 6: xor1 = XOR(a1, b1)
        Gate { gate_type: GateType::Xor, inputs: vec![2, 3] },
        // 7: xnor1 = NOT(xor1)
        Gate { gate_type: GateType::Not, inputs: vec![6] },
        // 8: equal = AND(xnor0, xnor1)
        Gate { gate_type: GateType::And, inputs: vec![5, 7] },
    ]
}

/// Build a XOR circuit.
pub fn xor_circuit(a: bool, b: bool) -> Vec<Gate> {
    vec![
        Gate { gate_type: GateType::Input(a), inputs: vec![] },
        Gate { gate_type: GateType::Input(b), inputs: vec![] },
        Gate { gate_type: GateType::Xor, inputs: vec![0, 1] },
    ]
}

/// Demonstrate determinism: same inputs always produce same outputs.
pub fn demonstrate_determinism(a: bool, b: bool) -> bool {
    let gates1 = and_circuit(a, b);
    let gates2 = and_circuit(a, b);
    let values1 = evaluate_circuit(&gates1);
    let values2 = evaluate_circuit(&gates2);
    values1 == values2
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
        let gates = equality_circuit(true, false, true, false);
        let values = evaluate_circuit(&gates);
        assert!(values[values.len() - 1]);
    }

    #[test]
    fn test_equality_circuit_not_equal() {
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
