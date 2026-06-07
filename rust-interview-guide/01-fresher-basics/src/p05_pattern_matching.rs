/// Problem: Pattern Matching
///
/// Master Rust's powerful pattern matching system.
///
/// Key Concepts:
/// - match expressions
/// - if let and while let
/// - Destructuring structs and enums
/// - Guards and bindings
/// - @ patterns

/// Problem 1: Basic match
/// Match a number to its description
pub fn describe_number(n: i32) -> &'static str {
    match n {
        0 => "zero",
        1..=9 => "single digit",
        10..=99 => "double digit",
        _ => "large number",
    }
}

/// Problem 2: Match with Option
/// Unwrap an Option with a message
pub fn unwrap_with_message(value: Option<i32>) -> String {
    match value {
        Some(x) if x > 0 => format!("Positive: {}", x),
        Some(x) if x < 0 => format!("Negative: {}", x),
        Some(_) => "Zero".to_string(),
        None => "None".to_string(),
    }
}

/// Problem 3: Match with Result
/// Handle a Result
pub fn handle_result(result: Result<i32, String>) -> String {
    match result {
        Ok(x) if x > 0 => format!("Success: {}", x),
        Ok(x) => format!("Non-positive: {}", x),
        Err(e) => format!("Error: {}", e),
    }
}

/// Problem 4: Destructuring structs
/// Extract fields from a struct
#[derive(Debug)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

pub fn describe_point(point: &Point) -> String {
    let Point { x, y } = point;
    if *x == 0 && *y == 0 {
        "Origin".to_string()
    } else if *x == 0 {
        format!("On y-axis at {}", y)
    } else if *y == 0 {
        format!("On x-axis at {}", x)
    } else {
        format!("At ({}, {})", x, y)
    }
}

/// Problem 5: Destructuring enums
/// Extract data from an enum
#[derive(Debug)]
pub enum Message {
    Quit,
    Echo(String),
    Move { x: i32, y: i32 },
    Color(u8, u8, u8),
}

pub fn process_message(msg: &Message) -> String {
    match msg {
        Message::Quit => "Quitting".to_string(),
        Message::Echo(text) => format!("Echo: {}", text),
        Message::Move { x, y } => format!("Moving to ({}, {})", x, y),
        Message::Color(r, g, b) => format!("Color: #{:02x}{:02x}{:02x}", r, g, b),
    }
}

/// Problem 6: if let
/// Match only one pattern
pub fn get_first_word(s: &str) -> Option<&str> {
    if let Some(index) = s.find(' ') {
        Some(&s[..index])
    } else {
        Some(s)
    }
}

/// Problem 7: while let
/// Pop elements from a vector
pub fn pop_all<T>(mut v: Vec<T>) -> Vec<T> {
    let mut result = Vec::new();
    while let Some(item) = v.pop() {
        result.push(item);
    }
    result
}

/// Problem 8: Match with guards
/// Match with additional conditions
pub fn classify_temperature(temp: f64) -> &'static str {
    match temp {
        t if t < 0.0 => "freezing",
        t if t < 15.0 => "cold",
        t if t < 25.0 => "comfortable",
        t if t < 35.0 => "warm",
        _ => "hot",
    }
}

/// Problem 9: @ bindings
/// Bind a value while matching
pub fn describe_age(age: u32) -> String {
    match age {
        n @ 0..=12 => format!("Child: {} years old", n),
        n @ 13..=19 => format!("Teenager: {} years old", n),
        n @ 20..=64 => format!("Adult: {} years old", n),
        n => format!("Senior: {} years old", n),
    }
}

/// Problem 10: Match with tuples
/// Match multiple values
pub fn classify_point(x: i32, y: i32) -> &'static str {
    match (x, y) {
        (0, 0) => "origin",
        (0, _) => "y-axis",
        (_, 0) => "x-axis",
        (x, y) if x == y => "diagonal",
        _ => "other",
    }
}

/// Problem 11: Match with references
/// Match a reference
pub fn describe_ref(value: &Option<i32>) -> String {
    match value {
        Some(x) if *x > 0 => format!("Positive: {}", x),
        Some(x) if *x < 0 => format!("Negative: {}", x),
        Some(_) => "Zero".to_string(),
        None => "None".to_string(),
    }
}

