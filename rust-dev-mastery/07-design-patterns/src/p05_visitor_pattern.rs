//! # Visitor Pattern
//!
//! The visitor pattern separates algorithms from data structures. In Rust,
//! it's implemented using enums with match (enum dispatch) or trait objects
//! with accept/visit methods (double dispatch).
//!
//! ## Key Concepts
//! - **Enum dispatch**: Match on enum variants (idiomatic Rust)
//! - **Trait-based visitor**: For open extension of operations
//! - **Double dispatch**: Visitor and element types both participate in dispatch

/// An AST (Abstract Syntax Tree) for a simple expression language.
/// Demonstrates enum-based visitor pattern.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(f64),
    Add(Box<Expr>, Box<Expr>),
    Multiply(Box<Expr>, Box<Expr>),
    Negate(Box<Expr>),
    Variable(String),
}

impl Expr {
    pub fn literal(value: f64) -> Self {
        Expr::Literal(value)
    }

    pub fn add(left: Expr, right: Expr) -> Self {
        Expr::Add(Box::new(left), Box::new(right))
    }

    pub fn multiply(left: Expr, right: Expr) -> Self {
        Expr::Multiply(Box::new(left), Box::new(right))
    }

    pub fn negate(expr: Expr) -> Self {
        Expr::Negate(Box::new(expr))
    }

    pub fn variable(name: impl Into<String>) -> Self {
        Expr::Variable(name.into())
    }
}

/// Visitor trait for expressions. Each method handles one variant.
pub trait ExprVisitor<T> {
    fn visit_literal(&mut self, value: f64) -> T;
    fn visit_add(&mut self, left: &Expr, right: &Expr) -> T;
    fn visit_multiply(&mut self, left: &Expr, right: &Expr) -> T;
    fn visit_negate(&mut self, expr: &Expr) -> T;
    fn visit_variable(&mut self, name: &str) -> T;
}

impl Expr {
    /// Accept a visitor, implementing double dispatch.
    pub fn accept<T>(&self, visitor: &mut dyn ExprVisitor<T>) -> T {
        match self {
            Expr::Literal(value) => visitor.visit_literal(*value),
            Expr::Add(left, right) => visitor.visit_add(left, right),
            Expr::Multiply(left, right) => visitor.visit_multiply(left, right),
            Expr::Negate(expr) => visitor.visit_negate(expr),
            Expr::Variable(name) => visitor.visit_variable(name),
        }
    }
}

/// Evaluates an expression to a numeric value.
pub struct Evaluator {
    variables: std::collections::HashMap<String, f64>,
}

impl Evaluator {
    pub fn new() -> Self {
        Evaluator {
            variables: std::collections::HashMap::new(),
        }
    }

    pub fn set_variable(&mut self, name: impl Into<String>, value: f64) {
        self.variables.insert(name.into(), value);
    }

    pub fn evaluate(&mut self, expr: &Expr) -> f64 {
        expr.accept(self)
    }
}

impl ExprVisitor<f64> for Evaluator {
    fn visit_literal(&mut self, value: f64) -> f64 {
        value
    }

    fn visit_add(&mut self, left: &Expr, right: &Expr) -> f64 {
        left.accept(self) + right.accept(self)
    }

    fn visit_multiply(&mut self, left: &Expr, right: &Expr) -> f64 {
        left.accept(self) * right.accept(self)
    }

    fn visit_negate(&mut self, expr: &Expr) -> f64 {
        -expr.accept(self)
    }

    fn visit_variable(&mut self, name: &str) -> f64 {
        self.variables.get(name).copied().unwrap_or(0.0)
    }
}

/// Converts an expression to its string representation.
pub struct Printer;

impl ExprVisitor<String> for Printer {
    fn visit_literal(&mut self, value: f64) -> String {
        format!("{value}")
    }

