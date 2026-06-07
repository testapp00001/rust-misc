/// Problem: Struct Ownership
///
/// Master ownership in structs.
///
/// Key Concepts:
/// - Structs that own their data
/// - Structs with references (lifetimes)
/// - Moving structs
/// - Cloning structs
/// - Structs with smart pointers

use std::rc::Rc;
use std::cell::RefCell;

/// Problem 1: Struct that owns data
/// A struct with owned String
#[derive(Debug, Clone)]
pub struct Person {
    pub name: String,
    pub age: u32,
}

impl Person {
    pub fn new(name: &str, age: u32) -> Self {
        Self {
            name: name.to_string(),
            age,
        }
    }
}

/// Problem 2: Struct with reference
/// A struct with a borrowed string
#[derive(Debug)]
pub struct Excerpt<'a> {
    pub text: &'a str,
}

impl<'a> Excerpt<'a> {
    pub fn new(text: &'a str) -> Self {
        Self { text }
    }
}

/// Problem 3: Struct with multiple owned fields
/// A struct with multiple owned fields
#[derive(Debug)]
pub struct Document {
    pub title: String,
    pub content: String,
    pub author: String,
}

impl Document {
    pub fn new(title: &str, content: &str, author: &str) -> Self {
        Self {
            title: title.to_string(),
            content: content.to_string(),
            author: author.to_string(),
        }
    }
}

/// Problem 4: Moving a struct
/// Move ownership of a struct
pub fn move_person() -> Person {
    let p = Person::new("Alice", 30);
    p // Ownership moved to caller
}

/// Problem 5: Cloning a struct
/// Clone a struct to create a deep copy
pub fn clone_person() -> (Person, Person) {
    let p1 = Person::new("Alice", 30);
    let p2 = p1.clone(); // Requires #[derive(Clone)]
    (p1, p2)
}

/// Problem 6: Struct with Vec
/// A struct that owns a Vec
#[derive(Debug)]
pub struct Team {
    pub name: String,
    pub members: Vec<String>,
}

impl Team {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            members: Vec::new(),
        }
    }

    pub fn add_member(&mut self, member: &str) {
        self.members.push(member.to_string());
    }
}

/// Problem 7: Struct with Rc for shared ownership
/// Use Rc for shared ownership
#[derive(Debug)]
pub struct SharedData {
    pub data: Rc<String>,
}

impl SharedData {
    pub fn new(data: &str) -> Self {
        Self {
            data: Rc::new(data.to_string()),
        }
    }
}

/// Problem 8: Struct with RefCell for interior mutability
/// Use RefCell for interior mutability
#[derive(Debug)]
pub struct MutableStruct {
    pub data: RefCell<i32>,
}

impl MutableStruct {
    pub fn new(value: i32) -> Self {
        Self {
            data: RefCell::new(value),
        }
    }

    pub fn increment(&self) {
        *self.data.borrow_mut() += 1;
    }

    pub fn get(&self) -> i32 {
        *self.data.borrow()
    }
}

/// Problem 9: Struct with Box for heap allocation
/// Use Box for large data
#[derive(Debug)]
pub struct LargeStruct {
    pub data: Box<[i32; 1000]>,
}

impl LargeStruct {
    pub fn new() -> Self {
        Self {
            data: Box::new([0; 1000]),
        }
    }
}

/// Problem 10: Struct with ownership transfer
/// Transfer ownership of struct fields
pub fn transfer_fields() -> (String, u32) {
    let p = Person::new("Alice", 30);
    (p.name, p.age) // Destructure moves fields
}

/// Problem 11: Struct with lifetime and methods
/// Methods that return references
pub struct TextProcessor<'a> {
    text: &'a str,
}

impl<'a> TextProcessor<'a> {
    pub fn new(text: &'a str) -> Self {
        Self { text }
    }

    pub fn first_word(&self) -> &str {
        self.text.split_whitespace().next().unwrap_or("")
    }

    pub fn last_word(&self) -> &str {
        self.text.split_whitespace().last().unwrap_or("")
    }
}

