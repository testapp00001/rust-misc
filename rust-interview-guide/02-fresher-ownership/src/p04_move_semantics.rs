/// Problem: Move Semantics
///
/// Master Rust's move semantics.
///
/// Key Concepts:
/// - Move vs copy
/// - When moves happen
/// - Preventing moves
/// - Partial moves
/// - Moves and pattern matching

/// Problem 1: Basic move
/// A value is moved when assigned to another variable
pub fn basic_move() -> String {
    let s1 = String::from("hello");
    let s2 = s1; // s1 is moved to s2
    s2
}

/// Problem 2: Copy instead of move
/// Stack data is copied, not moved
pub fn copy_example() -> (i32, i32) {
    let x = 5;
    let y = x; // x is copied (i32 implements Copy)
    (x, y)
}

/// Problem 3: Move in function call
/// Passing a value to a function moves it
pub fn take_string(s: String) -> String {
    s
}

pub fn move_to_function() -> String {
    let s = String::from("hello");
    take_string(s) // s is moved into take_string
}

/// Problem 4: Return moves ownership
/// Returning a value moves ownership to the caller
pub fn create_string() -> String {
    let s = String::from("hello");
    s // s is moved out of the function
}

/// Problem 5: Clone to prevent move
/// Use clone to create a deep copy
pub fn clone_example() -> (String, String) {
    let s1 = String::from("hello");
    let s2 = s1.clone(); // Deep copy
    (s1, s2) // Both are valid
}

/// Problem 6: Partial move
/// Move a field from a struct
pub fn partial_move() -> (String, i32) {
    let data = (String::from("hello"), 42);
    let (s, n) = data; // Destructure moves both
    (s, n)
}

/// Problem 7: Move in match
/// Pattern matching can move values
pub fn match_move() -> String {
    let x = Some(String::from("hello"));
    match x {
        Some(s) => s, // s is moved out of x
        None => String::new(),
    }
}

/// Problem 8: Move in loop
/// Each iteration moves the value
pub fn move_in_loop() -> Vec<String> {
    let mut result = Vec::new();
    for i in 0..3 {
        let s = format!("item {}", i);
        result.push(s); // s is moved into the vector
    }
    result
}

/// Problem 9: Move and Option
/// Option takes ownership of its contents
pub fn option_move() -> Option<String> {
    let s = Some(String::from("hello"));
    s // s is moved out
}

/// Problem 10: Move and Result
/// Result takes ownership of its contents
pub fn result_move() -> Result<String, String> {
    let s = Ok(String::from("hello"));
    s // s is moved out
}

/// Problem 11: Prevent move with reference
/// Use a reference to avoid moving
pub fn prevent_move() -> usize {
    let s = String::from("hello");
    let len = s.len(); // Borrow, don't move
    s.len() // s is still valid
}

/// Problem 12: Move in closure
/// Closures can capture by move
pub fn move_in_closure() -> impl FnOnce() -> String {
    let s = String::from("hello");
    move || s // Closure takes ownership of s
}

/// Problem 13: Move and ownership transfer
/// Transfer ownership through function calls
pub fn transfer_ownership() -> String {
    let s = String::from("hello");
    let s = add_world(s); // Ownership transferred
    s
}

fn add_world(mut s: String) -> String {
    s.push_str(" world");
    s
}

/// Problem 14: Move and Vec
/// Moving elements out of a Vec
pub fn move_from_vec() -> Vec<String> {
    let v = vec![
        String::from("hello"),
        String::from("world"),
    ];
    v.into_iter().collect() // into_iter moves elements
}

/// Problem 15: Move and pattern matching
/// Destructuring moves values
pub fn destructure_move() -> (String, String) {
    let pair = (String::from("hello"), String::from("world"));
    let (s1, s2) = pair; // Destructure moves both
    (s1, s2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_move() {
        let s = basic_move();
        assert_eq!(s, "hello");
    }

    #[test]
    fn test_copy_example() {
        let (x, y) = copy_example();
        assert_eq!(x, 5);
        assert_eq!(y, 5);
    }

    #[test]
    fn test_move_to_function() {
        let s = move_to_function();
        assert_eq!(s, "hello");
    }

    #[test]
    fn test_create_string() {
        let s = create_string();
        assert_eq!(s, "hello");
    }

    #[test]
    fn test_clone_example() {
        let (s1, s2) = clone_example();
        assert_eq!(s1, "hello");
        assert_eq!(s2, "hello");
    }

    #[test]
    fn test_partial_move() {
        let (s, n) = partial_move();
        assert_eq!(s, "hello");
        assert_eq!(n, 42);
    }

    #[test]
    fn test_match_move() {
        let s = match_move();
        assert_eq!(s, "hello");
    }

    #[test]
    fn test_move_in_loop() {
        let result = move_in_loop();
        assert_eq!(result, vec!["item 0", "item 1", "item 2"]);
    }

    #[test]
    fn test_option_move() {
        let s = option_move();
        assert_eq!(s, Some("hello".to_string()));
    }

    #[test]
    fn test_result_move() {
        let s = result_move();
        assert_eq!(s, Ok("hello".to_string()));
    }

    #[test]
    fn test_prevent_move() {
        assert_eq!(prevent_move(), 5);
    }

    #[test]
    fn test_move_in_closure() {
        let f = move_in_closure();
        assert_eq!(f(), "hello");
    }

    #[test]
    fn test_transfer_ownership() {
        let s = transfer_ownership();
        assert_eq!(s, "hello world");
    }

    #[test]
    fn test_move_from_vec() {
        let result = move_from_vec();
        assert_eq!(result, vec!["hello", "world"]);
    }

    #[test]
    fn test_destructure_move() {
        let (s1, s2) = destructure_move();
        assert_eq!(s1, "hello");
        assert_eq!(s2, "world");
    }
}
