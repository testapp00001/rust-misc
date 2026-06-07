/// Problem: Borrowing
///
/// Master Rust's borrowing system.
///
/// Key Concepts:
/// - Immutable references (&T)
/// - Mutable references (&mut T)
/// - Borrowing rules
/// - Multiple immutable references
/// - One mutable reference

/// Problem 1: Immutable borrow
/// Borrow a value without taking ownership
pub fn calculate_length(s: &String) -> usize {
    s.len()
}

/// Problem 2: Multiple immutable borrows
/// You can have multiple immutable references
pub fn multiple_immutable_borrows(s: &String) -> (usize, usize) {
    let len1 = s.len();
    let len2 = s.len();
    (len1, len2)
}

/// Problem 3: Mutable borrow
/// Modify a value through a mutable reference
pub fn add_suffix(s: &mut String, suffix: &str) {
    s.push_str(suffix);
}

/// Problem 4: One mutable reference at a time
/// Only one mutable reference is allowed
pub fn modify_value(x: &mut i32) {
    *x += 1;
}

/// Problem 5: Borrowing and scope
/// References are scoped
pub fn borrow_scope() -> usize {
    let s = String::from("hello");
    let len = calculate_length(&s);
    // s is still valid here
    len
}

/// Problem 6: Borrowing in function calls
/// Pass references to functions
pub fn first_word(s: &str) -> &str {
    let bytes = s.as_bytes();
    for (i, &byte) in bytes.iter().enumerate() {
        if byte == b' ' {
            return &s[..i];
        }
    }
    s
}

/// Problem 7: Borrowing slices
/// Borrow a slice of a string
pub fn get_first_n_chars(s: &str, n: usize) -> &str {
    if n >= s.len() {
        s
    } else {
        &s[..n]
    }
}

/// Problem 8: Borrowing and ownership transfer
/// Return a reference to owned data
pub fn get_greeting(name: &str) -> String {
    format!("Hello, {}!", name)
}

/// Problem 9: Borrowing with Option
/// Return an Option with a reference
pub fn find_char(s: &str, c: char) -> Option<&str> {
    s.find(c).map(|i| &s[..=i])
}

/// Problem 10: Borrowing and lifetimes
/// Return a reference with explicit lifetime
pub fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}

/// Problem 11: Mutable borrow and read
/// Modify and then read
pub fn increment_and_read(x: &mut i32) -> i32 {
    *x += 1;
    *x
}

/// Problem 12: Borrowing in struct
/// Store a reference in a struct
#[derive(Debug)]
pub struct Excerpt<'a> {
    pub part: &'a str,
}

impl<'a> Excerpt<'a> {
    pub fn new(text: &'a str, separator: char) -> Self {
        let index = text.find(separator).unwrap_or(text.len());
        Self {
            part: &text[..index],
        }
    }
}

/// Problem 13: Borrowing and collections
/// Borrow elements from a collection
pub fn get_first_and_last(v: &[i32]) -> Option<(&i32, &i32)> {
    if v.is_empty() {
        None
    } else {
        Some((&v[0], &v[v.len() - 1]))
    }
}

/// Problem 14: Borrowing and pattern matching
/// Borrow while pattern matching
pub fn describe_option(opt: &Option<i32>) -> String {
    match opt {
        Some(x) => format!("Some({})", x),
        None => "None".to_string(),
    }
}

/// Problem 15: Borrowing and iterators
/// Borrow while iterating
pub fn sum_references(v: &[i32]) -> i32 {
    v.iter().sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_length() {
        let s = String::from("hello");
        assert_eq!(calculate_length(&s), 5);
    }

    #[test]
    fn test_multiple_immutable_borrows() {
        let s = String::from("hello");
        let (len1, len2) = multiple_immutable_borrows(&s);
        assert_eq!(len1, 5);
        assert_eq!(len2, 5);
    }

    #[test]
    fn test_add_suffix() {
        let mut s = String::from("hello");
        add_suffix(&mut s, " world");
        assert_eq!(s, "hello world");
    }

    #[test]
    fn test_modify_value() {
        let mut x = 5;
        modify_value(&mut x);
        assert_eq!(x, 6);
    }

    #[test]
    fn test_borrow_scope() {
        assert_eq!(borrow_scope(), 5);
    }

    #[test]
    fn test_first_word() {
        assert_eq!(first_word("hello world"), "hello");
        assert_eq!(first_word("hello"), "hello");
    }

    #[test]
    fn test_get_first_n_chars() {
        assert_eq!(get_first_n_chars("hello", 3), "hel");
        assert_eq!(get_first_n_chars("hello", 10), "hello");
    }

    #[test]
    fn test_get_greeting() {
        assert_eq!(get_greeting("Alice"), "Hello, Alice!");
    }

    #[test]
    fn test_find_char() {
        assert_eq!(find_char("hello", 'l'), Some("hel"));
        assert_eq!(find_char("hello", 'z'), None);
    }

    #[test]
    fn test_longest() {
        assert_eq!(longest("hello", "world!"), "world!");
        assert_eq!(longest("hi", "bye"), "bye");
    }

    #[test]
    fn test_increment_and_read() {
        let mut x = 5;
        assert_eq!(increment_and_read(&mut x), 6);
    }

    #[test]
    fn test_excerpt() {
        let text = "hello world";
        let excerpt = Excerpt::new(text, ' ');
        assert_eq!(excerpt.part, "hello");
    }

    #[test]
    fn test_get_first_and_last() {
        let v = vec![1, 2, 3, 4, 5];
        let (first, last) = get_first_and_last(&v).unwrap();
        assert_eq!(*first, 1);
        assert_eq!(*last, 5);

        let v: Vec<i32> = vec![];
        assert!(get_first_and_last(&v).is_none());
    }

    #[test]
    fn test_describe_option() {
        assert_eq!(describe_option(&Some(42)), "Some(42)");
        assert_eq!(describe_option(&None), "None");
    }

    #[test]
    fn test_sum_references() {
        let v = vec![1, 2, 3, 4, 5];
        assert_eq!(sum_references(&v), 15);
    }
}
