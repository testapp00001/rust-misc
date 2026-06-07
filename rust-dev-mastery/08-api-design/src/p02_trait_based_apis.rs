//! # Trait-Based APIs
//!
//! Traits are the primary mechanism for designing extensible Rust APIs.
//! This module covers object safety, trait bounds, associated types, and
//! best practices for trait design.
//!
//! ## Key Concepts
//! - **Object safety**: Which traits can be used as `dyn Trait`
//! - **Trait bounds**: Constraining generic parameters
//! - **Associated types**: Type-level outputs from traits
//! - **Extension traits**: Adding methods to foreign types

/// A trait with an associated type — the implementor chooses the output type.
pub trait Serializer {
    type Output;
    type Error;

    fn serialize(&self, value: &dyn std::fmt::Debug) -> Result<Self::Output, Self::Error>;
}

pub struct JsonSerializer;
pub struct CsvSerializer;

impl Serializer for JsonSerializer {
    type Output = String;
    type Error = String;

    fn serialize(&self, value: &dyn std::fmt::Debug) -> Result<String, String> {
        Ok(format!("{value:?}"))
    }
}

impl Serializer for CsvSerializer {
    type Output = Vec<u8>;
    type Error = String;

    fn serialize(&self, value: &dyn std::fmt::Debug) -> Result<Vec<u8>, String> {
        Ok(format!("{value:?}").into_bytes())
    }
}

/// Demonstrates trait bounds for flexible generic constraints.
pub fn process_items<T, I>(items: I) -> Vec<T>
where
    T: Clone + Ord + std::fmt::Display,
    I: IntoIterator<Item = T>,
{
    let mut sorted: Vec<T> = items.into_iter().collect();
    sorted.sort();
    sorted
}

/// A trait that IS object-safe (can be used as dyn Trait).
pub trait Processor: Send + Sync {
    fn process(&self, input: &str) -> String;
    fn name(&self) -> &str;
}

pub struct UpperCaseProcessor;
pub struct LowerCaseProcessor;

impl Processor for UpperCaseProcessor {
    fn process(&self, input: &str) -> String {
        input.to_uppercase()
    }
    fn name(&self) -> &str {
        "uppercase"
    }
}

impl Processor for LowerCaseProcessor {
    fn process(&self, input: &str) -> String {
        input.to_lowercase()
    }
    fn name(&self) -> &str {
        "lowercase"
    }
}

/// Uses trait objects for runtime polymorphism.
pub fn apply_processors(processors: &[Box<dyn Processor>], input: &str) -> Vec<String> {
    processors.iter().map(|p| p.process(input)).collect()
}

/// A trait that is NOT object-safe (has generic methods).
/// This can only be used with static dispatch.
pub trait GenericProcessor {
    fn process<T: std::fmt::Display>(&self, input: T) -> String;
}

/// Demonstrates associated type vs generic parameter tradeoffs.
///
/// Associated type: one implementation = one output type.
/// Generic parameter: one implementation can handle many input types.
pub trait Transform {
    type Input;
    type Output;

    fn transform(&self, input: Self::Input) -> Self::Output;
}

pub struct Doubler;
impl Transform for Doubler {
    type Input = i32;
    type Output = i32;

    fn transform(&self, input: i32) -> i32 {
        input * 2
    }
}

/// A collection trait with associated types for iterators.
pub trait Collection {
    type Item;
    type Iter<'a>: Iterator<Item = &'a Self::Item>
    where
        Self: 'a;

    fn iter(&self) -> Self::Iter<'_>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// A simple stack implementing the Collection trait.
pub struct Stack<T> {
    items: Vec<T>,
}

impl<T> Stack<T> {
    pub fn new() -> Self {
        Stack { items: Vec::new() }
    }

    pub fn push(&mut self, item: T) {
        self.items.push(item);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.items.pop()
    }
}

impl<T> Collection for Stack<T> {
    type Item = T;
    type Iter<'a> = std::slice::Iter<'a, T> where T: 'a;

    fn iter(&self) -> std::slice::Iter<'_, T> {
        self.items.iter()
    }

    fn len(&self) -> usize {
        self.items.len()
    }
}