    fn visit_add(&mut self, left: &Expr, right: &Expr) -> String {
        format!("({} + {})", left.accept(self), right.accept(self))
    }

    fn visit_multiply(&mut self, left: &Expr, right: &Expr) -> String {
        format!("({} * {})", left.accept(self), right.accept(self))
    }

    fn visit_negate(&mut self, expr: &Expr) -> String {
        format!("(-{})", expr.accept(self))
    }

    fn visit_variable(&mut self, name: &str) -> String {
        name.to_string()
    }
}

/// Counts the number of nodes in an expression tree.
pub struct NodeCounter;

impl ExprVisitor<usize> for NodeCounter {
    fn visit_literal(&mut self, _: f64) -> usize {
        1
    }

    fn visit_add(&mut self, left: &Expr, right: &Expr) -> usize {
        1 + left.accept(self) + right.accept(self)
    }

    fn visit_multiply(&mut self, left: &Expr, right: &Expr) -> usize {
        1 + left.accept(self) + right.accept(self)
    }

    fn visit_negate(&mut self, expr: &Expr) -> usize {
        1 + expr.accept(self)
    }

    fn visit_variable(&mut self, _: &str) -> usize {
        1
    }
}

/// Collects all variable names used in an expression.
pub struct VariableCollector {
    variables: Vec<String>,
}

impl VariableCollector {
    pub fn new() -> Self {
        VariableCollector { variables: Vec::new() }
    }

    pub fn collect(&mut self, expr: &Expr) -> Vec<String> {
        self.variables.clear();
        expr.accept(self);
        self.variables.clone()
    }
}

impl ExprVisitor<()> for VariableCollector {
    fn visit_literal(&mut self, _: f64) {}
    fn visit_add(&mut self, left: &Expr, right: &Expr) {
        left.accept(self);
        right.accept(self);
    }
    fn visit_multiply(&mut self, left: &Expr, right: &Expr) {
        left.accept(self);
        right.accept(self);
    }
    fn visit_negate(&mut self, expr: &Expr) {
        expr.accept(self);
    }
    fn visit_variable(&mut self, name: &str) {
        if !self.variables.contains(&name.to_string()) {
            self.variables.push(name.to_string());
        }
    }
}

/// Simplified visitor using a closure (more ergonomic for simple operations).
pub fn fold_expr<T>(expr: &Expr, init: T, f: &mut impl FnMut(T, &Expr) -> T) -> T {
    let acc = f(init, expr);
    match expr {
        Expr::Literal(_) | Expr::Variable(_) => acc,
        Expr::Add(l, r) | Expr::Multiply(l, r) => {
            let acc = fold_expr(l, acc, f);
            fold_expr(r, acc, f)
        }
        Expr::Negate(e) => fold_expr(e, acc, f),
    }
}

/// A JSON-like value with a visitor for processing.
#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(Vec<(String, JsonValue)>),
}

pub trait JsonVisitor<T> {
    fn visit_null(&mut self) -> T;
    fn visit_bool(&mut self, value: bool) -> T;
    fn visit_number(&mut self, value: f64) -> T;
    fn visit_string(&mut self, value: &str) -> T;
    fn visit_array(&mut self, elements: &[JsonValue]) -> T;
    fn visit_object(&mut self, entries: &[(String, JsonValue)]) -> T;
}

impl JsonValue {
    pub fn accept<T>(&self, visitor: &mut dyn JsonVisitor<T>) -> T {
        match self {
            JsonValue::Null => visitor.visit_null(),
            JsonValue::Bool(b) => visitor.visit_bool(*b),
            JsonValue::Number(n) => visitor.visit_number(*n),
            JsonValue::String(s) => visitor.visit_string(s),
            JsonValue::Array(arr) => visitor.visit_array(arr),
            JsonValue::Object(obj) => visitor.visit_object(obj),
        }
    }
}

/// Counts the total number of JSON values (including nested).
pub struct JsonCounter;

