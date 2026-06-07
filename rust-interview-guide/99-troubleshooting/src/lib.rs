/// Rust Troubleshooting Guide
///
/// This module contains examples of common errors and their fixes.
/// Each example demonstrates a specific error pattern and the correct solution.

/// Example 1: Ownership and Borrowing
pub mod ownership_examples {
    /// ERROR: cannot move out of borrowed content
    /// ```compile_fail
    /// let s = String::from("hello");
    /// let r = &s;
    /// let s2 = *r; // ERROR
    /// ```
    pub fn move_from_borrow() -> String {
        let s = String::from("hello");
        let r = &s;
        r.clone() // FIX: Use clone() instead of dereference
    }

    /// ERROR: cannot borrow as mutable because it is also borrowed as immutable
    /// ```compile_fail
    /// let mut data = vec![1, 2, 3];
    /// let first = &data[0];
    /// data.push(4); // ERROR
    /// println!("{}", first);
    /// ```
    pub fn mutable_while_borrowed() -> i32 {
        let mut data = vec![1, 2, 3];
        let first = data[0]; // FIX: Copy the value
        data.push(4);
        first + data[3]
    }

    /// ERROR: use of moved value
    /// ```compile_fail
    /// let s = String::from("hello");
    /// let s2 = s;
    /// println!("{}", s); // ERROR
    /// ```
    pub fn use_after_move() -> (String, String) {
        let s = String::from("hello");
        let s2 = s.clone(); // FIX: Clone instead of move
        (s, s2)
    }
}

/// Example 2: Lifetime Errors
pub mod lifetime_examples {
    /// ERROR: missing lifetime specifier
    /// ```compile_fail
    /// fn longest(x: &str, y: &str) -> &str {
    ///     if x.len() > y.len() { x } else { y }
    /// }
    /// ```
    pub fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
        if x.len() > y.len() { x } else { y }
    }

    /// ERROR: borrowed value does not live long enough
    /// ```compile_fail
    /// let r;
    /// {
    ///     let x = 5;
    ///     r = &x;
    /// }
    /// println!("{}", r); // ERROR
    /// ```
    pub fn lifetime_scope() -> i32 {
        let x = 5;
        let r = &x;
        *r
    }
}

/// Example 3: Concurrency Errors
pub mod concurrency_examples {
    use std::sync::{Arc, Mutex};
    use std::thread;

    /// ERROR: Receiver cannot be shared between threads
    /// ```compile_fail
    /// use std::sync::{Arc, mpsc};
    /// use std::thread;
    ///
    /// let (tx, rx) = mpsc::channel();
    /// let rx = Arc::new(rx);
    /// for _ in 0..3 {
    ///     let rx = Arc::clone(&rx);
    ///     thread::spawn(move || {
    ///         rx.recv(); // ERROR
    ///     });
    /// }
    /// ```
    pub fn single_receiver() -> Vec<i32> {
        use std::sync::mpsc;
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            for i in 0..5 {
                tx.send(i).unwrap();
            }
        });

        rx.iter().collect()
    }

    /// ERROR: MutexGuard dropped while borrowed
    /// ```compile_fail
    /// let data = Arc::new(Mutex::new(0));
    /// let result = data.lock().unwrap() + 1; // ERROR
    /// ```
    pub fn mutex_guard_lifetime() -> i32 {
        let data = Arc::new(Mutex::new(0));
        let result = *data.lock().unwrap() + 1; // FIX: Store guard
        result
    }

    /// ERROR: Moved value in loop
    /// ```compile_fail
    /// let data = vec![1, 2, 3, 4, 5];
    /// for i in 0..data.len() {
    ///     thread::spawn(move || {
    ///         data[i] * 2 // ERROR
    ///     });
    /// }
    /// ```
    pub fn shared_across_threads() -> Vec<i32> {
        let data = Arc::new(vec![1, 2, 3, 4, 5]);
        let mut handles = vec![];

        for i in 0..5 {
            let data = Arc::clone(&data);
            handles.push(thread::spawn(move || data[i] * 2));
        }

        handles.into_iter().map(|h| h.join().unwrap()).collect()
    }
}

/// Example 4: Type Errors
pub mod type_examples {
    /// ERROR: type annotations needed
    /// ```compile_fail
    /// let x = "42".parse().unwrap(); // ERROR
    /// ```
    pub fn parse_with_annotation() -> i32 {
        "42".parse::<i32>().unwrap() // FIX: Add type annotation
    }

