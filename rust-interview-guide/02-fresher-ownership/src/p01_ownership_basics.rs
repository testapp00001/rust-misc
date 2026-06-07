/// Problem: Ownership Basics
///
/// Understand Rust's ownership system.
///
/// Key Concepts:
/// - Each value has one owner
/// - When owner goes out of scope, value is dropped
/// - Move semantics for heap data

/// Problem 1: Move semantics
/// This code moves the string from s1 to s2
pub fn move_example() -> String {
    let s1 = String::from("hello");
    let s2 = s1; // s1 is moved to s2
    s2 // s1 is no longer valid
}

/// Problem 2: Clone to keep both
/// Use clone to create a deep copy
pub fn clone_example() -> (String, String) {
    let s1 = String::from("hello");
    let s2 = s1.clone(); // Deep copy
    (s1, s2) // Both are valid
}

/// Problem 3: Copy trait
/// Stack data is copied automatically
pub fn copy_example() -> (i32, i32) {
    let x = 5;
    let y = x; // x is copied (i32 implements Copy)
    (x, y) // Both are valid
}

/// Problem 4: Function takes ownership
/// When you pass a value to a function, it's moved
pub fn take_ownership(s: String) -> String {
    s // Return ownership back
}

/// Problem 5: Function borrows
/// Use a reference to borrow without taking ownership
pub fn calculate_length(s: &String) -> usize {
    s.len()
}

/// Problem 6: Multiple ownership with Rc
/// Use Rc for multiple owners
pub fn multiple_ownership() -> (std::rc::Rc<String>, std::rc::Rc<String>) {
    use std::rc::Rc;
    let s = Rc::new(String::from("hello"));
    let s2 = Rc::clone(&s);
    (s, s2) // Both are valid
}

/// Problem 7: Drop scope
/// Value is dropped when owner goes out of scope
pub fn drop_example() -> String {
    let s = String::from("hello");
    // s is valid here
    s // s is returned, so it's not dropped
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

/// Problem 9: Partial move
/// Move a field from a struct
pub fn partial_move() -> (String, i32) {
    let data = (String::from("hello"), 42);
    let (s, n) = data; // Destructure moves both
    (s, n)
}

/// Problem 10: Ownership and Option
/// Option takes ownership of its contents
pub fn option_ownership() -> Option<String> {
    let s = Some(String::from("hello"));
    s // s is moved out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_example() {
        let s = move_example();
        assert_eq!(s, "hello");
    }

    #[test]
    fn test_clone_example() {
        let (s1, s2) = clone_example();
        assert_eq!(s1, "hello");
        assert_eq!(s2, "hello");
    }

    #[test]
    fn test_copy_example() {
        let (x, y) = copy_example();
        assert_eq!(x, 5);
        assert_eq!(y, 5);
    }

    #[test]
    fn test_take_ownership() {
        let s = String::from("hello");
        let s = take_ownership(s);
        assert_eq!(s, "hello");
    }

    #[test]
    fn test_calculate_length() {
        let s = String::from("hello");
        let len = calculate_length(&s);
        assert_eq!(len, 5);
    }

    #[test]
    fn test_multiple_ownership() {
        let (s1, s2) = multiple_ownership();
        assert_eq!(*s1, "hello");
        assert_eq!(*s2, "hello");
    }

    #[test]
    fn test_drop_example() {
        let s = drop_example();
        assert_eq!(s, "hello");
    }

    #[test]
    fn test_move_in_loop() {
        let result = move_in_loop();
        assert_eq!(result, vec!["item 0", "item 1", "item 2"]);
    }

    #[test]
    fn test_partial_move() {
        let (s, n) = partial_move();
        assert_eq!(s, "hello");
        assert_eq!(n, 42);
    }

    #[test]
    fn test_option_ownership() {
        let s = option_ownership();
        assert_eq!(s, Some("hello".to_string()));
    }
}
