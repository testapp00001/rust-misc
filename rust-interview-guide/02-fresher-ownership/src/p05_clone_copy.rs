/// Problem: Clone and Copy
///
/// Master Rust's Clone and Copy traits.
///
/// Key Concepts:
/// - Copy trait for stack data
/// - Clone trait for deep copies
/// - When to use each
/// - Implementing Copy and Clone
/// - Performance implications

/// Problem 1: Copy trait
/// Types that implement Copy are copied automatically
pub fn copy_example() -> (i32, i32) {
    let x = 5;
    let y = x; // x is copied (i32 implements Copy)
    (x, y) // Both are valid
}

/// Problem 2: Clone trait
/// Use clone for explicit deep copies
pub fn clone_example() -> (String, String) {
    let s1 = String::from("hello");
    let s2 = s1.clone(); // Deep copy
    (s1, s2) // Both are valid
}

/// Problem 3: Copy vs Clone
/// Compare Copy and Clone behavior
pub fn copy_vs_clone() -> (i32, i32, String, String) {
    let x = 5;
    let y = x; // Copy

    let s1 = String::from("hello");
    let s2 = s1.clone(); // Clone

    (x, y, s1, s2)
}

/// Problem 4: Deriving Copy and Clone
/// Use derive macro for automatic implementation
#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

pub fn point_copy() -> (Point, Point) {
    let p1 = Point { x: 1.0, y: 2.0 };
    let p2 = p1; // Copy
    (p1, p2) // Both are valid
}

/// Problem 5: Clone for complex types
/// Clone a Vec of Strings
pub fn clone_vec() -> (Vec<String>, Vec<String>) {
    let v1 = vec![
        String::from("hello"),
        String::from("world"),
    ];
    let v2 = v1.clone(); // Deep copy of Vec and all Strings
    (v1, v2)
}

/// Problem 6: Copy for arrays
/// Arrays of Copy types are also Copy
pub fn copy_array() -> ([i32; 3], [i32; 3]) {
    let a1 = [1, 2, 3];
    let a2 = a1; // Copy
    (a1, a2) // Both are valid
}

/// Problem 7: Clone with Option
/// Clone an Option
pub fn clone_option() -> (Option<String>, Option<String>) {
    let o1 = Some(String::from("hello"));
    let o2 = o1.clone(); // Deep copy
    (o1, o2)
}

/// Problem 8: Clone with Result
/// Clone a Result
pub fn clone_result() -> (Result<String, String>, Result<String, String>) {
    let r1 = Ok(String::from("hello"));
    let r2 = r1.clone(); // Deep copy
    (r1, r2)
}

/// Problem 9: Copy with references
/// References are Copy
pub fn copy_reference() -> (&'static str, &'static str) {
    let s1 = "hello";
    let s2 = s1; // Copy (references are Copy)
    (s1, s2)
}

/// Problem 10: Clone with HashMap
/// Clone a HashMap
pub fn clone_hashmap() -> (std::collections::HashMap<String, i32>, std::collections::HashMap<String, i32>) {
    let mut m1 = std::collections::HashMap::new();
    m1.insert("key".to_string(), 42);
    let m2 = m1.clone(); // Deep copy
    (m1, m2)
}

/// Problem 11: Manual Clone implementation
/// Implement Clone manually
#[derive(Debug)]
pub struct MyStruct {
    pub data: Vec<i32>,
}

impl Clone for MyStruct {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
        }
    }
}

pub fn manual_clone() -> (MyStruct, MyStruct) {
    let s1 = MyStruct { data: vec![1, 2, 3] };
    let s2 = s1.clone();
    (s1, s2)
}

/// Problem 12: Copy with tuples
/// Tuples of Copy types are Copy
pub fn copy_tuple() -> ((i32, f64), (i32, f64)) {
    let t1 = (1, 2.0);
    let t2 = t1; // Copy
    (t1, t2)
}

/// Problem 13: Clone performance
/// Demonstrate clone performance implications
pub fn clone_performance() -> usize {
    let s = String::from("hello");
    let mut total = 0;
    for _ in 0..1000 {
        let clone = s.clone();
        total += clone.len();
    }
    total
}

/// Problem 14: Copy with bool
/// bool is Copy
pub fn copy_bool() -> (bool, bool) {
    let b1 = true;
    let b2 = b1; // Copy
    (b1, b2)
}

/// Problem 15: Clone with Box
/// Clone a Box (clones the contents)
pub fn clone_box() -> (Box<i32>, Box<i32>) {
    let b1 = Box::new(42);
    let b2 = b1.clone(); // Clone the contents
    (b1, b2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copy_example() {
        let (x, y) = copy_example();
        assert_eq!(x, 5);
        assert_eq!(y, 5);
    }

    #[test]
    fn test_clone_example() {
        let (s1, s2) = clone_example();
        assert_eq!(s1, "hello");
        assert_eq!(s2, "hello");
    }

    #[test]
    fn test_copy_vs_clone() {
        let (x, y, s1, s2) = copy_vs_clone();
        assert_eq!(x, 5);
        assert_eq!(y, 5);
        assert_eq!(s1, "hello");
        assert_eq!(s2, "hello");
    }

    #[test]
    fn test_point_copy() {
        let (p1, p2) = point_copy();
        assert_eq!(p1.x, 1.0);
        assert_eq!(p2.x, 1.0);
    }

    #[test]
    fn test_clone_vec() {
        let (v1, v2) = clone_vec();
        assert_eq!(v1, vec!["hello", "world"]);
        assert_eq!(v2, vec!["hello", "world"]);
    }

    #[test]
    fn test_copy_array() {
        let (a1, a2) = copy_array();
        assert_eq!(a1, [1, 2, 3]);
        assert_eq!(a2, [1, 2, 3]);
    }

    #[test]
    fn test_clone_option() {
        let (o1, o2) = clone_option();
        assert_eq!(o1, Some("hello".to_string()));
        assert_eq!(o2, Some("hello".to_string()));
    }

    #[test]
    fn test_clone_result() {
        let (r1, r2) = clone_result();
        assert_eq!(r1, Ok("hello".to_string()));
        assert_eq!(r2, Ok("hello".to_string()));
    }

    #[test]
    fn test_copy_reference() {
        let (s1, s2) = copy_reference();
        assert_eq!(s1, "hello");
        assert_eq!(s2, "hello");
    }

    #[test]
    fn test_clone_hashmap() {
        let (m1, m2) = clone_hashmap();
        assert_eq!(m1.get("key"), Some(&42));
        assert_eq!(m2.get("key"), Some(&42));
    }

    #[test]
    fn test_manual_clone() {
        let (s1, s2) = manual_clone();
        assert_eq!(s1.data, vec![1, 2, 3]);
        assert_eq!(s2.data, vec![1, 2, 3]);
    }

    #[test]
    fn test_copy_tuple() {
        let (t1, t2) = copy_tuple();
        assert_eq!(t1, (1, 2.0));
        assert_eq!(t2, (1, 2.0));
    }

    #[test]
    fn test_clone_performance() {
        assert_eq!(clone_performance(), 5000);
    }

    #[test]
    fn test_copy_bool() {
        let (b1, b2) = copy_bool();
        assert!(b1);
        assert!(b2);
    }

    #[test]
    fn test_clone_box() {
        let (b1, b2) = clone_box();
        assert_eq!(*b1, 42);
        assert_eq!(*b2, 42);
    }
}