    /// ERROR: trait not implemented
    /// ```compile_fail
    /// struct MyStruct { data: i32 }
    /// let s = MyStruct { data: 42 };
    /// s.clone(); // ERROR
    /// ```
    #[derive(Clone)]
    pub struct MyStruct {
        pub data: i32,
    }

    /// ERROR: cannot implement Copy for type with String
    /// ```compile_fail
    /// #[derive(Copy, Clone)]
    /// struct MyStruct {
    ///     data: String, // ERROR
    /// }
    /// ```
    #[derive(Clone)] // FIX: Only Clone, not Copy
    pub struct MyStringStruct {
        pub data: String,
    }
}

/// Example 5: Pattern Matching Errors
pub mod pattern_examples {
    #[derive(Debug)]
    pub enum Color {
        Red,
        Green,
        Blue,
    }

    /// ERROR: non-exhaustive patterns
    /// ```compile_fail
    /// match color {
    ///     Color::Red => {},
    ///     Color::Green => {},
    ///     // Missing Blue!
    /// }
    /// ```
    pub fn match_color(color: Color) -> &'static str {
        match color {
            Color::Red => "red",
            Color::Green => "green",
            Color::Blue => "blue", // FIX: Add missing arm
        }
    }
}

/// Example 6: Async Errors
pub mod async_examples {
    use std::future::Future;
    use std::pin::Pin;

    /// Working solution: Use boxed futures instead of async fn in traits
    /// This avoids the limitations of async fn in public traits
    pub trait MyTrait {
        fn my_method(&self) -> Pin<Box<dyn Future<Output = i32> + Send>>;
    }

    pub struct MyStruct;

    impl MyTrait for MyStruct {
        fn my_method(&self) -> Pin<Box<dyn Future<Output = i32> + Send>> {
            Box::pin(async { 42 })
        }
    }
}

/// Example 7: Common Patterns
pub mod patterns {
    /// Builder pattern
    pub struct Builder {
        value: i32,
        name: String,
    }

    impl Builder {
        pub fn new() -> Self {
            Self {
                value: 0,
                name: String::new(),
            }
        }

        pub fn value(mut self, value: i32) -> Self {
            self.value = value;
            self
        }

        pub fn name(mut self, name: &str) -> Self {
            self.name = name.to_string();
            self
        }

        pub fn build(self) -> Config {
            Config {
                value: self.value,
                name: self.name,
            }
        }
    }

    pub struct Config {
        pub value: i32,
        pub name: String,
    }

    /// RAII pattern
    pub struct Resource {
        name: String,
    }

    impl Resource {
        pub fn new(name: &str) -> Self {
            println!("Acquiring resource: {}", name);
            Self {
                name: name.to_string(),
            }
        }
    }

    impl Drop for Resource {
        fn drop(&mut self) {
            println!("Releasing resource: {}", self.name);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ownership_examples() {
        let s = ownership_examples::move_from_borrow();
        assert_eq!(s, "hello");

        let result = ownership_examples::mutable_while_borrowed();
        assert_eq!(result, 5);
    }

    #[test]
    fn test_lifetime_examples() {
        let result = lifetime_examples::longest("hello", "world!");
        assert_eq!(result, "world!");

        let result = lifetime_examples::lifetime_scope();
        assert_eq!(result, 5);
    }

    #[test]
    fn test_concurrency_examples() {
        let results = concurrency_examples::single_receiver();
        assert_eq!(results, vec![0, 1, 2, 3, 4]);

        let result = concurrency_examples::mutex_guard_lifetime();
        assert_eq!(result, 1);

        let results = concurrency_examples::shared_across_threads();
        assert_eq!(results, vec![2, 4, 6, 8, 10]);
    }

    #[test]
    fn test_type_examples() {
        let x = type_examples::parse_with_annotation();
        assert_eq!(x, 42);

        let s = type_examples::MyStruct { data: 42 };
        let s2 = s.clone();
        assert_eq!(s2.data, 42);
    }

    #[test]
    fn test_pattern_examples() {
        assert_eq!(pattern_examples::match_color(pattern_examples::Color::Red), "red");
        assert_eq!(pattern_examples::match_color(pattern_examples::Color::Blue), "blue");
    }

    #[test]
    fn test_patterns() {
        let config = patterns::Builder::new()
            .value(42)
            .name("test")
            .build();
        assert_eq!(config.value, 42);
        assert_eq!(config.name, "test");
    }
}