/// Problem 12: Complex pattern matching
/// Match nested structures
#[derive(Debug)]
pub enum Expression {
    Number(f64),
    Add(Box<Expression>, Box<Expression>),
    Multiply(Box<Expression>, Box<Expression>),
}

pub fn evaluate(expr: &Expression) -> f64 {
    match expr {
        Expression::Number(n) => *n,
        Expression::Add(a, b) => evaluate(a) + evaluate(b),
        Expression::Multiply(a, b) => evaluate(a) * evaluate(b),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_describe_number() {
        assert_eq!(describe_number(0), "zero");
        assert_eq!(describe_number(5), "single digit");
        assert_eq!(describe_number(50), "double digit");
        assert_eq!(describe_number(100), "large number");
    }

    #[test]
    fn test_unwrap_with_message() {
        assert_eq!(unwrap_with_message(Some(5)), "Positive: 5");
        assert_eq!(unwrap_with_message(Some(-5)), "Negative: -5");
        assert_eq!(unwrap_with_message(Some(0)), "Zero");
        assert_eq!(unwrap_with_message(None), "None");
    }

    #[test]
    fn test_handle_result() {
        assert_eq!(handle_result(Ok(5)), "Success: 5");
        assert_eq!(handle_result(Ok(-5)), "Non-positive: -5");
        assert_eq!(
            handle_result(Err("error".to_string())),
            "Error: error"
        );
    }

    #[test]
    fn test_describe_point() {
        assert_eq!(describe_point(&Point { x: 0, y: 0 }), "Origin");
        assert_eq!(describe_point(&Point { x: 0, y: 5 }), "On y-axis at 5");
        assert_eq!(describe_point(&Point { x: 5, y: 0 }), "On x-axis at 5");
        assert_eq!(describe_point(&Point { x: 3, y: 4 }), "At (3, 4)");
    }

    #[test]
    fn test_process_message() {
        assert_eq!(process_message(&Message::Quit), "Quitting");
        assert_eq!(
            process_message(&Message::Echo("hello".to_string())),
            "Echo: hello"
        );
        assert_eq!(
            process_message(&Message::Move { x: 1, y: 2 }),
            "Moving to (1, 2)"
        );
        assert_eq!(
            process_message(&Message::Color(255, 0, 0)),
            "Color: #ff0000"
        );
    }

    #[test]
    fn test_get_first_word() {
        assert_eq!(get_first_word("hello world"), Some("hello"));
        assert_eq!(get_first_word("hello"), Some("hello"));
    }

    #[test]
    fn test_pop_all() {
        let v = vec![1, 2, 3];
        assert_eq!(pop_all(v), vec![3, 2, 1]);
    }

    #[test]
    fn test_classify_temperature() {
        assert_eq!(classify_temperature(-5.0), "freezing");
        assert_eq!(classify_temperature(10.0), "cold");
        assert_eq!(classify_temperature(20.0), "comfortable");
        assert_eq!(classify_temperature(30.0), "warm");
        assert_eq!(classify_temperature(40.0), "hot");
    }

    #[test]
    fn test_describe_age() {
        assert_eq!(describe_age(5), "Child: 5 years old");
        assert_eq!(describe_age(15), "Teenager: 15 years old");
        assert_eq!(describe_age(30), "Adult: 30 years old");
        assert_eq!(describe_age(70), "Senior: 70 years old");
    }

    #[test]
    fn test_classify_point() {
        assert_eq!(classify_point(0, 0), "origin");
        assert_eq!(classify_point(0, 5), "y-axis");
        assert_eq!(classify_point(5, 0), "x-axis");
        assert_eq!(classify_point(3, 3), "diagonal");
        assert_eq!(classify_point(3, 4), "other");
    }

    #[test]
    fn test_describe_ref() {
        assert_eq!(describe_ref(&Some(5)), "Positive: 5");
        assert_eq!(describe_ref(&Some(-5)), "Negative: -5");
        assert_eq!(describe_ref(&Some(0)), "Zero");
        assert_eq!(describe_ref(&None), "None");
    }

    #[test]
    fn test_evaluate() {
        // 2 + (3 * 4) = 14
        let expr = Expression::Add(
            Box::new(Expression::Number(2.0)),
            Box::new(Expression::Multiply(
                Box::new(Expression::Number(3.0)),
                Box::new(Expression::Number(4.0)),
            )),
        );
        assert!((evaluate(&expr) - 14.0).abs() < f64::EPSILON);
    }
}
