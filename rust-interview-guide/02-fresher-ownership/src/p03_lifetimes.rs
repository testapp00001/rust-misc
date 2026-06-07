/// Problem: Lifetimes
///
/// Master Rust's lifetime system.
///
/// Key Concepts:
/// - Lifetime annotations
/// - Lifetime elision rules
/// - Structs with lifetimes
/// - Static lifetime
/// - Lifetime subtyping

/// Problem 1: Basic lifetime annotation
/// Return the longer of two strings
pub fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}

/// Problem 2: Lifetime in struct
/// A struct that holds a reference
#[derive(Debug)]
pub struct Excerpt<'a> {
    pub part: &'a str,
}

impl<'a> Excerpt<'a> {
    pub fn new(text: &'a str) -> Self {
        Self { part: text }
    }

    pub fn level(&self) -> i32 {
        3
    }
}

/// Problem 3: Lifetime elision
/// The compiler can infer lifetimes in simple cases
pub fn first_word(s: &str) -> &str {
    let bytes = s.as_bytes();
    for (i, &byte) in bytes.iter().enumerate() {
        if byte == b' ' {
            return &s[..i];
        }
    }
    s
}

/// Problem 4: Multiple lifetime parameters
/// A function with two lifetime parameters
pub fn announce_and_return<'a, 'b>(announcement: &'a str, x: &'b str) -> &'b str {
    println!("Attention: {}", announcement);
    x
}

/// Problem 5: Static lifetime
/// A string that lives for the entire program
pub fn static_string() -> &'static str {
    "I live forever"
}

/// Problem 6: Lifetime and generics
/// Combine lifetimes with generics
pub fn longest_with_announcement<'a, T: std::fmt::Display>(
    x: &'a str,
    y: &'a str,
    ann: T,
) -> &'a str {
    println!("Announcement: {}", ann);
    if x.len() > y.len() { x } else { y }
}

/// Problem 7: Struct with multiple references
/// A struct with multiple references
#[derive(Debug)]
pub struct Pair<'a, 'b> {
    pub first: &'a str,
    pub second: &'b str,
}

impl<'a, 'b> Pair<'a, 'b> {
    pub fn new(first: &'a str, second: &'b str) -> Self {
        Self { first, second }
    }

    pub fn first(&self) -> &str {
        self.first
    }

    pub fn second(&self) -> &str {
        self.second
    }
}

/// Problem 8: Lifetime subtyping
/// 'a: 'b means 'a outlives 'b
pub fn longest_first<'a: 'b, 'b>(x: &'a str, y: &'b str) -> &'b str {
    if x.len() > y.len() { x } else { y }
}

/// Problem 9: Lifetime in method
/// Return a reference from a method
pub struct TextHolder<'a> {
    text: &'a str,
}

impl<'a> TextHolder<'a> {
    pub fn new(text: &'a str) -> Self {
        Self { text }
    }

    pub fn get_text(&self) -> &str {
        self.text
    }
}

/// Problem 10: Lifetime and closures
/// Return a closure that captures a reference
pub fn create_greeter<'a>(name: &'a str) -> impl Fn() -> String + 'a {
    move || format!("Hello, {}!", name)
}

/// Problem 11: Lifetime and Option
/// Return an Option with a reference
pub fn find_substring<'a>(s: &'a str, needle: &str) -> Option<&'a str> {
    s.find(needle).map(|i| &s[i..i + needle.len()])
}

/// Problem 12: Lifetime and Vec
/// Return a reference to an element in a Vec
pub fn find_max(v: &[i32]) -> Option<&i32> {
    v.iter().max()
}

/// Problem 13: Lifetime and HashMap
/// Return a reference to a value in a HashMap
pub fn find_in_map<'a>(
    map: &'a std::collections::HashMap<String, String>,
    key: &str,
) -> Option<&'a String> {
    map.get(key)
}

/// Problem 14: Lifetime and Result
/// Return a Result with a reference
pub fn parse_and_find<'a>(s: &'a str, separator: char) -> Result<&'a str, &'static str> {
    s.find(separator)
        .map(|i| &s[..i])
        .ok_or("Separator not found")
}

/// Problem 15: Lifetime and trait objects
/// Return a trait object with a lifetime
pub trait Describable {
    fn describe(&self) -> String;
}

pub struct Item {
    pub name: String,
}

impl Describable for Item {
    fn describe(&self) -> String {
        format!("Item: {}", self.name)
    }
}

pub fn create_describable(name: &str) -> Box<dyn Describable> {
    Box::new(Item {
        name: name.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_longest() {
        assert_eq!(longest("hello", "world!"), "world!");
        assert_eq!(longest("hi", "bye"), "bye");
    }

    #[test]
    fn test_excerpt() {
        let text = "hello world";
        let excerpt = Excerpt::new(text);
        assert_eq!(excerpt.part, "hello world");
        assert_eq!(excerpt.level(), 3);
    }

    #[test]
    fn test_first_word() {
        assert_eq!(first_word("hello world"), "hello");
        assert_eq!(first_word("hello"), "hello");
    }

    #[test]
    fn test_announce_and_return() {
        assert_eq!(announce_and_return("attention", "result"), "result");
    }

    #[test]
    fn test_static_string() {
        assert_eq!(static_string(), "I live forever");
    }

    #[test]
    fn test_longest_with_announcement() {
        assert_eq!(
            longest_with_announcement("hello", "world!", "Comparing strings"),
            "world!"
        );
    }

    #[test]
    fn test_pair() {
        let pair = Pair::new("hello", "world");
        assert_eq!(pair.first(), "hello");
        assert_eq!(pair.second(), "world");
    }

    #[test]
    fn test_longest_first() {
        assert_eq!(longest_first("hello", "hi"), "hello");
    }

    #[test]
    fn test_text_holder() {
        let holder = TextHolder::new("hello");
        assert_eq!(holder.get_text(), "hello");
    }

    #[test]
    fn test_create_greeter() {
        let greeter = create_greeter("Alice");
        assert_eq!(greeter(), "Hello, Alice!");
    }

    #[test]
    fn test_find_substring() {
        assert_eq!(find_substring("hello world", "world"), Some("world"));
        assert_eq!(find_substring("hello", "world"), None);
    }

    #[test]
    fn test_find_max() {
        let v = vec![1, 2, 3, 4, 5];
        assert_eq!(find_max(&v), Some(&5));

        let v: Vec<i32> = vec![];
        assert_eq!(find_max(&v), None);
    }

    #[test]
    fn test_find_in_map() {
        let mut map = std::collections::HashMap::new();
        map.insert("key".to_string(), "value".to_string());
        assert_eq!(find_in_map(&map, "key"), Some(&"value".to_string()));
        assert_eq!(find_in_map(&map, "missing"), None);
    }

    #[test]
    fn test_parse_and_find() {
        assert_eq!(parse_and_find("hello=world", '='), Ok("hello"));
        assert_eq!(parse_and_find("hello", '='), Err("Separator not found"));
    }

    #[test]
    fn test_create_describable() {
        let item = create_describable("test");
        assert_eq!(item.describe(), "Item: test");
    }
}