impl JsonVisitor<usize> for JsonCounter {
    fn visit_null(&mut self) -> usize {
        1
    }
    fn visit_bool(&mut self, _: bool) -> usize {
        1
    }
    fn visit_number(&mut self, _: f64) -> usize {
        1
    }
    fn visit_string(&mut self, _: &str) -> usize {
        1
    }
    fn visit_array(&mut self, elements: &[JsonValue]) -> usize {
        1 + elements.iter().map(|e| e.accept(self)).sum::<usize>()
    }
    fn visit_object(&mut self, entries: &[(String, JsonValue)]) -> usize {
        1 + entries.iter().map(|(_, v)| v.accept(self)).sum::<usize>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluator_literal() {
        let mut eval = Evaluator::new();
        assert_eq!(eval.evaluate(&Expr::literal(42.0)), 42.0);
    }

    #[test]
    fn test_evaluator_add() {
        let mut eval = Evaluator::new();
        let expr = Expr::add(Expr::literal(3.0), Expr::literal(4.0));
        assert_eq!(eval.evaluate(&expr), 7.0);
    }

    #[test]
    fn test_evaluator_complex() {
        let mut eval = Evaluator::new();
        // (2 + 3) * 4
        let expr = Expr::multiply(
            Expr::add(Expr::literal(2.0), Expr::literal(3.0)),
            Expr::literal(4.0),
        );
        assert_eq!(eval.evaluate(&expr), 20.0);
    }

    #[test]
    fn test_evaluator_variables() {
        let mut eval = Evaluator::new();
        eval.set_variable("x", 10.0);
        eval.set_variable("y", 20.0);

        let expr = Expr::add(Expr::variable("x"), Expr::variable("y"));
        assert_eq!(eval.evaluate(&expr), 30.0);
    }

    #[test]
    fn test_evaluator_negate() {
        let mut eval = Evaluator::new();
        let expr = Expr::negate(Expr::literal(5.0));
        assert_eq!(eval.evaluate(&expr), -5.0);
    }

    #[test]
    fn test_printer() {
        let mut printer = Printer;
        let expr = Expr::add(Expr::literal(1.0), Expr::literal(2.0));
        assert_eq!(expr.accept(&mut printer), "(1 + 2)");
    }

    #[test]
    fn test_printer_complex() {
        let mut printer = Printer;
        let expr = Expr::multiply(
            Expr::variable("x"),
            Expr::negate(Expr::literal(3.0)),
        );
        assert_eq!(expr.accept(&mut printer), "(x * (-3))");
    }

    #[test]
    fn test_node_counter() {
        let mut counter = NodeCounter;
        let expr = Expr::add(Expr::literal(1.0), Expr::literal(2.0));
        assert_eq!(expr.accept(&mut counter), 3); // add + 2 literals
    }

    #[test]
    fn test_variable_collector() {
        let mut collector = VariableCollector::new();
        let expr = Expr::add(
            Expr::variable("x"),
            Expr::multiply(Expr::variable("y"), Expr::variable("x")),
        );
        let vars = collector.collect(&expr);
        assert_eq!(vars, vec!["x", "y"]); // x appears twice but collected once
    }

    #[test]
    fn test_fold_expr() {
        let expr = Expr::add(Expr::literal(1.0), Expr::literal(2.0));
        let count = fold_expr(&expr, 0, &mut |acc, _| acc + 1);
        assert_eq!(count, 3);
    }

    #[test]
    fn test_json_counter() {
        let json = JsonValue::Array(vec![
            JsonValue::Number(1.0),
            JsonValue::Object(vec![
                ("key".into(), JsonValue::String("value".into())),
                ("flag".into(), JsonValue::Bool(true)),
            ]),
            JsonValue::Null,
        ]);

        let count = json.accept(&mut JsonCounter);
        assert_eq!(count, 6); // array + number + object + string + bool + null
    }
}