/// Problem 12: Struct with owned and borrowed fields
/// Mix owned and borrowed fields
pub struct Article<'a> {
    pub title: String,      // Owned
    pub content: &'a str,   // Borrowed
    pub author: String,     // Owned
}

impl<'a> Article<'a> {
    pub fn new(title: &str, content: &'a str, author: &str) -> Self {
        Self {
            title: title.to_string(),
            content,
            author: author.to_string(),
        }
    }
}

/// Problem 13: Struct with Option<String>
/// Optional owned data
#[derive(Debug)]
pub struct User {
    pub name: String,
    pub email: Option<String>,
}

impl User {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            email: None,
        }
    }

    pub fn with_email(mut self, email: &str) -> Self {
        self.email = Some(email.to_string());
        self
    }
}

/// Problem 14: Struct with Result field
/// Result as a field type
#[derive(Debug)]
pub struct Config {
    pub value: Result<String, String>,
}

impl Config {
    pub fn new(value: Result<String, String>) -> Self {
        Self { value }
    }
}

/// Problem 15: Struct with closure
/// Store a closure in a struct
pub struct Processor {
    pub transform: Box<dyn Fn(i32) -> i32>,
}

impl Processor {
    pub fn new(transform: Box<dyn Fn(i32) -> i32>) -> Self {
        Self { transform }
    }

    pub fn process(&self, value: i32) -> i32 {
        (self.transform)(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_person() {
        let p = Person::new("Alice", 30);
        assert_eq!(p.name, "Alice");
        assert_eq!(p.age, 30);
    }

    #[test]
    fn test_excerpt() {
        let text = "hello world";
        let excerpt = Excerpt::new(text);
        assert_eq!(excerpt.text, "hello world");
    }

    #[test]
    fn test_document() {
        let doc = Document::new("Title", "Content", "Author");
        assert_eq!(doc.title, "Title");
    }

    #[test]
    fn test_move_person() {
        let p = move_person();
        assert_eq!(p.name, "Alice");
    }

    #[test]
    fn test_clone_person() {
        let (p1, p2) = clone_person();
        assert_eq!(p1.name, "Alice");
        assert_eq!(p2.name, "Alice");
    }

    #[test]
    fn test_team() {
        let mut team = Team::new("Engineering");
        team.add_member("Alice");
        team.add_member("Bob");
        assert_eq!(team.members, vec!["Alice", "Bob"]);
    }

    #[test]
    fn test_shared_data() {
        let d1 = SharedData::new("hello");
        let d2 = SharedData::new("hello");
        assert_eq!(*d1.data, "hello");
        assert_eq!(*d2.data, "hello");
    }

    #[test]
    fn test_mutable_struct() {
        let s = MutableStruct::new(5);
        assert_eq!(s.get(), 5);
        s.increment();
        assert_eq!(s.get(), 6);
    }

    #[test]
    fn test_large_struct() {
        let s = LargeStruct::new();
        assert_eq!(s.data.len(), 1000);
    }

    #[test]
    fn test_transfer_fields() {
        let (name, age) = transfer_fields();
        assert_eq!(name, "Alice");
        assert_eq!(age, 30);
    }

    #[test]
    fn test_text_processor() {
        let tp = TextProcessor::new("hello world foo");
        assert_eq!(tp.first_word(), "hello");
        assert_eq!(tp.last_word(), "foo");
    }

    #[test]
    fn test_article() {
        let content = "Article content";
        let article = Article::new("Title", content, "Author");
        assert_eq!(article.title, "Title");
        assert_eq!(article.content, "Article content");
    }

    #[test]
    fn test_user() {
        let user = User::new("Alice").with_email("alice@example.com");
        assert_eq!(user.name, "Alice");
        assert_eq!(user.email, Some("alice@example.com".to_string()));
    }

    #[test]
    fn test_config() {
        let config = Config::new(Ok("value".to_string()));
        assert_eq!(config.value, Ok("value".to_string()));
    }

    #[test]
    fn test_processor() {
        let p = Processor::new(Box::new(|x| x * 2));
        assert_eq!(p.process(5), 10);
    }
}
