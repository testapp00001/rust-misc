//! # Unit Testing Fundamentals
//!
//! Rust's built-in test framework provides everything needed for unit testing.
//! Tests are functions annotated with `#[test]` that live in `#[cfg(test)]` modules.
//!
//! Key concepts:
//! - Test organization within the same file as production code
//! - Assertion macros: `assert!`, `assert_eq!`, `assert_ne!`
//! - Testing for panics with `#[should_panic]`
//! - Ignoring tests with `#[ignore]`
//! - Test return types (Result-based tests)

/// A simple calculator for demonstrating test patterns.
#[derive(Debug, Clone)]
pub struct Calculator {
    history: Vec<f64>,
}

impl Calculator {
    pub fn new() -> Self {
        Calculator {
            history: Vec::new(),
        }
    }

    pub fn add(&mut self, a: f64, b: f64) -> f64 {
        let result = a + b;
        self.history.push(result);
        result
    }

    pub fn subtract(&mut self, a: f64, b: f64) -> f64 {
        let result = a - b;
        self.history.push(result);
        result
    }

    pub fn multiply(&mut self, a: f64, b: f64) -> f64 {
        let result = a * b;
        self.history.push(result);
        result
    }

    pub fn divide(&mut self, a: f64, b: f64) -> Result<f64, CalcError> {
        if b == 0.0 {
            return Err(CalcError::DivisionByZero);
        }
        let result = a / b;
        self.history.push(result);
        Ok(result)
    }

    pub fn last_result(&self) -> Option<f64> {
        self.history.last().copied()
    }

    pub fn history(&self) -> &[f64] {
        &self.history
    }

    pub fn clear(&mut self) {
        self.history.clear();
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CalcError {
    DivisionByZero,
    Overflow,
}

/// Demonstrates testing with a stack data structure.
#[derive(Debug)]
pub struct Stack<T> {
    elements: Vec<T>,
}

impl<T> Stack<T> {
    pub fn new() -> Self {
        Stack {
            elements: Vec::new(),
        }
    }

    pub fn push(&mut self, value: T) {
        self.elements.push(value);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.elements.pop()
    }

    pub fn peek(&self) -> Option<&T> {
        self.elements.last()
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }

    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Basic assertion tests
    #[test]
    fn test_addition() {
        let mut calc = Calculator::new();
        assert_eq!(calc.add(2.0, 3.0), 5.0);
    }

    #[test]
    fn test_subtraction() {
        let mut calc = Calculator::new();
        assert_eq!(calc.subtract(10.0, 4.0), 6.0);
    }

    #[test]
    fn test_multiplication() {
        let mut calc = Calculator::new();
        assert_eq!(calc.multiply(3.0, 4.0), 12.0);
    }

    // Result-based tests
    #[test]
    fn test_division() -> Result<(), CalcError> {
        let mut calc = Calculator::new();
        let result = calc.divide(10.0, 2.0)?;
        assert_eq!(result, 5.0);
        Ok(())
    }

    // Testing for errors
    #[test]
    fn test_division_by_zero() {
        let mut calc = Calculator::new();
        let result = calc.divide(10.0, 0.0);
        assert_eq!(result, Err(CalcError::DivisionByZero));
    }

    // Testing history tracking
    #[test]
    fn test_history() {
        let mut calc = Calculator::new();
        calc.add(1.0, 2.0);
        calc.add(3.0, 4.0);
        calc.multiply(2.0, 3.0);

        assert_eq!(calc.history().len(), 3);
        assert_eq!(calc.last_result(), Some(6.0));
    }

    #[test]
    fn test_clear_history() {
        let mut calc = Calculator::new();
        calc.add(1.0, 2.0);
        calc.clear();
        assert!(calc.history().is_empty());
        assert_eq!(calc.last_result(), None);
    }

    // Stack tests
    #[test]
    fn test_stack_push_pop() {
        let mut stack = Stack::new();
        stack.push(1);
        stack.push(2);
        stack.push(3);

        assert_eq!(stack.pop(), Some(3));
        assert_eq!(stack.pop(), Some(2));
        assert_eq!(stack.pop(), Some(1));
        assert_eq!(stack.pop(), None);
    }

    #[test]
    fn test_stack_peek() {
        let mut stack = Stack::new();
        assert_eq!(stack.peek(), None);

        stack.push(42);
        assert_eq!(stack.peek(), Some(&42));
        assert_eq!(stack.len(), 1); // peek doesn't remove
    }

    #[test]
    fn test_stack_empty() {
        let stack = Stack::<i32>::new();
        assert!(stack.is_empty());
        assert_eq!(stack.len(), 0);
    }

    // Ignored test (for long-running or environment-specific tests)
    #[test]
    #[ignore = "requires database connection"]
    fn test_database_connection() {
        // This test would connect to a real database
        // Marked as ignored for regular test runs
    }

    // Test with custom failure message
    #[test]
    fn test_with_custom_message() {
        let calc = Calculator::new();
        assert!(
            calc.history().is_empty(),
            "New calculator should have empty history"
        );
    }

    // Testing floating point with tolerance
    #[test]
    fn test_floating_point_tolerance() {
        let mut calc = Calculator::new();
        let result = calc.divide(1.0, 3.0).unwrap();
        assert!(
            (result - 0.3333333).abs() < 1e-6,
            "1/3 should be approximately 0.3333333, got {result}"
        );
    }
}