/// An extension trait that adds convenience methods to iterators.
pub trait IteratorExt: Iterator {
    /// Collects into a Vec and sorts by the given key.
    fn sorted_by_key<K: Ord>(self, f: impl FnMut(&Self::Item) -> K) -> Vec<Self::Item>
    where
        Self: Sized,
    {
        let mut items: Vec<Self::Item> = self.collect();
        items.sort_by_key(f);
        items
    }

    /// Returns the first N items as a Vec.
    fn take_vec(self, n: usize) -> Vec<Self::Item>
    where
        Self: Sized,
    {
        self.take(n).collect()
    }
}

/// Blanket implementation: all iterators get these methods.
impl<I: Iterator> IteratorExt for I {}

/// A builder trait for consistent API across multiple builders.
pub trait Buildable {
    type Output;
    type Error;

    fn build(self) -> Result<Self::Output, Self::Error>;
}

#[derive(Debug, Clone)]
pub struct Server {
    pub host: String,
    pub port: u16,
}

pub struct ServerBuilder {
    host: Option<String>,
    port: u16,
}

impl ServerBuilder {
    pub fn new() -> Self {
        ServerBuilder {
            host: None,
            port: 8080,
        }
    }

    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.host = Some(host.into());
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }
}

impl Buildable for ServerBuilder {
    type Output = Server;
    type Error = String;

    fn build(self) -> Result<Server, String> {
        Ok(Server {
            host: self.host.ok_or("host required")?,
            port: self.port,
        })
    }
}

/// Demonstrates the "newtype + trait impl" pattern for adding
/// trait implementations to foreign types.
pub struct DisplayAdapter<T>(pub T);

impl<T: std::fmt::Debug> std::fmt::Display for DisplayAdapter<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serializer_json() {
        let s = JsonSerializer;
        let result = s.serialize(&42).unwrap();
        assert_eq!(result, "42");
    }

    #[test]
    fn test_serializer_csv() {
        let s = CsvSerializer;
        let result = s.serialize(&"hello").unwrap();
        assert!(!result.is_empty());
    }

    #[test]
    fn test_process_items() {
        let result = process_items(vec![3, 1, 4, 1, 5, 9]);
        assert_eq!(result, vec![1, 1, 3, 4, 5, 9]);
    }

    #[test]
    fn test_apply_processors() {
        let processors: Vec<Box<dyn Processor>> = vec![
            Box::new(UpperCaseProcessor),
            Box::new(LowerCaseProcessor),
        ];

        let results = apply_processors(&processors, "Hello World");
        assert_eq!(results, vec!["HELLO WORLD", "hello world"]);
    }

    #[test]
    fn test_transform() {
        let doubler = Doubler;
        assert_eq!(doubler.transform(5), 10);
        assert_eq!(doubler.transform(-3), -6);
    }

    #[test]
    fn test_stack_collection() {
        let mut stack = Stack::new();
        stack.push(1);
        stack.push(2);
        stack.push(3);

        assert_eq!(stack.len(), 3);
        assert!(!stack.is_empty());

        let items: Vec<&i32> = stack.iter().collect();
        assert_eq!(items, vec![&1, &2, &3]);
    }

    #[test]
    fn test_stack_pop() {
        let mut stack = Stack::new();
        stack.push(10);
        stack.push(20);

        assert_eq!(stack.pop(), Some(20));
        assert_eq!(stack.pop(), Some(10));
        assert_eq!(stack.pop(), None);
    }

    #[test]
    fn test_iterator_ext_sorted() {
        let sorted = vec![3, 1, 4, 1, 5].into_iter().sorted_by_key(|&x| x);
        assert_eq!(sorted, vec![1, 1, 3, 4, 5]);
    }

    #[test]
    fn test_iterator_ext_take_vec() {
        let items = (0..100).take_vec(5);
        assert_eq!(items, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_buildable_server() {
        let server = ServerBuilder::new()
            .host("localhost")
            .port(8080)
            .build()
            .unwrap();

        assert_eq!(server.host, "localhost");
        assert_eq!(server.port, 8080);
    }

    #[test]
    fn test_buildable_missing_host() {
        let result = ServerBuilder::new().port(8080).build();
        assert!(result.is_err());
    }

    #[test]
    fn test_display_adapter() {
        let adapter = DisplayAdapter(vec![1, 2, 3]);
        assert_eq!(format!("{adapter}"), "[1, 2, 3]");
    }
}
