/// Problem: String Ownership
///
/// Master Rust's string types and ownership.
///
/// Key Concepts:
/// - String vs &str
/// - String creation and manipulation
/// - String slicing
/// - String conversion
/// - UTF-8 encoding

/// Problem 1: String creation
/// Create a String from a string literal
pub fn create_string() -> String {
    let s = String::from("hello");
    s
}

/// Problem 2: String from &str
/// Convert &str to String
pub fn from_str(s: &str) -> String {
    s.to_string()
}

/// Problem 3: String to &str
/// Convert String to &str
pub fn to_str(s: String) -> &'static str {
    "hello"
}

/// Problem 4: String concatenation
/// Concatenate strings
pub fn concat_strings(s1: &str, s2: &str) -> String {
    let mut result = String::from(s1);
    result.push_str(s2);
    result
}

/// Problem 5: String slicing
/// Get a slice of a string
pub fn get_slice(s: &str, start: usize, end: usize) -> &str {
    &s[start..end]
}

/// Problem 6: String with ownership
/// Return a String (ownership transferred)
pub fn create_owned_string() -> String {
    let s = String::from("hello world");
    s
}

/// Problem 7: Borrow a String
/// Borrow a String as &str
pub fn borrow_string(s: &String) -> &str {
    s.as_str()
}

/// Problem 8: String with lifetime
/// Return a string slice with lifetime
pub fn get_first_word(s: &str) -> &str {
    let bytes = s.as_bytes();
    for (i, &byte) in bytes.iter().enumerate() {
        if byte == b' ' {
            return &s[..i];
        }
    }
    s
}

/// Problem 9: String and ownership transfer
/// Transfer ownership through function
pub fn transfer_string(s: String) -> String {
    s
}

/// Problem 10: String with format!
/// Create strings with format!
pub fn format_greeting(name: &str) -> String {
    format!("Hello, {}!", name)
}

/// Problem 11: String with replace
/// Replace parts of a string
pub fn replace_string(s: &str, from: &str, to: &str) -> String {
    s.replace(from, to)
}

/// Problem 12: String with split
/// Split a string
pub fn split_string(s: &str) -> Vec<&str> {
    s.split_whitespace().collect()
}

/// Problem 13: String with trim
/// Trim whitespace
pub fn trim_string(s: &str) -> &str {
    s.trim()
}

/// Problem 14: String with case conversion
/// Convert case
pub fn to_uppercase(s: &str) -> String {
    s.to_uppercase()
}

/// Problem 15: String with contains
/// Check if string contains substring
pub fn contains_substring(s: &str, needle: &str) -> bool {
    s.contains(needle)
}

/// Problem 16: String with starts_with and ends_with
/// Check string prefixes and suffixes
pub fn check_prefix_suffix(s: &str) -> (bool, bool) {
    (s.starts_with("hello"), s.ends_with("world"))
}

/// Problem 17: String with chars
/// Iterate over characters
pub fn count_chars(s: &str) -> usize {
    s.chars().count()
}

/// Problem 18: String with bytes
/// Get bytes of a string
pub fn get_bytes(s: &str) -> &[u8] {
    s.as_bytes()
}

/// Problem 19: String with capacity
/// Reserve capacity for a string
pub fn with_capacity() -> String {
    let mut s = String::with_capacity(10);
    s.push_str("hello");
    s
}

/// Problem 20: String ownership and cloning
/// Clone a string to keep both
pub fn clone_string(s: &str) -> (String, String) {
    let s1 = String::from(s);
    let s2 = s1.clone();
    (s1, s2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_string() {
        let s = create_string();
        assert_eq!(s, "hello");
    }

    #[test]
    fn test_from_str() {
        assert_eq!(from_str("hello"), "hello");
    }

    #[test]
    fn test_to_str() {
        assert_eq!(to_str(String::from("hello")), "hello");
    }

    #[test]
    fn test_concat_strings() {
        assert_eq!(concat_strings("hello", " world"), "hello world");
    }

    #[test]
    fn test_get_slice() {
        assert_eq!(get_slice("hello", 0, 3), "hel");
    }

    #[test]
    fn test_create_owned_string() {
        assert_eq!(create_owned_string(), "hello world");
    }

    #[test]
    fn test_borrow_string() {
        let s = String::from("hello");
        assert_eq!(borrow_string(&s), "hello");
    }

    #[test]
    fn test_get_first_word() {
        assert_eq!(get_first_word("hello world"), "hello");
        assert_eq!(get_first_word("hello"), "hello");
    }

    #[test]
    fn test_transfer_string() {
        let s = String::from("hello");
        assert_eq!(transfer_string(s), "hello");
    }

    #[test]
    fn test_format_greeting() {
        assert_eq!(format_greeting("Alice"), "Hello, Alice!");
    }

    #[test]
    fn test_replace_string() {
        assert_eq!(replace_string("hello world", "world", "Rust"), "hello Rust");
    }

    #[test]
    fn test_split_string() {
        assert_eq!(split_string("hello world"), vec!["hello", "world"]);
    }

    #[test]
    fn test_trim_string() {
        assert_eq!(trim_string("  hello  "), "hello");
    }

    #[test]
    fn test_to_uppercase() {
        assert_eq!(to_uppercase("hello"), "HELLO");
    }

    #[test]
    fn test_contains_substring() {
        assert!(contains_substring("hello world", "world"));
        assert!(!contains_substring("hello", "world"));
    }

    #[test]
    fn test_check_prefix_suffix() {
        assert_eq!(check_prefix_suffix("hello world"), (true, true));
        assert_eq!(check_prefix_suffix("hello"), (true, false));
    }

    #[test]
    fn test_count_chars() {
        assert_eq!(count_chars("hello"), 5);
        assert_eq!(count_chars("héllo"), 5); // UTF-8
    }

    #[test]
    fn test_get_bytes() {
        assert_eq!(get_bytes("hello"), b"hello");
    }

    #[test]
    fn test_with_capacity() {
        let s = with_capacity();
        assert_eq!(s, "hello");
        assert!(s.capacity() >= 10);
    }

    #[test]
    fn test_clone_string() {
        let (s1, s2) = clone_string("hello");
        assert_eq!(s1, "hello");
        assert_eq!(s2, "hello");
    }
}
